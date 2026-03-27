//! Couche transport MeshCore
//!
//! Abstraction des connexions BLE, Serial et TCP vers les dispositifs MeshCore.
//! Utilise `meshcore-rs` comme backend et ajoute la gestion de reconnexion.

pub mod manager;
pub mod reconnect;

use thiserror::Error;

/// Erreurs de la couche transport
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Erreur de connexion : {0}")]
    Connection(String),
    #[error("Dispositif non trouvé : {0}")]
    DeviceNotFound(String),
    #[error("Timeout de connexion après {0}s")]
    Timeout(u64),
    #[error("Connexion perdue")]
    Disconnected,
    #[error("Erreur BLE : {0}")]
    Ble(String),
    #[error("Erreur série : {0}")]
    Serial(String),
    #[error("Erreur TCP : {0}")]
    Tcp(String),
    #[error("Erreur meshcore-rs : {0}")]
    MeshCore(String),
}
