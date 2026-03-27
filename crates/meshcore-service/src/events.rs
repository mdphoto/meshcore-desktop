//! Événements applicatifs émis par la couche service

use serde::{Deserialize, Serialize};

/// Événement émis par l'application vers l'interface
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    /// Connexion établie
    Connected { device_name: String },
    /// Connexion perdue
    Disconnected,
    /// Tentative de reconnexion en cours
    Reconnecting { attempt: u32 },

    /// Message direct reçu
    DirectMessageReceived {
        sender_pubkey: String,
        sender_name: String,
        text: String,
        snr: Option<f32>,
    },
    /// Message de canal reçu
    ChannelMessageReceived {
        channel_idx: u8,
        sender_name: String,
        text: String,
    },
    /// Confirmation d'envoi
    MessageSent { message_id: String },
    /// ACK reçu (message délivré)
    MessageDelivered { message_id: String },
    /// Échec d'envoi
    MessageFailed { message_id: String, reason: String },

    /// Contacts synchronisés
    ContactsSynced { count: usize },
    /// Nouveau contact découvert
    ContactDiscovered {
        pubkey: String,
        name: String,
        node_type: u8,
    },
    /// Mise à jour de chemin
    PathUpdated {
        pubkey_prefix: String,
        path_len: i8,
    },

    /// Info batterie
    BatteryUpdate { millivolts: u16, percent: u8 },
    /// Info dispositif
    DeviceInfoReceived { name: String, fw_version: String },
    /// Statistiques radio
    StatsReceived { noise_floor: i16, last_rssi: i16, snr: f32 },

    /// Erreur
    Error { message: String },
}
