//! Service de gestion du dispositif

use crate::events::AppEvent;
use crate::state::AppState;
use meshcore_protocol::types::BatteryChemistry;
use tracing::info;

pub async fn get_device_info(state: &AppState) -> Result<DeviceInfoSummary, String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let si = mc.self_info().await.ok_or("Pas d'info self reçue")?;

    Ok(DeviceInfoSummary {
        name: si.name.clone(),
        public_key: hex_encode(&si.public_key),
        adv_type: si.adv_type,
        tx_power: si.tx_power,
        max_tx_power: si.max_tx_power,
        radio_freq: si.radio_freq,
        radio_bw: si.radio_bw,
        sf: si.sf,
        cr: si.cr,
        lat: meshcore_rs::parsing::from_microdegrees(si.adv_lat),
        lon: meshcore_rs::parsing::from_microdegrees(si.adv_lon),
    })
}

pub async fn get_battery(
    state: &AppState,
    chemistry: BatteryChemistry,
) -> Result<(u16, u8), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let battery = mc
        .commands()
        .lock()
        .await
        .get_bat()
        .await
        .map_err(|e| e.to_string())?;
    let mv = battery.battery_mv;
    let percent = chemistry.percentage(mv);
    state.emit(AppEvent::BatteryUpdate {
        millivolts: mv,
        percent,
    });
    Ok((mv, percent))
}

pub async fn sync_time(state: &AppState) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as u32;
    mc.commands()
        .lock()
        .await
        .set_time(now)
        .await
        .map_err(|e| e.to_string())?;
    info!("Horloge synchronisée : {}", now);
    Ok(())
}

pub async fn set_tx_power(state: &AppState, power: u8) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .set_tx_power(power)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn set_device_name(state: &AppState, name: &str) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .set_name(name)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn reboot(state: &AppState) -> Result<(), String> {
    let conn = state.connection.read().await;
    let mc = conn.meshcore().ok_or("Non connecté")?;
    mc.commands()
        .lock()
        .await
        .reboot()
        .await
        .map_err(|e| e.to_string())?;
    state.emit(AppEvent::Disconnected);
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfoSummary {
    pub name: String,
    pub public_key: String,
    pub adv_type: u8,
    pub tx_power: u8,
    pub max_tx_power: u8,
    pub radio_freq: u32,
    pub radio_bw: u32,
    pub sf: u8,
    pub cr: u8,
    pub lat: f64,
    pub lon: f64,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
