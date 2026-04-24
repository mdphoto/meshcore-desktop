//! Pairing BLE plateforme-spécifique.
//!
//! - **Linux** : appel D-Bus direct vers BlueZ via `bluez-async`. Le PIN (si demandé
//!   par le device) est géré par **l'agent Bluetooth système** — typiquement un
//!   popup de l'environnement de bureau (GNOME/KDE/…) ou `bluetoothctl` déjà
//!   démarré en mode `agent on`. Notre process ne peut pas fournir le PIN
//!   programmatiquement sans enregistrer un agent D-Bus complet.
//! - **macOS / Windows** : le pairing se fait exclusivement via les Réglages
//!   Bluetooth système — l'OS affiche son propre prompt à la première connexion.
//!
//! Workflow utilisateur :
//! 1. Dans l'UI, touche `P` sur un device BLE → lance `pair_ble(addr)`
//! 2. Si le device demande un PIN : popup de l'OS (hors de notre contrôle)
//! 3. Le résultat revient via le `Result` : Paired / AlreadyPaired / Err

use std::time::Duration;

/// Résultat du pairing
#[derive(Debug)]
pub enum PairResult {
    /// Pairing réussi (nouveau)
    Paired,
    /// Déjà pairé, pas d'action
    AlreadyPaired,
}

/// Tente de pair un dispositif BLE par son adresse MAC.
///
/// Le PIN éventuel est demandé par l'agent Bluetooth système (popup OS).
/// Cette fonction ne fournit PAS le PIN elle-même : enregistrer un agent
/// D-Bus custom nécessite une infrastructure complète qui dépasse le cadre
/// d'un client mesh.
pub async fn pair_ble(addr: &str) -> Result<PairResult, String> {
    #[cfg(target_os = "linux")]
    {
        pair_ble_linux(addr).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = addr;
        Err(
            "Pairing programmatique non supporté sur cette plateforme. \
             Utilisez les Réglages Bluetooth système, puis reconnectez-vous."
                .to_string(),
        )
    }
}

/// Indique si un dispositif BLE est déjà pairé côté OS.
pub async fn is_ble_paired(addr: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        is_ble_paired_linux(addr).await.unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = addr;
        false
    }
}

#[cfg(target_os = "linux")]
async fn pair_ble_linux(addr: &str) -> Result<PairResult, String> {
    use bluez_async::BluetoothSession;

    let (_, session) = BluetoothSession::new()
        .await
        .map_err(|e| format!("D-Bus session : {}", e))?;

    // Trouve le device par adresse MAC (BlueZ parse "AA:BB:CC:DD:EE:FF")
    let mac = parse_mac(addr)?;

    // Démarre un discovery bref pour que BlueZ connaisse le device si on l'a vu via
    // btleplug mais pas encore propagé côté BlueZ-D-Bus.
    let _ = session.start_discovery().await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let devices = session
        .get_devices()
        .await
        .map_err(|e| format!("list_devices : {}", e))?;

    let _ = session.stop_discovery().await;

    let device = devices
        .into_iter()
        .find(|d| d.mac_address == mac)
        .ok_or_else(|| format!("Device {} introuvable côté BlueZ (scanner d'abord).", addr))?;

    // Check paired déjà
    if device.paired {
        return Ok(PairResult::AlreadyPaired);
    }

    // Lance le pair (timeout 30s). Si l'agent système prompt pour un PIN,
    // l'utilisateur devra répondre via le popup OS pendant ce délai.
    session
        .pair_with_timeout(&device.id, Duration::from_secs(30))
        .await
        .map_err(|e| format!("pair : {}", e))?;

    // Double-check post-op : BlueZ peut dire OK sans avoir vraiment pairé
    // si un timeout a lieu côté agent.
    if is_ble_paired_linux(addr).await.unwrap_or(false) {
        Ok(PairResult::Paired)
    } else {
        Err(
            "Pairing terminé sans succès. Si un popup PIN est apparu, \
             répondez-y et réessayez ; sinon vérifiez que le device est en pairing mode."
                .to_string(),
        )
    }
}

#[cfg(target_os = "linux")]
async fn is_ble_paired_linux(addr: &str) -> Result<bool, String> {
    use bluez_async::BluetoothSession;

    let (_, session) = BluetoothSession::new()
        .await
        .map_err(|e| format!("D-Bus session : {}", e))?;
    let mac = parse_mac(addr)?;
    let devices = session
        .get_devices()
        .await
        .map_err(|e| format!("list_devices : {}", e))?;
    Ok(devices
        .into_iter()
        .find(|d| d.mac_address == mac)
        .map(|d| d.paired)
        .unwrap_or(false))
}

#[cfg(target_os = "linux")]
fn parse_mac(addr: &str) -> Result<bluez_async::MacAddress, String> {
    addr.parse()
        .map_err(|e| format!("MAC invalide « {} » : {}", addr, e))
}
