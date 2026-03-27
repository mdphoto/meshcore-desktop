//! Gestionnaire de connexion unifié
//!
//! Encapsule `meshcore_rs::MeshCore` et fournit une interface haut niveau
//! pour la connexion, la déconnexion et l'envoi de commandes.
//! Supporte les connexions simultanées via plusieurs slots nommés.

use crate::TransportError;
use meshcore_rs::MeshCore;
use std::collections::HashMap;
use tracing::info;

/// Type de connexion
#[derive(Debug, Clone)]
pub enum ConnectionTarget {
    /// Connexion BLE par nom ou adresse
    Ble { name_or_addr: String },
    /// Connexion série
    Serial { port: String, baud_rate: u32 },
    /// Connexion TCP
    Tcp { host: String, port: u16 },
}

impl ConnectionTarget {
    /// Génère un identifiant unique pour cette cible
    pub fn id(&self) -> String {
        match self {
            Self::Ble { name_or_addr } => format!("ble:{}", name_or_addr),
            Self::Serial { port, baud_rate } => format!("serial:{}:{}", port, baud_rate),
            Self::Tcp { host, port } => format!("tcp:{}:{}", host, port),
        }
    }
}

/// Connexion active
struct ActiveConnection {
    meshcore: MeshCore,
    target: ConnectionTarget,
}

/// Gestionnaire multi-connexion vers des dispositifs MeshCore
///
/// Maintient une connexion "primaire" (la première ou la dernière sélectionnée)
/// plus des connexions secondaires optionnelles.
pub struct ConnectionManager {
    /// Connexions actives par identifiant
    connections: HashMap<String, ActiveConnection>,
    /// ID de la connexion primaire (celle utilisée par défaut pour les commandes)
    primary_id: Option<String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            primary_id: None,
        }
    }

    /// Connecte au dispositif MeshCore spécifié
    pub async fn connect(&mut self, target: ConnectionTarget) -> Result<(), TransportError> {
        let id = target.id();
        info!("Connexion à {:?} (id: {})", target, id);

        let mc = match &target {
            ConnectionTarget::Serial { port, baud_rate } => MeshCore::serial(port, *baud_rate)
                .await
                .map_err(|e| TransportError::Serial(e.to_string()))?,
            ConnectionTarget::Tcp { host, port } => MeshCore::tcp(host, *port)
                .await
                .map_err(|e| TransportError::Tcp(e.to_string()))?,
            ConnectionTarget::Ble { name_or_addr } => MeshCore::ble_connect(name_or_addr)
                .await
                .map_err(|e| TransportError::Ble(e.to_string()))?,
        };

        // Si c'est la première connexion, elle devient primaire
        if self.primary_id.is_none() {
            self.primary_id = Some(id.clone());
        }

        self.connections.insert(
            id.clone(),
            ActiveConnection {
                meshcore: mc,
                target,
            },
        );
        info!(
            "Connecté (id: {}), total: {} connexion(s)",
            id,
            self.connections.len()
        );
        Ok(())
    }

    /// Déconnecte la connexion primaire (ou une connexion spécifique)
    pub async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Some(id) = self.primary_id.take() {
            self.disconnect_by_id(&id).await?;
        }
        // Promouvoir la prochaine connexion comme primaire
        self.primary_id = self.connections.keys().next().cloned();
        Ok(())
    }

    /// Déconnecte une connexion spécifique par ID
    pub async fn disconnect_by_id(&mut self, id: &str) -> Result<(), TransportError> {
        if let Some(conn) = self.connections.remove(id) {
            match conn.meshcore.disconnect().await {
                Ok(()) => info!("Déconnecté (id: {})", id),
                Err(e) => info!("Erreur disconnect {} (nettoyage) : {}", id, e),
            }
        }
        if self.primary_id.as_deref() == Some(id) {
            self.primary_id = self.connections.keys().next().cloned();
        }
        Ok(())
    }

    /// Déconnecte toutes les connexions
    pub async fn disconnect_all(&mut self) {
        let ids: Vec<String> = self.connections.keys().cloned().collect();
        for id in ids {
            let _ = self.disconnect_by_id(&id).await;
        }
        self.primary_id = None;
    }

    /// Vérifie si au moins une connexion est active
    pub async fn is_connected(&self) -> bool {
        for conn in self.connections.values() {
            if conn.meshcore.is_connected().await {
                return true;
            }
        }
        false
    }

    /// Accède au client MeshCore primaire
    pub fn meshcore(&self) -> Option<&MeshCore> {
        let id = self.primary_id.as_ref()?;
        self.connections.get(id).map(|c| &c.meshcore)
    }

    /// Accède à un client MeshCore par ID
    pub fn meshcore_by_id(&self, id: &str) -> Option<&MeshCore> {
        self.connections.get(id).map(|c| &c.meshcore)
    }

    /// Sélectionne une connexion comme primaire
    pub fn set_primary(&mut self, id: &str) -> bool {
        if self.connections.contains_key(id) {
            self.primary_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Retourne la cible de connexion primaire
    pub fn target(&self) -> Option<&ConnectionTarget> {
        let id = self.primary_id.as_ref()?;
        self.connections.get(id).map(|c| &c.target)
    }

    /// Liste toutes les connexions actives (id, target)
    pub fn list_connections(&self) -> Vec<(String, &ConnectionTarget, bool)> {
        self.connections
            .iter()
            .map(|(id, conn)| {
                let is_primary = self.primary_id.as_deref() == Some(id.as_str());
                (id.clone(), &conn.target, is_primary)
            })
            .collect()
    }

    /// Nombre de connexions actives
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
