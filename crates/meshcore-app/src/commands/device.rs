use meshcore_service::AppState;
use meshcore_service::device::DeviceInfoSummary;
use meshcore_protocol::types::BatteryChemistry;
use tauri::State;

#[tauri::command]
pub async fn get_device_info(state: State<'_, AppState>) -> Result<DeviceInfoSummary, String> {
    meshcore_service::device::get_device_info(&state).await
}

#[tauri::command]
pub async fn get_battery(state: State<'_, AppState>, chemistry: String) -> Result<(u16, u8), String> {
    let chem = match chemistry.as_str() {
        "lipo" => BatteryChemistry::LiPo,
        "lifepo4" => BatteryChemistry::LiFePO4,
        "nimh" => BatteryChemistry::NiMH,
        _ => BatteryChemistry::LiPo,
    };
    meshcore_service::device::get_battery(&state, chem).await
}

#[tauri::command]
pub async fn sync_time(state: State<'_, AppState>) -> Result<(), String> {
    meshcore_service::device::sync_time(&state).await
}

#[tauri::command]
pub async fn set_device_name(state: State<'_, AppState>, name: String) -> Result<(), String> {
    meshcore_service::device::set_device_name(&state, &name).await
}

#[tauri::command]
pub async fn reboot(state: State<'_, AppState>) -> Result<(), String> {
    meshcore_service::device::reboot(&state).await
}

#[tauri::command]
pub async fn set_tx_power(state: State<'_, AppState>, power: u8) -> Result<(), String> {
    meshcore_service::device::set_tx_power(&state, power).await
}

#[tauri::command]
pub fn scan_serial_ports() -> Result<Vec<String>, String> {
    let ports = tokio_serial::available_ports().map_err(|e: tokio_serial::Error| e.to_string())?;
    Ok(ports.into_iter().map(|p| p.port_name).collect())
}
