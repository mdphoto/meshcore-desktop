//! Opérations CRUD pour les companions (dispositifs connus)

use crate::models::StoredCompanion;
use crate::StorageError;
use rusqlite::{params, Connection};

pub fn upsert_companion(conn: &Connection, companion: &StoredCompanion) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO companions (transport_type, name, address, pin, last_used)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(transport_type, address) DO UPDATE SET
            name = excluded.name,
            pin = excluded.pin,
            last_used = excluded.last_used",
        params![
            companion.transport_type, companion.name, companion.address,
            companion.pin, companion.last_used,
        ],
    )?;
    Ok(())
}

pub fn get_all_companions(conn: &Connection) -> Result<Vec<StoredCompanion>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, transport_type, name, address, pin, last_used FROM companions ORDER BY last_used DESC"
    )?;
    let companions = stmt
        .query_map([], |row| {
            Ok(StoredCompanion {
                id: row.get(0)?,
                transport_type: row.get(1)?,
                name: row.get(2)?,
                address: row.get(3)?,
                pin: row.get(4)?,
                last_used: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(companions)
}

pub fn delete_companion(conn: &Connection, id: i64) -> Result<bool, StorageError> {
    let affected = conn.execute("DELETE FROM companions WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}
