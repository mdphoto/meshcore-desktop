//! Couche TUI (ratatui) pour MeshCore — exposée comme library pour être embarquée dans le
//! binaire unifié `meshcore-cli`.
//!
//! Entrypoint principal : [`run_tui`].

// Variantes d'Action et champs d'état réservés pour futures extensions.
#![allow(dead_code)]

pub mod action;
pub mod actions;
pub mod app;
pub mod config;
pub mod events;
pub mod keymap;
pub mod state;
pub mod theme;
pub mod tui;
pub mod ui;
pub mod util;

use anyhow::Result;
use app::App;
use meshcorex_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Lance la TUI avec un `AppState` déjà construit.
///
/// - `service` : état applicatif partagé (connexion + DB + bus d'événements)
/// - `auto_reconnect` : si `true`, la TUI tentera de se reconnecter au dernier
///   companion sauvegardé au démarrage (cas typique : aucun paramètre CLI fourni)
pub async fn run_tui(service: Arc<AppState>, auto_reconnect: bool) -> Result<()> {
    let mut tui = tui::Tui::new()?;
    let (action_tx, action_rx) = mpsc::unbounded_channel::<action::Action>();
    let mut app = App::new(service, action_tx);
    app.auto_reconnect_on_startup = auto_reconnect;

    let result = app.run(&mut tui, action_rx).await;
    drop(tui);
    result
}
