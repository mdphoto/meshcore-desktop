//! Opérations CRUD pour les messages

use crate::StorageError;
use crate::models::StoredMessage;
use rusqlite::{Connection, params};

/// Insère un nouveau message
pub fn insert_message(conn: &Connection, msg: &StoredMessage) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO messages (id, direction, sender_pubkey, sender_name, recipient_pubkey, channel_idx, text, timestamp, status, snr, rssi, path_len, attempt, reply_to, reaction)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            msg.id, msg.direction, msg.sender_pubkey, msg.sender_name,
            msg.recipient_pubkey, msg.channel_idx, msg.text, msg.timestamp,
            msg.status, msg.snr, msg.rssi, msg.path_len, msg.attempt,
            msg.reply_to, msg.reaction,
        ],
    )?;
    Ok(())
}

/// Met à jour le statut d'un message
pub fn update_message_status(
    conn: &Connection,
    id: &str,
    status: &str,
) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

/// Récupère les messages d'une conversation directe (paginé)
pub fn get_direct_messages(
    conn: &Connection,
    contact_pubkey: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, direction, sender_pubkey, sender_name, recipient_pubkey, channel_idx,
                text, timestamp, status, snr, rssi, path_len, attempt, reply_to, reaction
         FROM messages
         WHERE (sender_pubkey = ?1 OR recipient_pubkey = ?1) AND channel_idx IS NULL
         ORDER BY timestamp DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let messages = stmt
        .query_map(params![contact_pubkey, limit, offset], map_message_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

/// Récupère les messages d'un canal (paginé)
pub fn get_channel_messages(
    conn: &Connection,
    channel_idx: u8,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, direction, sender_pubkey, sender_name, recipient_pubkey, channel_idx,
                text, timestamp, status, snr, rssi, path_len, attempt, reply_to, reaction
         FROM messages
         WHERE channel_idx = ?1
         ORDER BY timestamp DESC
         LIMIT ?2 OFFSET ?3",
    )?;

    let messages = stmt
        .query_map(params![channel_idx, limit, offset], map_message_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

/// Retourne les pubkeys des contacts avec qui on a des messages DM
pub fn get_dm_contact_pubkeys(conn: &Connection) -> Result<Vec<String>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT COALESCE(sender_pubkey, recipient_pubkey) as pk
         FROM messages
         WHERE channel_idx IS NULL AND pk IS NOT NULL
         ORDER BY (SELECT MAX(timestamp) FROM messages m2
                   WHERE (m2.sender_pubkey = pk OR m2.recipient_pubkey = pk) AND m2.channel_idx IS NULL) DESC"
    )?;
    let keys = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(keys)
}

/// Supprime les messages d'une conversation
pub fn delete_conversation(conn: &Connection, contact_pubkey: &str) -> Result<u64, StorageError> {
    let affected = conn.execute(
        "DELETE FROM messages WHERE sender_pubkey = ?1 OR recipient_pubkey = ?1",
        params![contact_pubkey],
    )?;
    Ok(affected as u64)
}

/// Recherche des messages par texte
pub fn search_messages(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> Result<Vec<StoredMessage>, StorageError> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, direction, sender_pubkey, sender_name, recipient_pubkey, channel_idx,
                text, timestamp, status, snr, rssi, path_len, attempt, reply_to, reaction
         FROM messages
         WHERE text LIKE ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    )?;

    let messages = stmt
        .query_map(params![pattern, limit], map_message_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

fn map_message_row(row: &rusqlite::Row) -> rusqlite::Result<StoredMessage> {
    Ok(StoredMessage {
        id: row.get(0)?,
        direction: row.get(1)?,
        sender_pubkey: row.get(2)?,
        sender_name: row.get(3)?,
        recipient_pubkey: row.get(4)?,
        channel_idx: row.get(5)?,
        text: row.get(6)?,
        timestamp: row.get(7)?,
        status: row.get(8)?,
        snr: row.get(9)?,
        rssi: row.get(10)?,
        path_len: row.get(11)?,
        attempt: row.get(12)?,
        reply_to: row.get(13)?,
        reaction: row.get(14)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use crate::models::StoredMessage;

    #[test]
    fn test_insert_and_get_direct() {
        let db = Database::in_memory().unwrap();
        let msg = StoredMessage::new_outgoing("pubkey123", "Bonjour mesh !");

        db.with_conn(|conn| insert_message(conn, &msg)).unwrap();

        let messages = db
            .with_conn(|conn| get_direct_messages(conn, "pubkey123", 50, 0))
            .unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text, "Bonjour mesh !");
    }

    #[test]
    fn test_update_status() {
        let db = Database::in_memory().unwrap();
        let msg = StoredMessage::new_outgoing("pubkey123", "test");

        db.with_conn(|conn| insert_message(conn, &msg)).unwrap();
        db.with_conn(|conn| update_message_status(conn, &msg.id, "delivered"))
            .unwrap();

        let messages = db
            .with_conn(|conn| get_direct_messages(conn, "pubkey123", 50, 0))
            .unwrap();
        assert_eq!(messages[0].status, "delivered");
    }
}
