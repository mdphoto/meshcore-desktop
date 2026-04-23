use crate::action::{Action, AsyncResult};
use meshcore_protocol::types::BatteryChemistry;
use meshcore_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_refresh_info(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcore_service::device::get_device_info(&state).await;
        let _ = action_tx.send(Action::Async(AsyncResult::DeviceInfoLoaded(result)));
    });
}

pub fn spawn_battery(
    state: Arc<AppState>,
    chemistry: BatteryChemistry,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::device::get_battery(&state, chemistry).await;
        let _ = action_tx.send(Action::Async(AsyncResult::BatteryLoaded(result)));
    });
}

pub fn spawn_sync_time(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcore_service::device::sync_time(&state).await;
        let msg = match result {
            Ok(()) => Ok("Heure synchronisée".to_string()),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}

pub fn spawn_set_tx_power(
    state: Arc<AppState>,
    power: u8,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::device::set_tx_power(&state, power).await;
        let msg = match result {
            Ok(()) => Ok(format!("TX power = {} dBm", power)),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        // Refresh info pour voir la nouvelle valeur
        spawn_refresh_info(state, action_tx);
    });
}

pub fn spawn_set_name(
    state: Arc<AppState>,
    name: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::device::set_device_name(&state, &name).await;
        let msg = match result {
            Ok(()) => Ok(format!("Nom du device : {}", name)),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        spawn_refresh_info(state, action_tx);
    });
}

pub fn spawn_reboot(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcore_service::device::reboot(&state).await;
        let msg = match result {
            Ok(()) => Ok("Redémarrage envoyé".to_string()),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}

pub fn spawn_send_advert(
    state: Arc<AppState>,
    flood: bool,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::send_advert(&state, flood).await;
        let msg = match result {
            Ok(()) => Ok(if flood {
                "Advert flood envoyé".to_string()
            } else {
                "Advert envoyé".to_string()
            }),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}
