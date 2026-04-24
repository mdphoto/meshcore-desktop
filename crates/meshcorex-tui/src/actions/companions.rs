use chrono::Utc;
use meshcorex_service::AppState;
use meshcorex_storage::companions;
use meshcorex_storage::models::StoredCompanion;
use meshcorex_transport::manager::ConnectionTarget;
use std::sync::Arc;

/// Dérive un StoredCompanion depuis une ConnectionTarget + nom logique
pub fn companion_from_target(target: &ConnectionTarget, name: &str) -> StoredCompanion {
    let (transport_type, address) = match target {
        ConnectionTarget::Ble { name_or_addr } => ("ble".to_string(), name_or_addr.clone()),
        ConnectionTarget::Serial { port, .. } => ("serial".to_string(), port.clone()),
        ConnectionTarget::Tcp { host, port } => {
            ("tcp".to_string(), format!("{}:{}", host, port))
        }
    };
    StoredCompanion {
        id: None,
        transport_type,
        name: name.to_string(),
        address,
        pin: None,
        last_used: Utc::now().to_rfc3339(),
    }
}

/// Convertit un companion stocké en ConnectionTarget (None si type inconnu ou TCP mal formé)
pub fn target_from_companion(companion: &StoredCompanion) -> Option<ConnectionTarget> {
    match companion.transport_type.as_str() {
        "ble" => Some(ConnectionTarget::Ble {
            name_or_addr: companion.address.clone(),
        }),
        "serial" => Some(ConnectionTarget::Serial {
            port: companion.address.clone(),
            baud_rate: 115_200,
        }),
        "tcp" => {
            let (host, port) = companion.address.rsplit_once(':')?;
            let port = port.parse::<u16>().ok()?;
            Some(ConnectionTarget::Tcp {
                host: host.to_string(),
                port,
            })
        }
        _ => None,
    }
}

/// Sauvegarde/rafraîchit le companion pour la connexion primaire actuelle.
/// Ne fait rien si aucune connexion n'est active.
pub async fn save_current_primary(service: Arc<AppState>, device_name: String) {
    let conn = service.connection.read().await;
    let Some(target) = conn.target().cloned() else {
        return;
    };
    drop(conn);
    let companion = companion_from_target(&target, &device_name);
    let _ = service
        .db
        .with_conn(|c| companions::upsert_companion(c, &companion));
}

/// Tente l'auto-reconnexion au dernier companion connu. Non bloquant.
pub async fn try_auto_reconnect(service: Arc<AppState>) -> Result<String, String> {
    let list = service
        .db
        .with_conn(companions::get_all_companions)
        .map_err(|e| e.to_string())?;

    let Some(last) = list.first() else {
        return Err("Aucun companion connu".to_string());
    };

    let Some(target) = target_from_companion(last) else {
        return Err(format!(
            "Type de transport inconnu : {}",
            last.transport_type
        ));
    };

    tracing::info!("Auto-reconnexion au dernier companion : {:?}", target);
    meshcorex_service::connection::connect(&service, target).await?;
    Ok(last.name.clone())
}
