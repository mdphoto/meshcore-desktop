//! Opérations CRUD pour les canaux

use crate::StorageError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChannel {
    pub idx: u8,
    pub name: String,
    pub channel_type: String,
    pub psk: Vec<u8>,
    pub notifications_enabled: bool,
    pub unread_count: u32,
}

pub fn upsert_channel(conn: &Connection, channel: &StoredChannel) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO channels (idx, name, channel_type, psk, notifications_enabled, unread_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(idx) DO UPDATE SET
            name = excluded.name,
            channel_type = excluded.channel_type,
            psk = excluded.psk,
            notifications_enabled = excluded.notifications_enabled",
        params![
            channel.idx, channel.name, channel.channel_type,
            channel.psk, channel.notifications_enabled, channel.unread_count,
        ],
    )?;
    Ok(())
}

pub fn get_all_channels(conn: &Connection) -> Result<Vec<StoredChannel>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT idx, name, channel_type, psk, notifications_enabled, unread_count FROM channels ORDER BY idx"
    )?;
    let channels = stmt
        .query_map([], |row| {
            Ok(StoredChannel {
                idx: row.get(0)?,
                name: row.get(1)?,
                channel_type: row.get(2)?,
                psk: row.get(3)?,
                notifications_enabled: row.get(4)?,
                unread_count: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(channels)
}

pub fn increment_unread(conn: &Connection, channel_idx: u8) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE channels SET unread_count = unread_count + 1 WHERE idx = ?1",
        params![channel_idx],
    )?;
    Ok(())
}

pub fn reset_unread(conn: &Connection, channel_idx: u8) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE channels SET unread_count = 0 WHERE idx = ?1",
        params![channel_idx],
    )?;
    Ok(())
}

pub fn delete_channel(conn: &Connection, channel_idx: u8) -> Result<bool, StorageError> {
    let affected = conn.execute("DELETE FROM channels WHERE idx = ?1", params![channel_idx])?;
    Ok(affected > 0)
}
