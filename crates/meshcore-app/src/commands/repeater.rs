//! Thin wrappers Tauri pour l'administration des repeaters
//! La logique est dans meshcore_service::repeater

use meshcore_service::AppState;
use meshcore_service::repeater::{AclEntry, RepeaterNeighbour, RepeaterStatus};
use tauri::State;

#[tauri::command]
pub async fn repeater_login(
    state: State<'_, AppState>,
    pubkey: String,
    password: String,
) -> Result<String, String> {
    meshcore_service::repeater::login(&state, &pubkey, &password).await
}

#[tauri::command]
pub async fn repeater_logout(state: State<'_, AppState>, pubkey: String) -> Result<(), String> {
    meshcore_service::repeater::logout(&state, &pubkey).await
}

#[tauri::command]
pub async fn repeater_status(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<RepeaterStatus, String> {
    meshcore_service::repeater::status(&state, &pubkey).await
}

#[tauri::command]
pub async fn repeater_neighbours(
    state: State<'_, AppState>,
    pubkey: String,
    count: u16,
    offset: u16,
) -> Result<Vec<RepeaterNeighbour>, String> {
    meshcore_service::repeater::neighbours(&state, &pubkey, count, offset).await
}

#[tauri::command]
pub async fn repeater_telemetry(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<Vec<u8>, String> {
    meshcore_service::repeater::telemetry(&state, &pubkey).await
}

#[tauri::command]
pub async fn repeater_acl(
    state: State<'_, AppState>,
    pubkey: String,
) -> Result<Vec<AclEntry>, String> {
    meshcore_service::repeater::acl(&state, &pubkey).await
}

#[tauri::command]
pub async fn send_advert(state: State<'_, AppState>, flood: bool) -> Result<(), String> {
    meshcore_service::repeater::send_advert(&state, flood).await
}

#[tauri::command]
pub async fn repeater_send_cli(
    state: State<'_, AppState>,
    pubkey: String,
    command: String,
) -> Result<String, String> {
    meshcore_service::repeater::send_cli(&state, &pubkey, &command).await
}
