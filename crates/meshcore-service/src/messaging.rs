//! Service de messagerie : envoi, réception, retry, statut

use crate::state::AppState;
use crate::events::AppEvent;
use meshcore_storage::messages;
use meshcore_storage::models::StoredMessage;
use tracing::info;

/// Envoie un message direct à un contact
pub async fn send_direct_message(
    state: &AppState,
    recipient_pubkey: &str,
    text: &str,
) -> Result<String, String> {
    let msg = StoredMessage::new_outgoing(recipient_pubkey, text);
    let msg_id = msg.id.clone();

    state.db.with_conn(|conn| messages::insert_message(conn, &msg))
        .map_err(|e| e.to_string())?;

    let conn = state.connection.read().await;
    if let Some(mc) = conn.meshcore() {
        mc.commands().lock().await
            .send_msg(recipient_pubkey, text, None)
            .await
            .map_err(|e| e.to_string())?;

        state.db.with_conn(|c| messages::update_message_status(c, &msg_id, "sent"))
            .map_err(|e| e.to_string())?;

        state.emit(AppEvent::MessageSent { message_id: msg_id.clone() });
        info!("Message envoyé à {}", &recipient_pubkey[..12.min(recipient_pubkey.len())]);
    } else {
        return Err("Non connecté".to_string());
    }

    Ok(msg_id)
}

/// Envoie un message sur un canal
pub async fn send_channel_message(
    state: &AppState,
    channel_idx: u8,
    text: &str,
) -> Result<String, String> {
    let msg = StoredMessage::new_channel_outgoing(channel_idx, text);
    let msg_id = msg.id.clone();

    state.db.with_conn(|conn| messages::insert_message(conn, &msg))
        .map_err(|e| e.to_string())?;

    let conn = state.connection.read().await;
    if let Some(mc) = conn.meshcore() {
        mc.commands().lock().await
            .send_channel_msg(channel_idx, text, None)
            .await
            .map_err(|e| e.to_string())?;

        state.db.with_conn(|c| messages::update_message_status(c, &msg_id, "sent"))
            .map_err(|e| e.to_string())?;

        state.emit(AppEvent::MessageSent { message_id: msg_id.clone() });
    } else {
        return Err("Non connecté".to_string());
    }

    Ok(msg_id)
}

/// Récupère les messages d'une conversation directe
pub fn get_direct_messages(
    state: &AppState,
    contact_pubkey: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, String> {
    state.db.with_conn(|conn| messages::get_direct_messages(conn, contact_pubkey, limit, offset))
        .map_err(|e| e.to_string())
}

/// Récupère les messages d'un canal
pub fn get_channel_messages(
    state: &AppState,
    channel_idx: u8,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, String> {
    state.db.with_conn(|conn| messages::get_channel_messages(conn, channel_idx, limit, offset))
        .map_err(|e| e.to_string())
}
