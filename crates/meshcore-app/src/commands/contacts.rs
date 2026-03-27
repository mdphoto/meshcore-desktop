use meshcore_service::AppState;
use meshcore_storage::models::StoredContact;
use tauri::State;

#[tauri::command]
pub async fn sync_contacts(state: State<'_, AppState>) -> Result<usize, String> {
    meshcore_service::contacts::sync_contacts(&state).await
}

#[tauri::command]
pub fn get_all_contacts(state: State<'_, AppState>) -> Result<Vec<StoredContact>, String> {
    meshcore_service::contacts::get_all_contacts(&state)
}

#[tauri::command]
pub fn toggle_favorite(
    state: State<'_, AppState>,
    public_key: String,
    favorite: bool,
) -> Result<(), String> {
    meshcore_service::contacts::toggle_favorite(&state, &public_key, favorite)
}

#[tauri::command]
pub async fn delete_contact(state: State<'_, AppState>, public_key: String) -> Result<(), String> {
    meshcore_service::contacts::delete_contact(&state, &public_key).await
}
