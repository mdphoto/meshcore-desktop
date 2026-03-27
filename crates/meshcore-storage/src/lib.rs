//! Couche de persistance MeshCore basée sur SQLite
//!
//! Gère le stockage des contacts, messages, canaux, companions et paramètres.

pub mod db;
pub mod models;
pub mod contacts;
pub mod messages;
pub mod channels;
pub mod companions;
pub mod settings;

pub use db::Database;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Erreur SQLite : {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Erreur de migration : {0}")]
    Migration(String),
    #[error("Enregistrement non trouvé : {0}")]
    NotFound(String),
    #[error("Erreur de sérialisation : {0}")]
    Serialization(String),
}
