//! Couche protocole MeshCore
//!
//! Re-exporte les types de meshcore-rs et ajoute des extensions
//! pour la compression SMAZ et les types applicatifs.

pub mod types;
pub mod compression;
pub mod channel;

// Re-export meshcore-rs pour accès direct
pub use meshcore_rs;
pub use meshcore_rs::packets::PacketType;
pub use meshcore_rs::events::Contact;
