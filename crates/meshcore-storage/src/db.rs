//! Initialisation et migration de la base de données SQLite

use crate::StorageError;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tracing::info;

/// Base de données SQLite pour MeshCore (thread-safe)
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Ouvre ou crée la base de données au chemin donné
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let db = Self { conn: Mutex::new(conn) };
        db.run_migrations()?;
        info!("Base de données ouverte : {}", path.display());
        Ok(db)
    }

    /// Crée une base de données en mémoire (pour les tests)
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn: Mutex::new(conn) };
        db.run_migrations()?;
        Ok(db)
    }

    /// Exécute une opération avec la connexion SQLite
    pub fn with_conn<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&Connection) -> Result<R, StorageError>,
    {
        let conn = self.conn.lock().map_err(|e| StorageError::Migration(e.to_string()))?;
        f(&conn)
    }

    fn run_migrations(&self) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::Migration(e.to_string()))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS contacts (
                public_key TEXT PRIMARY KEY,
                name TEXT NOT NULL DEFAULT '',
                node_type INTEGER NOT NULL DEFAULT 1,
                flags INTEGER NOT NULL DEFAULT 0,
                path BLOB NOT NULL DEFAULT x'',
                path_len INTEGER NOT NULL DEFAULT -1,
                lat REAL NOT NULL DEFAULT 0.0,
                lon REAL NOT NULL DEFAULT 0.0,
                last_seen TEXT NOT NULL DEFAULT '',
                is_favorite INTEGER NOT NULL DEFAULT 0,
                group_name TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                direction TEXT NOT NULL,
                sender_pubkey TEXT,
                sender_name TEXT NOT NULL DEFAULT '',
                recipient_pubkey TEXT,
                channel_idx INTEGER,
                text TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                snr REAL,
                rssi INTEGER,
                path_len INTEGER,
                attempt INTEGER NOT NULL DEFAULT 1,
                reply_to TEXT,
                reaction TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_pubkey);
            CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_pubkey);
            CREATE INDEX IF NOT EXISTS idx_messages_channel ON messages(channel_idx);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);

            CREATE TABLE IF NOT EXISTS channels (
                idx INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                channel_type TEXT NOT NULL,
                psk BLOB NOT NULL,
                notifications_enabled INTEGER NOT NULL DEFAULT 1,
                unread_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS companions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                transport_type TEXT NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                address TEXT NOT NULL,
                pin TEXT,
                last_used TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(transport_type, address)
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "
        ).map_err(|e| StorageError::Migration(e.to_string()))?;

        info!("Migrations appliquées");
        Ok(())
    }
}

// Database is Send + Sync grâce au Mutex
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory() {
        let db = Database::in_memory().unwrap();
        let count = db.with_conn(|conn| {
            let count: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='contacts'",
                    [],
                    |row| row.get(0),
                )?;
            Ok(count)
        }).unwrap();
        assert_eq!(count, 1);
    }
}
