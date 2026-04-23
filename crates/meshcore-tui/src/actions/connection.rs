use crate::action::{Action, AsyncResult, BleDevice, ConnectionInfo};
use meshcore_service::AppState;
use meshcore_transport::manager::{ConnectionTarget, ConnectionManager};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_ble_scan(action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = scan_ble().await;
        let action = match result {
            Ok(devices) => Action::Async(AsyncResult::BleScanDone(devices)),
            Err(e) => Action::Async(AsyncResult::BleScanFailed(e)),
        };
        let _ = action_tx.send(action);
    });
}

async fn scan_ble() -> Result<Vec<BleDevice>, String> {
    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
    use btleplug::platform::Manager;

    let duration = Duration::from_secs(5);
    let manager = Manager::new()
        .await
        .map_err(|e| format!("BLE manager: {}", e))?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|e| format!("Pas d'adaptateur BLE : {}", e))?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or_else(|| "Aucun adaptateur Bluetooth".to_string())?;

    #[cfg(target_os = "linux")]
    {
        use uuid::Uuid;
        let svc = Uuid::from_u128(0x6e400001_b5a3_f393_e0a9_e50e24dcca9e);
        adapter
            .start_scan(ScanFilter { services: vec![svc] })
            .await
            .map_err(|e| format!("start_scan: {}", e))?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| format!("start_scan: {}", e))?;
    }

    tokio::time::sleep(duration).await;
    adapter
        .stop_scan()
        .await
        .map_err(|e| format!("stop_scan: {}", e))?;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(|e| format!("peripherals: {}", e))?;

    let mut devices = Vec::new();
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            let name = props.local_name.unwrap_or_default();
            if name.starts_with("MeshCore-") || name.starts_with("Whisper-") {
                devices.push(BleDevice {
                    name,
                    address: props.address.to_string(),
                    rssi: props.rssi,
                });
            }
        }
    }
    devices.sort_by_key(|d| std::cmp::Reverse(d.rssi.unwrap_or(-128)));
    Ok(devices)
}

pub fn spawn_serial_scan(action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = tokio_serial::available_ports()
            .map(|list| list.into_iter().map(|p| p.port_name).collect::<Vec<_>>())
            .map_err(|e| format!("Scan série : {}", e));
        let action = match result {
            Ok(ports) => Action::Async(AsyncResult::SerialScanDone(ports)),
            Err(e) => Action::Async(AsyncResult::SerialScanFailed(e)),
        };
        let _ = action_tx.send(action);
    });
}

pub fn spawn_connect(
    state: Arc<AppState>,
    target: ConnectionTarget,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let msg = match meshcore_service::connection::connect(&state, target).await {
            Ok(()) => Ok("Connecté".to_string()),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        refresh_list(state, action_tx);
    });
}

pub fn spawn_disconnect_primary(
    state: Arc<AppState>,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let msg = match meshcore_service::connection::disconnect(&state).await {
            Ok(()) => Ok("Déconnecté".to_string()),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        refresh_list(state, action_tx);
    });
}

pub fn spawn_disconnect_by_id(
    state: Arc<AppState>,
    id: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let mut conn = state.connection.write().await;
        let msg = match conn.disconnect_by_id(&id).await {
            Ok(()) => Ok("Déconnecté".to_string()),
            Err(e) => Err(e.to_string()),
        };
        drop(conn);
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        refresh_list(state, action_tx);
    });
}

pub fn spawn_set_primary(
    state: Arc<AppState>,
    id: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let mut conn = state.connection.write().await;
        let _ = conn.set_primary(&id);
        drop(conn);
        refresh_list(state, action_tx);
    });
}

pub fn refresh_list(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let conn = state.connection.read().await;
        let list = list_connections(&conn);
        drop(conn);
        let _ = action_tx.send(Action::Async(AsyncResult::ConnectionsListed(list)));
    });
}

fn list_connections(conn: &ConnectionManager) -> Vec<ConnectionInfo> {
    conn.list_connections()
        .into_iter()
        .map(|(id, target, is_primary)| ConnectionInfo {
            id,
            label: match target {
                ConnectionTarget::Ble { name_or_addr } => format!("BLE : {}", name_or_addr),
                ConnectionTarget::Serial { port, baud_rate } => {
                    format!("Série : {} @ {}", port, baud_rate)
                }
                ConnectionTarget::Tcp { host, port } => format!("TCP : {}:{}", host, port),
            },
            is_primary,
        })
        .collect()
}

pub fn parse_tcp(input: &str) -> Option<ConnectionTarget> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (host, port) = match trimmed.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().ok()?),
        None => (trimmed.to_string(), 4403),
    };
    Some(ConnectionTarget::Tcp { host, port })
}
