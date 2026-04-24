//! Service de gestion des canaux

use crate::state::AppState;
use meshcorex_storage::channels as store;
use meshcorex_storage::channels::StoredChannel;

pub fn get_all_channels(state: &AppState) -> Result<Vec<StoredChannel>, String> {
    state
        .db
        .with_conn(store::get_all_channels)
        .map_err(|e| e.to_string())
}

pub fn upsert_channel(state: &AppState, channel: &StoredChannel) -> Result<(), String> {
    state
        .db
        .with_conn(|conn| store::upsert_channel(conn, channel))
        .map_err(|e| e.to_string())
}

pub async fn sync_channel_to_device(
    state: &AppState,
    channel_idx: u8,
    name: &str,
    psk: &[u8; 16],
) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .set_channel(channel_idx, name, psk)
        .await
        .map_err(|e| e.to_string())
}

pub fn mark_as_read(state: &AppState, channel_idx: u8) -> Result<(), String> {
    state
        .db
        .with_conn(|conn| store::reset_unread(conn, channel_idx))
        .map_err(|e| e.to_string())
}

pub async fn delete_channel(state: &AppState, channel_idx: u8) -> Result<(), String> {
    // 1. Supprimer de la base de données locale
    state
        .db
        .with_conn(|conn| store::delete_channel(conn, channel_idx).map(|_| ()))
        .map_err(|e| e.to_string())?;

    // 2. Effacer le canal sur le device (nom vide + PSK vide)
    // Cela évite que le canal ne réapparaisse lors de la reconnexion
    let conn = state.connection.read().await;
    if let Some(mc) = conn.meshcore() {
        let empty_psk = [0u8; 16];
        let _ = mc
            .commands()
            .lock()
            .await
            .set_channel(channel_idx, "", &empty_psk)
            .await;
    }

    Ok(())
}
