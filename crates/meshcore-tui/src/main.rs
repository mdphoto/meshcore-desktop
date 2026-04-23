// Certaines variantes d'Action et des champs d'état sont volontairement présents mais non encore
// utilisés — ils seront branchés en M2/M3 (chat, canaux, device admin).
#![allow(dead_code)]

mod action;
mod actions;
mod app;
mod config;
mod events;
mod keymap;
mod state;
mod theme;
mod tui;
mod ui;
mod util;

use anyhow::Result;
use app::App;
use clap::Parser;
use config::Cli;
use meshcore_service::AppState;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        "info,meshcore=debug"
    } else {
        "warn"
    };
    // Rediriger les logs vers un fichier pour ne pas polluer l'écran TUI
    let log_dir = cli.data_dir();
    std::fs::create_dir_all(&log_dir)?;
    let log_path = log_dir.join("tui.log");
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_writer(log_file)
        .with_ansi(false)
        .with_target(false)
        .init();

    let data_dir = cli.data_dir();
    let service = Arc::new(
        AppState::new(data_dir).map_err(|e| anyhow::anyhow!("Init AppState : {}", e))?,
    );

    // Auto-connexion explicite si args CLI — sinon App::run tentera le dernier companion
    let has_cli_target = cli.port.is_some() || cli.tcp.is_some() || cli.ble.is_some();
    if has_cli_target {
        maybe_auto_connect(&cli, service.clone()).await;
    }

    let mut tui = tui::Tui::new()?;
    let (action_tx, action_rx) = mpsc::unbounded_channel::<action::Action>();
    let mut app = App::new(service, action_tx);
    app.auto_reconnect_on_startup = !has_cli_target;

    let run_result = app.run(&mut tui, action_rx).await;

    // Drop de Tui restore déjà, on explicite pour la clarté
    drop(tui);
    run_result
}

async fn maybe_auto_connect(cli: &Cli, service: Arc<AppState>) {
    use meshcore_transport::manager::ConnectionTarget;

    if let Some(port) = &cli.port {
        let target = ConnectionTarget::Serial {
            port: port.clone(),
            baud_rate: cli.baud,
        };
        let _ = meshcore_service::connection::connect(&service, target).await;
    } else if let Some(tcp) = &cli.tcp {
        if let Some(target) = actions::connection::parse_tcp(tcp) {
            let _ = meshcore_service::connection::connect(&service, target).await;
        }
    } else if let Some(ble) = &cli.ble {
        let target = ConnectionTarget::Ble {
            name_or_addr: ble.clone(),
        };
        let _ = meshcore_service::connection::connect(&service, target).await;
    }
}
