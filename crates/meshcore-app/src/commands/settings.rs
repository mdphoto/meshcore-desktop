use meshcore_service::AppState;
use tauri::State;

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    state
        .db
        .with_conn(|conn| meshcore_storage::settings::get(conn, &key))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    state
        .db
        .with_conn(|conn| meshcore_storage::settings::set(conn, &key, &value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_settings(state: State<'_, AppState>) -> Result<Vec<(String, String)>, String> {
    state
        .db
        .with_conn(|conn| meshcore_storage::settings::get_all(conn))
        .map_err(|e| e.to_string())
}
