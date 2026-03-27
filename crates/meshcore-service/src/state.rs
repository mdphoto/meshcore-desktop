//! État global de l'application

use crate::events::AppEvent;
use crate::los::SrtmCache;
use meshcore_storage::Database;
use meshcore_transport::manager::ConnectionManager;
use std::path::PathBuf;
use tokio::sync::{RwLock, broadcast};

/// État partagé de l'application (Send + Sync)
pub struct AppState {
    /// Gestionnaire de connexion
    pub connection: RwLock<ConnectionManager>,
    /// Base de données
    pub db: Database,
    /// Bus d'événements
    pub event_tx: broadcast::Sender<AppEvent>,
    /// Cache SRTM pour l'analyse ligne de vue
    pub srtm_cache: SrtmCache,
    /// Répertoire de données de l'application
    pub data_dir: PathBuf,
}

impl AppState {
    /// Crée un nouvel état applicatif
    pub fn new(data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("meshcore.db");
        let db = Database::open(&db_path)?;

        let (event_tx, _) = broadcast::channel(256);

        let srtm_cache = SrtmCache::new(data_dir.join("srtm"));

        Ok(Self {
            connection: RwLock::new(ConnectionManager::new()),
            db,
            event_tx,
            srtm_cache,
            data_dir,
        })
    }

    /// Crée un état avec base en mémoire (pour les tests)
    pub fn in_memory() -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::in_memory()?;
        let (event_tx, _) = broadcast::channel(256);

        let srtm_cache = SrtmCache::new(PathBuf::from("/tmp/meshcore-test/srtm"));

        Ok(Self {
            connection: RwLock::new(ConnectionManager::new()),
            db,
            event_tx,
            srtm_cache,
            data_dir: PathBuf::from("/tmp/meshcore-test"),
        })
    }

    /// S'abonne aux événements
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.event_tx.subscribe()
    }

    /// Émet un événement
    pub fn emit(&self, event: AppEvent) {
        let _ = self.event_tx.send(event);
    }
}
