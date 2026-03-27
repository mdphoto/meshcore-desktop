//! Opérations CRUD pour les contacts

use crate::StorageError;
use crate::models::StoredContact;
use rusqlite::{Connection, params};

/// Insère ou met à jour un contact
pub fn upsert_contact(conn: &Connection, contact: &StoredContact) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO contacts (public_key, name, node_type, flags, path, path_len, lat, lon, last_seen, is_favorite, group_name, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
         ON CONFLICT(public_key) DO UPDATE SET
            name = excluded.name,
            node_type = excluded.node_type,
            flags = excluded.flags,
            path = excluded.path,
            path_len = excluded.path_len,
            lat = excluded.lat,
            lon = excluded.lon,
            last_seen = excluded.last_seen,
            is_favorite = excluded.is_favorite,
            group_name = excluded.group_name,
            updated_at = datetime('now')",
        params![
            contact.public_key,
            contact.name,
            contact.node_type,
            contact.flags,
            contact.path,
            contact.path_len,
            contact.lat,
            contact.lon,
            contact.last_seen,
            contact.is_favorite,
            contact.group_name,
        ],
    )?;
    Ok(())
}

/// Récupère tous les contacts
pub fn get_all_contacts(conn: &Connection) -> Result<Vec<StoredContact>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT public_key, name, node_type, flags, path, path_len, lat, lon, last_seen, is_favorite, group_name
         FROM contacts ORDER BY name"
    )?;

    let contacts = stmt
        .query_map([], |row| {
            Ok(StoredContact {
                public_key: row.get(0)?,
                name: row.get(1)?,
                node_type: row.get(2)?,
                flags: row.get(3)?,
                path: row.get(4)?,
                path_len: row.get(5)?,
                lat: row.get(6)?,
                lon: row.get(7)?,
                last_seen: row.get(8)?,
                is_favorite: row.get(9)?,
                group_name: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(contacts)
}

/// Récupère un contact par clé publique
pub fn get_contact(
    conn: &Connection,
    public_key: &str,
) -> Result<Option<StoredContact>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT public_key, name, node_type, flags, path, path_len, lat, lon, last_seen, is_favorite, group_name
         FROM contacts WHERE public_key = ?1"
    )?;

    let mut rows = stmt.query_map(params![public_key], |row| {
        Ok(StoredContact {
            public_key: row.get(0)?,
            name: row.get(1)?,
            node_type: row.get(2)?,
            flags: row.get(3)?,
            path: row.get(4)?,
            path_len: row.get(5)?,
            lat: row.get(6)?,
            lon: row.get(7)?,
            last_seen: row.get(8)?,
            is_favorite: row.get(9)?,
            group_name: row.get(10)?,
        })
    })?;

    Ok(rows.next().transpose()?)
}

/// Supprime un contact
pub fn delete_contact(conn: &Connection, public_key: &str) -> Result<bool, StorageError> {
    let affected = conn.execute(
        "DELETE FROM contacts WHERE public_key = ?1",
        params![public_key],
    )?;
    Ok(affected > 0)
}

/// Met à jour le statut favori d'un contact
pub fn set_favorite(
    conn: &Connection,
    public_key: &str,
    favorite: bool,
) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE contacts SET is_favorite = ?1, updated_at = datetime('now') WHERE public_key = ?2",
        params![favorite, public_key],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    fn test_contact() -> StoredContact {
        StoredContact {
            public_key: "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
                .to_string(),
            name: "Test Node".to_string(),
            node_type: 1,
            flags: 0,
            path: vec![],
            path_len: -1,
            lat: 44.75,
            lon: 4.85,
            last_seen: "2026-03-27T10:00:00Z".to_string(),
            is_favorite: false,
            group_name: None,
        }
    }

    #[test]
    fn test_upsert_and_get() {
        let db = Database::in_memory().unwrap();
        let contact = test_contact();

        db.with_conn(|conn| upsert_contact(conn, &contact)).unwrap();
        let fetched = db
            .with_conn(|conn| get_contact(conn, &contact.public_key))
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "Test Node");
    }

    #[test]
    fn test_delete() {
        let db = Database::in_memory().unwrap();
        let contact = test_contact();

        db.with_conn(|conn| upsert_contact(conn, &contact)).unwrap();
        assert!(
            db.with_conn(|conn| delete_contact(conn, &contact.public_key))
                .unwrap()
        );
        assert!(
            db.with_conn(|conn| get_contact(conn, &contact.public_key))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_favorite() {
        let db = Database::in_memory().unwrap();
        let contact = test_contact();

        db.with_conn(|conn| upsert_contact(conn, &contact)).unwrap();
        db.with_conn(|conn| set_favorite(conn, &contact.public_key, true))
            .unwrap();

        let fetched = db
            .with_conn(|conn| get_contact(conn, &contact.public_key))
            .unwrap()
            .unwrap();
        assert!(fetched.is_favorite);
    }
}
