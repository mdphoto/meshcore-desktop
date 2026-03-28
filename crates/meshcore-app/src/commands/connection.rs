use meshcore_service::AppState;
use meshcore_transport::manager::ConnectionTarget;
use serde::Serialize;
use tauri::State;

/// Périphérique BLE découvert
#[derive(Debug, Clone, Serialize)]
pub struct BleDevice {
    pub name: String,
    pub address: String,
    pub rssi: Option<i16>,
}

/// Scanne les périphériques BLE visibles (filtre MeshCore-* et Whisper-*)
///
/// meshcore-rs `ble_connect()` attend le **nom** du périphérique (ex: "MeshCore-AB12")
/// ou l'adresse MAC au format "XX:XX:XX:XX:XX:XX".
/// On retourne les deux pour que le frontend puisse connecter par nom.
#[tauri::command]
pub async fn scan_ble_devices(duration_secs: Option<u64>) -> Result<Vec<BleDevice>, String> {
    use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
    use btleplug::platform::Manager;
    use std::time::Duration;

    let duration = Duration::from_secs(duration_secs.unwrap_or(5));

    let manager = Manager::new()
        .await
        .map_err(|e| format!("Erreur BLE manager : {}", e))?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|e| format!("Pas d'adaptateur BLE : {}", e))?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or("Aucun adaptateur Bluetooth trouvé")?;

    // Scanner SANS filtre UUID (le filtrage par service UUID ne fonctionne pas
    // sur Windows WinRT). On filtre par nom après le scan.
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Erreur scan : {}", e))?;

    tokio::time::sleep(duration).await;
    adapter
        .stop_scan()
        .await
        .map_err(|e| format!("Erreur stop scan : {}", e))?;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(|e| format!("Erreur listing : {}", e))?;

    let mut devices = Vec::new();
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            let name = props.local_name.unwrap_or_default();
            // Filtrer par nom : MeshCore-* ou Whisper-* (préfixes standard MeshCore)
            if name.starts_with("MeshCore-") || name.starts_with("Whisper-") {
                let address = props.address.to_string();
                let rssi = props.rssi;
                devices.push(BleDevice {
                    name,
                    address,
                    rssi,
                });
            }
        }
    }

    // Trier par RSSI décroissant (meilleur signal en premier)
    devices.sort_by(|a, b| b.rssi.unwrap_or(-128).cmp(&a.rssi.unwrap_or(-128)));
    Ok(devices)
}

#[tauri::command]
pub async fn connect_serial(
    state: State<'_, AppState>,
    port: String,
    baud_rate: u32,
) -> Result<(), String> {
    meshcore_service::connection::connect(&state, ConnectionTarget::Serial { port, baud_rate })
        .await
}

#[tauri::command]
pub async fn connect_tcp(
    state: State<'_, AppState>,
    host: String,
    port: u16,
) -> Result<(), String> {
    meshcore_service::connection::connect(&state, ConnectionTarget::Tcp { host, port }).await
}

#[tauri::command]
pub async fn connect_ble(state: State<'_, AppState>, name_or_addr: String) -> Result<(), String> {
    meshcore_service::connection::connect(&state, ConnectionTarget::Ble { name_or_addr }).await
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    meshcore_service::connection::disconnect(&state).await
}

#[tauri::command]
pub async fn is_connected(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(meshcore_service::connection::is_connected(&state).await)
}

/// Appaire un périphérique BLE avec PIN
///
/// Sur Windows/macOS : l'OS gère l'appairage automatiquement via btleplug.
/// Sur Linux : utilise bluetoothctl en fallback.
#[tauri::command]
pub async fn pair_ble_device(address: String, pin: String) -> Result<String, String> {
    tracing::info!("Pairing BLE device {} avec PIN {}...", address, pin);

    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;
        let script = format!(
            r#"
{{
  echo "agent off"
  sleep 0.3
  echo "agent on"
  sleep 0.3
  echo "default-agent"
  sleep 0.3
  echo "scan on"
  sleep 5
  echo "scan off"
  sleep 0.5
  echo "pair {addr}"
  sleep 8
  echo "{pin}"
  sleep 4
  echo "trust {addr}"
  sleep 1
  echo "quit"
}} | bluetoothctl 2>&1
"#,
            addr = address,
            pin = pin
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(&script)
            .output()
            .await
            .map_err(|e| format!("Impossible de lancer bluetoothctl : {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let clean = stdout
            .replace("\x1b[0m", "")
            .replace("\x1b[0;94m", "")
            .replace("\x1b[0;92m", "")
            .replace("\x1b[1;39m", "")
            .replace("\x1b[K;9", "");

        tracing::info!("bluetoothctl output:\n{}", clean);

        if clean.contains("Pairing successful") {
            Ok(format!("Appairage réussi pour {}", address))
        } else if clean.contains("Already Paired") || clean.contains("already paired") {
            Ok(format!("Déjà appairé : {}", address))
        } else if clean.contains("Failed to pair") {
            Err("Échec de l'appairage — vérifiez le PIN".to_string())
        } else if clean.contains("trust succeeded") {
            Ok(format!("Trust OK pour {}", address))
        } else {
            Ok("Appairage tenté — vérifiez les logs backend".to_string())
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Sur Windows/macOS, l'appairage BLE est géré par l'OS.
        // btleplug déclenche l'appairage automatiquement lors de la connexion.
        // Le PIN est demandé par une popup système.
        let _ = pin; // Le PIN est géré par l'OS
        Ok(format!(
            "Sur cette plateforme, l'appairage est géré par le système. \
             Connectez-vous directement à {} — l'OS demandera le PIN si nécessaire.",
            address
        ))
    }
}

/// Vérifie si un device BLE est appairé
#[tauri::command]
pub async fn is_ble_paired(address: String) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        use tokio::process::Command;
        let output = Command::new("bluetoothctl")
            .args(["info", &address])
            .output()
            .await
            .map_err(|e| format!("bluetoothctl error: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("Paired: yes"))
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Sur Windows/macOS, pas de moyen simple de vérifier via btleplug.
        // On retourne true pour ne pas bloquer le flux de connexion.
        let _ = address;
        Ok(true)
    }
}

/// Déconnecte une connexion spécifique par son ID
#[tauri::command]
pub async fn disconnect_by_id(
    state: State<'_, AppState>,
    connection_id: String,
) -> Result<(), String> {
    let mut conn = state.connection.write().await;
    conn.disconnect_by_id(&connection_id)
        .await
        .map_err(|e| e.to_string())
}

/// Sélectionne une connexion comme primaire
#[tauri::command]
pub async fn set_primary_connection(
    state: State<'_, AppState>,
    connection_id: String,
) -> Result<bool, String> {
    let mut conn = state.connection.write().await;
    Ok(conn.set_primary(&connection_id))
}

/// Liste toutes les connexions actives
#[tauri::command]
pub async fn list_connections(state: State<'_, AppState>) -> Result<Vec<ConnectionInfo>, String> {
    let conn = state.connection.read().await;
    Ok(conn
        .list_connections()
        .iter()
        .map(|(id, target, is_primary)| ConnectionInfo {
            id: id.clone(),
            label: match target {
                ConnectionTarget::Ble { name_or_addr } => format!("BLE: {}", name_or_addr),
                ConnectionTarget::Serial { port, baud_rate } => {
                    format!("Serial: {} @ {}", port, baud_rate)
                }
                ConnectionTarget::Tcp { host, port } => format!("TCP: {}:{}", host, port),
            },
            is_primary: *is_primary,
        })
        .collect())
}

/// Info d'une connexion active (sérialisable pour le frontend)
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub label: String,
    pub is_primary: bool,
}
