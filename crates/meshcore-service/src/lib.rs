//! Couche service MeshCore
//!
//! Orchestre le protocole, le transport, le stockage et la crypto.
//! Gère l'état applicatif et émet des événements vers l'interface.

pub mod state;
pub mod events;
pub mod connection;
pub mod messaging;
pub mod contacts;
pub mod channels;
pub mod device;
pub mod los;

pub use state::AppState;
pub use events::AppEvent;
