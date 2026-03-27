//! Commandes Tauri pour l'administration des repeaters/relais
//!
//! Architecture bi-protocole :
//! - Binaire (meshcore-rs) : status, batterie, voisins, ACL, télémétrie
//! - CLI texte (send_msg) : configuration avancée (password, radio, GPS, regions, logs...)

use meshcore_rs::events::{EventPayload, EventType};
use meshcore_service::AppState;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;

/// Timeout pour les requêtes binaires vers les repeaters (BLE multi-hop = lent)
const REPEATER_TIMEOUT: Duration = Duration::from_secs(30);

/// Statut détaillé d'un repeater
#[derive(Debug, Clone, Serialize)]
pub struct RepeaterStatus {
    pub battery_mv: u16,
    pub tx_queue_len: u16,
    pub noise_floor: i16,
    pub last_rssi: i16,
    pub nb_recv: u32,
    pub nb_sent: u32,
    pub airtime: u32,
    pub uptime: u32,
    pub flood_sent: u32,
    pub direct_sent: u32,
    pub snr: f32,
    pub dup_count: u32,
    pub rx_airtime: u32,
}

/// Voisin d'un repeater
#[derive(Debug, Clone, Serialize)]
pub struct RepeaterNeighbour {
    pub pubkey_hex: String,
    pub secs_ago: i32,
    pub snr: f32,
    pub name: Option<String>,
}

/// Entrée ACL
#[derive(Debug, Clone, Serialize)]
pub struct AclEntry {
    pub pubkey_hex: String,
    pub permissions: u8,
    pub name: Option<String>,
}

// ═══════════════════════════════════════════════════════════
// Commandes binaires (protocole meshcore-rs natif)
// ═══════════════════════════════════════════════════════════

#[tauri::command]
pub async fn repeater_login(
    state: State<'_, AppState>,
    pubkey: String,
    password: String,
) -> Result<String, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .send_login(pubkey.as_str(), &password)
        .await
        .map_err(|e| format!("Login repeater échoué : {}", e))?;
    tracing::info!("Login repeater {}", &pubkey[..12.min(pubkey.len())]);
    Ok("Login envoyé".to_string())
}

#[tauri::command]
pub async fn repeater_logout(state: State<'_, AppState>, pubkey: String) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .send_logout(pubkey.as_str())
        .await
        .map_err(|e| format!("Logout repeater échoué : {}", e))?;
    tracing::info!("Logout repeater {}", &pubkey[..12.min(pubkey.len())]);
    Ok(())
}

#[tauri::command]
pub async fn repeater_status(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<RepeaterStatus, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let s = mc
        .commands()
        .lock()
        .await
        .request_status_with_timeout(pubkey.as_str(), REPEATER_TIMEOUT)
        .await
        .map_err(|e| format!("Erreur statut repeater : {}", e))?;
    Ok(RepeaterStatus {
        battery_mv: s.battery_mv,
        tx_queue_len: s.tx_queue_len,
        noise_floor: s.noise_floor,
        last_rssi: s.last_rssi,
        nb_recv: s.nb_recv,
        nb_sent: s.nb_sent,
        airtime: s.airtime,
        uptime: s.uptime,
        flood_sent: s.flood_sent,
        direct_sent: s.direct_sent,
        snr: s.snr,
        dup_count: s.dup_count,
        rx_airtime: s.rx_airtime,
    })
}

#[tauri::command]
pub async fn repeater_neighbours(
    state: State<'_, AppState>,
    pubkey: String,
    count: u16,
    offset: u16,
) -> Result<Vec<RepeaterNeighbour>, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let data = mc
        .commands()
        .lock()
        .await
        .request_neighbours_with_timeout(pubkey.as_str(), count, offset, REPEATER_TIMEOUT)
        .await
        .map_err(|e| format!("Erreur voisins : {}", e))?;
    let contacts = mc.contacts().await;
    let neighbours = data
        .neighbours
        .iter()
        .map(|n| {
            let pubkey_hex = n
                .pubkey
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            let name = contacts.get(&pubkey_hex).map(|c| c.adv_name.clone());
            RepeaterNeighbour {
                pubkey_hex,
                secs_ago: n.secs_ago,
                snr: n.snr,
                name,
            }
        })
        .collect();
    Ok(neighbours)
}

#[tauri::command]
pub async fn repeater_telemetry(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<Vec<u8>, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .request_telemetry_with_timeout(pubkey.as_str(), REPEATER_TIMEOUT)
        .await
        .map_err(|e| format!("Erreur télémétrie : {}", e))
}

/// Demande la liste de contrôle d'accès du repeater
#[tauri::command]
pub async fn repeater_acl(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<Vec<AclEntry>, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let entries = mc
        .commands()
        .lock()
        .await
        .request_acl_with_timeout(pubkey.as_str(), REPEATER_TIMEOUT)
        .await
        .map_err(|e| format!("Erreur ACL : {}", e))?;
    let contacts = mc.contacts().await;
    Ok(entries
        .iter()
        .map(|e| {
            let pubkey_hex = e
                .prefix
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            let name = contacts
                .values()
                .find(|c| {
                    let ck = c
                        .public_key
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    ck.starts_with(&pubkey_hex)
                })
                .map(|c| c.adv_name.clone());
            AclEntry {
                pubkey_hex,
                permissions: e.permissions,
                name,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn send_advert(state: State<'_, AppState>, flood: bool) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .send_advert(flood)
        .await
        .map_err(|e| format!("Erreur advert : {}", e))?;
    tracing::info!("Advert envoyé (flood: {})", flood);
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// Commandes CLI texte (envoyées comme messages au repeater)
// ═══════════════════════════════════════════════════════════

/// Envoie une commande CLI texte au repeater et attend la réponse
///
/// Les commandes CLI sont envoyées comme des messages texte normaux.
/// Le repeater répond par un message texte contenant le résultat.
#[tauri::command]
pub async fn repeater_send_cli(
    state: State<'_, AppState>,
    pubkey: String,
    command: String,
) -> Result<String, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;

    tracing::info!(
        "CLI repeater [{}]: {}",
        &pubkey[..12.min(pubkey.len())],
        command
    );

    // Envoyer la commande comme message texte
    mc.commands()
        .lock()
        .await
        .send_msg(pubkey.as_str(), &command, None)
        .await
        .map_err(|e| format!("Erreur envoi CLI : {}", e))?;

    // Attendre la réponse (message entrant du repeater) avec timeout
    let response = mc
        .wait_for_event(
            Some(EventType::ContactMsgRecv),
            HashMap::new(),
            Duration::from_secs(15),
        )
        .await;

    match response {
        Some(event) => {
            if let EventPayload::ContactMessage(msg) = event.payload {
                tracing::info!("CLI réponse: {}", msg.text);
                Ok(msg.text)
            } else {
                Ok("(réponse reçue, format inattendu)".to_string())
            }
        }
        None => {
            // Pas de réponse dans le délai — la commande a peut-être fonctionné
            Ok("(envoyé, pas de réponse dans les 15s)".to_string())
        }
    }
}
