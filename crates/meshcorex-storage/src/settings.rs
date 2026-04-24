//! Stockage clé-valeur pour les paramètres de l'application

use crate::StorageError;
use rusqlite::{Connection, params};

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, StorageError> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
    Ok(rows.next().transpose()?)
}

pub fn delete(conn: &Connection, key: &str) -> Result<bool, StorageError> {
    let affected = conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(affected > 0)
}

pub fn get_all(conn: &Connection) -> Result<Vec<(String, String)>, StorageError> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
    let settings = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(settings)
}
