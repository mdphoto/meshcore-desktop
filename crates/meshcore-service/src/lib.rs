//! Couche service MeshCore
//!
//! Orchestre le protocole, le transport, le stockage et la crypto.
//! Gère l'état applicatif et émet des événements vers l'interface.

pub mod channels;
pub mod connection;
pub mod contacts;
pub mod device;
pub mod events;
pub mod los;
pub mod messaging;
pub mod state;

pub use events::AppEvent;
pub use state::AppState;
