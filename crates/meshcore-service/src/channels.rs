//! Service de gestion des canaux

use crate::state::AppState;
use meshcore_storage::channels as store;
use meshcore_storage::channels::StoredChannel;

pub fn get_all_channels(state: &AppState) -> Result<Vec<StoredChannel>, String> {
    state
        .db
        .with_conn(|conn| store::get_all_channels(conn))
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

pub fn delete_channel(state: &AppState, channel_idx: u8) -> Result<(), String> {
    state
        .db
        .with_conn(|conn| store::delete_channel(conn, channel_idx).map(|_| ()))
        .map_err(|e| e.to_string())
}
