use crate::action::{Action, AsyncResult};
use meshcore_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_login(
    state: Arc<AppState>,
    pubkey: String,
    password: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::login(&state, &pubkey, &password).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RepeaterLoginResult(result)));
    });
}

/// Variante spécifique pour le login à une room server : renvoie le résultat
/// avec la pubkey ciblée pour que l'UI puisse updater `rooms_logged_in`.
pub fn spawn_room_login(
    state: Arc<AppState>,
    pubkey: String,
    password: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::login(&state, &pubkey, &password).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RoomLoginResult {
            pubkey,
            result,
        }));
    });
}

pub fn spawn_logout(
    state: Arc<AppState>,
    pubkey: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::logout(&state, &pubkey).await;
        let msg = match result {
            Ok(()) => Ok("Déconnecté du repeater".to_string()),
            Err(e) => Err(e),
        };
        let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg)));
    });
}

pub fn spawn_status(
    state: Arc<AppState>,
    pubkey: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::status(&state, &pubkey).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RepeaterStatusLoaded(result)));
    });
}

pub fn spawn_neighbours(
    state: Arc<AppState>,
    pubkey: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::neighbours(&state, &pubkey, 32, 0).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RepeaterNeighboursLoaded(result)));
    });
}

pub fn spawn_acl(state: Arc<AppState>, pubkey: String, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::acl(&state, &pubkey).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RepeaterAclLoaded(result)));
    });
}

pub fn spawn_cli(
    state: Arc<AppState>,
    pubkey: String,
    command: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result = meshcore_service::repeater::send_cli(&state, &pubkey, &command).await;
        let _ = action_tx.send(Action::Async(AsyncResult::RepeaterCliResult {
            command,
            result,
        }));
    });
}
