use crate::action::{Action, AsyncResult};
use crate::state::chat::{ConversationId, PAGE_SIZE};
use meshcore_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// Charge depuis la DB les pubkeys de tous les contacts ayant des messages DM,
/// triées par dernier message. Utilisé pour afficher les conversations existantes
/// au démarrage, même sans clic préalable.
pub fn reload_dm_pubkeys(state: Arc<AppState>, action_tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
        match meshcore_service::messaging::get_dm_pubkeys(&state) {
            Ok(list) => {
                tracing::info!("tui: {} DM pubkeys chargés depuis DB", list.len());
                let _ = action_tx.send(Action::Async(AsyncResult::DmPubkeysLoaded(list)));
            }
            Err(e) => {
                tracing::warn!("tui: reload_dm_pubkeys: {}", e);
            }
        }
    });
}

pub fn load_messages(
    state: Arc<AppState>,
    conversation: ConversationId,
    offset: u32,
    prepend: bool,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        tracing::info!(
            "tui: load_messages conversation={:?} offset={} prepend={}",
            conversation,
            offset,
            prepend
        );
        let result = match &conversation {
            ConversationId::Dm(pk) => {
                meshcore_service::messaging::get_direct_messages(&state, pk, PAGE_SIZE, offset)
            }
            ConversationId::Channel(idx) => {
                meshcore_service::messaging::get_channel_messages(&state, *idx, PAGE_SIZE, offset)
            }
        };
        match result {
            Ok(mut list) => {
                tracing::info!(
                    "tui: load_messages got {} messages for {:?}",
                    list.len(),
                    conversation
                );
                // L'API retourne DESC (plus récent en tête), on inverse pour l'ordre chronologique
                list.reverse();
                let full = (list.len() as u32) < PAGE_SIZE;
                let _ = action_tx.send(Action::Async(AsyncResult::MessagesLoaded {
                    conversation,
                    messages: list,
                    prepend,
                    fully_loaded: full,
                }));
            }
            Err(e) => {
                tracing::error!("tui: load_messages error : {}", e);
                let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(e))));
            }
        }
    });
}

pub fn spawn_send_dm(
    state: Arc<AppState>,
    pubkey: String,
    text: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result =
            meshcore_service::messaging::send_direct_message(&state, &pubkey, &text).await;
        let msg = match result {
            Ok(_id) => Ok(String::new()),
            Err(e) => Err(format!("Envoi échoué : {}", e)),
        };
        if let Err(ref _e) = msg {
            let _ = action_tx.send(Action::Async(AsyncResult::Generic(msg.clone())));
        }
        // Recharger la conversation pour voir le message envoyé en local
        load_messages(
            state,
            ConversationId::Dm(pubkey),
            0,
            false,
            action_tx,
        );
    });
}

pub fn spawn_send_channel(
    state: Arc<AppState>,
    channel_idx: u8,
    text: String,
    action_tx: UnboundedSender<Action>,
) {
    tokio::spawn(async move {
        let result =
            meshcore_service::messaging::send_channel_message(&state, channel_idx, &text).await;
        if let Err(e) = result {
            let _ = action_tx.send(Action::Async(AsyncResult::Generic(Err(format!(
                "Envoi canal échoué : {}",
                e
            )))));
        }
        load_messages(
            state,
            ConversationId::Channel(channel_idx),
            0,
            false,
            action_tx,
        );
    });
}
