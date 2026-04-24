//! Modèles de données pour la persistance

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Contact stocké en base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContact {
    pub public_key: String,
    pub name: String,
    pub node_type: u8,
    pub flags: u8,
    pub path: Vec<u8>,
    pub path_len: i8,
    pub lat: f64,
    pub lon: f64,
    pub last_seen: String,
    pub is_favorite: bool,
    pub group_name: Option<String>,
}

/// Message stocké en base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: String,
    pub direction: String,
    pub sender_pubkey: Option<String>,
    pub sender_name: String,
    pub recipient_pubkey: Option<String>,
    pub channel_idx: Option<u8>,
    pub text: String,
    pub timestamp: String,
    pub status: String,
    pub snr: Option<f32>,
    pub rssi: Option<i16>,
    pub path_len: Option<u8>,
    pub attempt: u8,
    pub reply_to: Option<String>,
    pub reaction: Option<String>,
}

impl StoredMessage {
    /// Crée un nouveau message sortant (direct)
    pub fn new_outgoing(recipient_pubkey: &str, text: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            direction: "outgoing".to_string(),
            sender_pubkey: None,
            sender_name: String::new(),
            recipient_pubkey: Some(recipient_pubkey.to_string()),
            channel_idx: None,
            text: text.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            status: "pending".to_string(),
            snr: None,
            rssi: None,
            path_len: None,
            attempt: 1,
            reply_to: None,
            reaction: None,
        }
    }

    /// Crée un nouveau message sortant (canal)
    pub fn new_channel_outgoing(channel_idx: u8, text: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            direction: "outgoing".to_string(),
            sender_pubkey: None,
            sender_name: String::new(),
            recipient_pubkey: None,
            channel_idx: Some(channel_idx),
            text: text.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            status: "pending".to_string(),
            snr: None,
            rssi: None,
            path_len: None,
            attempt: 1,
            reply_to: None,
            reaction: None,
        }
    }
}

/// Companion (dispositif connu)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCompanion {
    pub id: Option<i64>,
    pub transport_type: String,
    pub name: String,
    pub address: String,
    pub pin: Option<String>,
    pub last_used: String,
}
