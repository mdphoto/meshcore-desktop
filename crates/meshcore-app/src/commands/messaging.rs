use meshcore_service::AppState;
use meshcore_storage::models::StoredMessage;
use tauri::State;

#[tauri::command]
pub async fn send_direct_message(
    state: State<'_, AppState>,
    recipient_pubkey: String,
    text: String,
) -> Result<String, String> {
    meshcore_service::messaging::send_direct_message(&state, &recipient_pubkey, &text).await
}

#[tauri::command]
pub async fn send_channel_message(
    state: State<'_, AppState>,
    channel_idx: u8,
    text: String,
) -> Result<String, String> {
    meshcore_service::messaging::send_channel_message(&state, channel_idx, &text).await
}

#[tauri::command]
pub fn get_direct_messages(
    state: State<'_, AppState>,
    contact_pubkey: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, String> {
    meshcore_service::messaging::get_direct_messages(&state, &contact_pubkey, limit, offset)
}

#[tauri::command]
pub fn get_channel_messages(
    state: State<'_, AppState>,
    channel_idx: u8,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, String> {
    meshcore_service::messaging::get_channel_messages(&state, channel_idx, limit, offset)
}

#[tauri::command]
pub fn delete_conversation(
    state: State<'_, AppState>,
    contact_pubkey: String,
) -> Result<u64, String> {
    state.db.with_conn(|conn| {
        meshcore_storage::messages::delete_conversation(conn, &contact_pubkey)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_messages(
    state: State<'_, AppState>,
    query: String,
    limit: u32,
) -> Result<Vec<StoredMessage>, String> {
    state.db.with_conn(|conn| {
        meshcore_storage::messages::search_messages(conn, &query, limit)
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_login(
    state: State<'_, AppState>,
    room_pubkey: String,
    password: String,
) -> Result<String, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands().lock().await
        .send_login(room_pubkey.clone(), &password)
        .await
        .map_err(|e| format!("Login échoué : {}", e))?;
    tracing::info!("Login Room Server {}", &room_pubkey[..12.min(room_pubkey.len())]);
    Ok("Login envoyé".to_string())
}

#[tauri::command]
pub fn get_dm_contact_pubkeys(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state.db.with_conn(|conn| {
        meshcore_storage::messages::get_dm_contact_pubkeys(conn)
    }).map_err(|e| e.to_string())
}
