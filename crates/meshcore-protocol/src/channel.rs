//! Gestion des types de canaux et dérivation des PSK

use crate::types::ChannelType;
use serde::{Deserialize, Serialize};

/// Canal MeshCore avec son type et sa clé pré-partagée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub index: u8,
    pub name: String,
    pub channel_type: ChannelType,
    pub psk: [u8; 16],
    pub notifications_enabled: bool,
}

/// Les 16 octets PSK fixes pour le canal Public (universel MeshCore)
pub const PUBLIC_CHANNEL_PSK: [u8; 16] = [
    0x8b, 0x33, 0x87, 0xe9, 0xc5, 0xcd, 0xea, 0x6a,
    0xc9, 0xe5, 0xed, 0xba, 0xa1, 0x15, 0xcd, 0x72,
];

impl Channel {
    /// Crée un canal public avec le PSK fixe
    pub fn new_public(index: u8, name: String) -> Self {
        Self {
            index,
            name,
            channel_type: ChannelType::Public,
            psk: PUBLIC_CHANNEL_PSK,
            notifications_enabled: true,
        }
    }

    /// Crée un canal hashtag avec PSK dérivé du nom par SHA-256
    pub fn new_hashtag(index: u8, name: String, psk: [u8; 16]) -> Self {
        Self {
            index,
            name,
            channel_type: ChannelType::Hashtag,
            psk,
            notifications_enabled: true,
        }
    }

    /// Crée un canal privé avec PSK aléatoire
    pub fn new_private(index: u8, name: String, psk: [u8; 16]) -> Self {
        Self {
            index,
            name,
            channel_type: ChannelType::Private,
            psk,
            notifications_enabled: true,
        }
    }

    /// Crée un canal communautaire avec PSK dérivé par HMAC-SHA256
    pub fn new_community(index: u8, name: String, psk: [u8; 16]) -> Self {
        Self {
            index,
            name,
            channel_type: ChannelType::Community,
            psk,
            notifications_enabled: true,
        }
    }
}
