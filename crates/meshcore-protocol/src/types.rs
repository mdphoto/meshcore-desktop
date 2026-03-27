use serde::{Deserialize, Serialize};

/// Type de nœud dans le réseau mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    Client = 1,
    Repeater = 2,
    RoomServer = 3,
    Sensor = 4,
}

impl TryFrom<u8> for NodeType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Client),
            2 => Ok(Self::Repeater),
            3 => Ok(Self::RoomServer),
            4 => Ok(Self::Sensor),
            other => Err(other),
        }
    }
}

/// Type de canal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Canal public avec PSK fixe partagé
    Public,
    /// PSK dérivé du nom par SHA-256
    Hashtag,
    /// PSK aléatoire, partagé hors bande
    Private,
    /// PSK dérivé par HMAC-SHA256 d'un secret partagé
    Community,
}

/// Statut de livraison d'un message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
}

/// Direction d'un message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
}

/// Type de transport utilisé pour la connexion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
    Ble,
    Serial,
    Tcp,
}

/// Catégorie de statistiques
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatsCategory {
    Core,
    Radio,
    Packets,
}

/// Chimie de batterie pour le calcul de pourcentage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryChemistry {
    LiPo,
    LiFePO4,
    NiMH,
}

impl BatteryChemistry {
    /// Calcule le pourcentage de batterie à partir du voltage (mV)
    pub fn percentage(&self, millivolts: u16) -> u8 {
        let mv = millivolts as f64;
        let pct = match self {
            Self::LiPo => {
                if mv >= 4200.0 {
                    100.0
                } else if mv <= 3300.0 {
                    0.0
                } else {
                    (mv - 3300.0) / (4200.0 - 3300.0) * 100.0
                }
            }
            Self::LiFePO4 => {
                if mv >= 3600.0 {
                    100.0
                } else if mv <= 2800.0 {
                    0.0
                } else {
                    (mv - 2800.0) / (3600.0 - 2800.0) * 100.0
                }
            }
            Self::NiMH => {
                if mv >= 1450.0 {
                    100.0
                } else if mv <= 1000.0 {
                    0.0
                } else {
                    (mv - 1000.0) / (1450.0 - 1000.0) * 100.0
                }
            }
        };
        pct.clamp(0.0, 100.0) as u8
    }
}
