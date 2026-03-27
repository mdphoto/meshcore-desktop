use meshcore_service::AppState;
use meshcore_storage::models::StoredCompanion;
use tauri::State;

#[tauri::command]
pub fn get_all_companions(state: State<'_, AppState>) -> Result<Vec<StoredCompanion>, String> {
    state
        .db
        .with_conn(meshcore_storage::companions::get_all_companions)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_companion(
    state: State<'_, AppState>,
    transport_type: String,
    name: String,
    address: String,
    pin: Option<String>,
) -> Result<(), String> {
    let companion = StoredCompanion {
        id: None,
        transport_type,
        name,
        address,
        pin,
        last_used: chrono::Utc::now().to_rfc3339(),
    };
    state
        .db
        .with_conn(|conn| meshcore_storage::companions::upsert_companion(conn, &companion))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_companion(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    state
        .db
        .with_conn(|conn| meshcore_storage::companions::delete_companion(conn, id))
        .map_err(|e| e.to_string())
}
