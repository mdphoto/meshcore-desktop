use crate::action::{Action, AsyncResult};
use meshcorex_protocol::types::BatteryChemistry;
use meshcorex_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_refresh_info(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::device::get_device_info(&state).await;
        let _ = action_tx.send(Action::Async(AsyncResult::DeviceInfoLoaded(result)));
    });
}

pub fn spawn_battery(
    state: Arc<AppState>,
    chemistry: BatteryChemistry,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcorex_service::device::get_battery(&state, chemistry).await;
        let _ = action_tx.send(Action::Async(AsyncResult::BatteryLoaded(result)));
    });
}

pub fn spawn_sync_time(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::device::sync_time(&state).await;
        let msg = match result {
            Ok(()) => Ok(crate::util::i18n::t("toast.time_synced")),
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
        let result = meshcorex_service::device::set_tx_power(&state, power).await;
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
        let result = meshcorex_service::device::set_device_name(&state, &name).await;
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
        let result = meshcorex_service::device::reboot(&state).await;
        let msg = match result {
            Ok(()) => Ok(crate::util::i18n::t("toast.reboot_sent")),
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
        let result = meshcorex_service::repeater::send_advert(&state, flood).await;
        let msg = match result {
            Ok(()) => Ok(if flood {
                crate::util::i18n::t("toast.advert_flood_sent")
            } else {
                crate::util::i18n::t("toast.advert_sent")
            }),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}
