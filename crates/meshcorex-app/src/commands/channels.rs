use meshcorex_service::AppState;
use meshcorex_storage::channels::StoredChannel;
use tauri::State;

#[tauri::command]
pub fn get_all_channels(state: State<'_, AppState>) -> Result<Vec<StoredChannel>, String> {
    meshcorex_service::channels::get_all_channels(&state)
}

#[tauri::command]
pub fn mark_as_read(state: State<'_, AppState>, channel_idx: u8) -> Result<(), String> {
    meshcorex_service::channels::mark_as_read(&state, channel_idx)
}

#[tauri::command]
pub async fn sync_channel_to_device(
    state: State<'_, AppState>,
    channel_idx: u8,
    name: String,
    psk: Vec<u8>,
) -> Result<(), String> {
    if psk.len() != 16 {
        return Err("Le PSK doit faire exactement 16 octets".to_string());
    }
    let psk_array: [u8; 16] = psk.try_into().unwrap();
    meshcorex_service::channels::sync_channel_to_device(&state, channel_idx, &name, &psk_array).await
}

#[tauri::command]
pub fn upsert_channel(
    state: State<'_, AppState>,
    idx: u8,
    name: String,
    channel_type: String,
    psk: Vec<u8>,
    notifications_enabled: bool,
) -> Result<(), String> {
    let channel = StoredChannel {
        idx,
        name,
        channel_type,
        psk,
        notifications_enabled,
        unread_count: 0,
    };
    meshcorex_service::channels::upsert_channel(&state, &channel)
}

#[tauri::command]
pub async fn delete_channel(state: State<'_, AppState>, channel_idx: u8) -> Result<(), String> {
    meshcorex_service::channels::delete_channel(&state, channel_idx).await
}
