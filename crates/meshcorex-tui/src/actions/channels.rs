use crate::action::{Action, AsyncResult};
use meshcorex_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn reload(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcorex_service::channels::get_all_channels(&state);
        match result {
            Ok(list) => {
                let _ = action_tx.send(Action::Async(AsyncResult::ChannelsReloaded(list)));
            }
            Err(e) => {
                let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(e))));
            }
        }
    });
}

pub fn spawn_mark_read(
    state: Arc<AppState>,
    channel_idx: u8,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcorex_service::channels::mark_as_read(&state, channel_idx);
        if let Err(e) = result {
            let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(e))));
            return;
        }
        reload(state, action_tx);
    });
}

pub fn spawn_sync_to_device(
    state: Arc<AppState>,
    channel: meshcorex_storage::channels::StoredChannel,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let psk_array: [u8; 16] = match channel.psk.as_slice().try_into() {
            Ok(arr) => arr,
            Err(_) => {
                let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(
                    "PSK de taille incorrecte (attendu 16 octets)".to_string(),
                ))));
                return;
            }
        };
        let result = meshcorex_service::channels::sync_channel_to_device(
            &state,
            channel.idx,
            &channel.name,
            &psk_array,
        )
        .await;
        let msg = match result {
            Ok(()) => Ok(crate::util::i18n::t("toast.channel_synced").replace("{}", &channel.name)),
            Err(e) => Err(format!("Sync canal : {}", e)),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}

pub fn spawn_delete(
    state: Arc<AppState>,
    channel_idx: u8,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcorex_service::channels::delete_channel(&state, channel_idx).await;
        let msg = match result {
            Ok(()) => Ok(crate::util::i18n::t("toast.channel_deleted")),
            Err(e) => Err(format!("Suppression canal : {}", e)),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
        reload(state, action_tx);
    });
}
