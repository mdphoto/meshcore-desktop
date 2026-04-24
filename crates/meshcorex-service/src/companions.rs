//! Gestion des companions (dispositifs BLE/série/TCP déjà connectés).
//!
//! À chaque connexion réussie, on enregistre le dispositif dans la table
//! `companions` pour permettre une reconnexion rapide sans rescan.

use crate::AppState;
use meshcorex_storage::models::StoredCompanion;
use meshcorex_transport::manager::ConnectionTarget;

/// Enregistre (ou met à jour) un companion dans la DB à partir d'un ConnectionTarget.
/// Appelé juste après une connexion réussie.
pub fn record_companion(state: &AppState, target: &ConnectionTarget) -> Result<(), String> {
    let (transport_type, name, address) = match target {
        ConnectionTarget::Ble { name_or_addr } => {
            ("ble", name_or_addr.clone(), name_or_addr.clone())
        }
        ConnectionTarget::Serial { port, baud_rate } => (
            "serial",
            format!("{} @ {}", port, baud_rate),
            port.clone(),
        ),
        ConnectionTarget::Tcp { host, port } => (
            "tcp",
            format!("{}:{}", host, port),
            format!("{}:{}", host, port),
        ),
    };
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let entry = StoredCompanion {
        id: None,
        transport_type: transport_type.to_string(),
        name,
        address,
        pin: None,
        last_used: now,
    };
    state
        .db
        .with_conn(|c| meshcorex_storage::companions::upsert_companion(c, &entry))
        .map_err(|e| e.to_string())
}

/// Liste tous les companions (triés par dernière utilisation desc).
pub fn list_companions(state: &AppState) -> Result<Vec<StoredCompanion>, String> {
    state
        .db
        .with_conn(meshcorex_storage::companions::get_all_companions)
        .map_err(|e| e.to_string())
}

/// Supprime un companion par id.
pub fn delete_companion(state: &AppState, id: i64) -> Result<bool, String> {
    state
        .db
        .with_conn(|c| meshcorex_storage::companions::delete_companion(c, id))
        .map_err(|e| e.to_string())
}

/// Reconstruit un ConnectionTarget depuis un companion stocké (pour reconnexion).
pub fn companion_to_target(c: &StoredCompanion) -> Option<ConnectionTarget> {
    match c.transport_type.as_str() {
        "ble" => Some(ConnectionTarget::Ble {
            name_or_addr: c.address.clone(),
        }),
        "serial" => Some(ConnectionTarget::Serial {
            port: c.address.clone(),
            baud_rate: 115200,
        }),
        "tcp" => {
            let (host, port) = c.address.rsplit_once(':')?;
            Some(ConnectionTarget::Tcp {
                host: host.to_string(),
                port: port.parse().ok()?,
            })
        }
        _ => None,
    }
}
