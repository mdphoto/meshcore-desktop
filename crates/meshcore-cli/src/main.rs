//! MeshCore — binaire unifié
//!
//! - Sans sous-commande : lance la TUI ratatui (mode interactif par défaut)
//! - Avec sous-commande : exécute une commande one-shot (scripting, headless)
//! - `--repl` : ancien REPL rustyline (legacy)
//!
//! Usage headless sur Raspberry Pi, serveur SSH, scripting — tout en un.

mod commands;
mod display;
mod repl;

use clap::Parser;
use meshcore_service::AppState;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "meshcore-desktop",
    version,
    about = "MeshCore Desktop — TUI + CLI unifié pour dispositifs LoRa"
)]
struct Cli {
    /// Port série (ex: /dev/ttyUSB0)
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate (défaut: 115200)
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Connexion TCP (ex: 192.168.1.50:4403)
    #[arg(long)]
    tcp: Option<String>,

    /// Connexion BLE par nom (ex: MeshCore-AB12)
    #[arg(long)]
    ble: Option<String>,

    /// Répertoire de données (défaut: ~/.local/share/meshcore/)
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Sortie JSON (pour scripting, avec sous-commande)
    #[arg(long)]
    json: bool,

    /// Mode verbose (debug logs)
    #[arg(short, long)]
    verbose: bool,

    /// Utilise l'ancien REPL rustyline au lieu de la TUI (sans sous-commande)
    #[arg(long)]
    repl: bool,

    /// Commande one-shot (si absent : lance la TUI, ou le REPL avec --repl)
    #[command(subcommand)]
    command: Option<SubCommand>,
}

#[derive(clap::Subcommand)]
enum SubCommand {
    /// Lister les contacts
    Contacts {
        #[command(subcommand)]
        action: Option<ContactsAction>,
    },
    /// Envoyer un message direct
    Send {
        /// Nom ou clé publique du destinataire
        dest: String,
        /// Message à envoyer
        message: Vec<String>,
    },
    /// Envoyer un message sur un canal
    Channel {
        /// Index du canal
        idx: u8,
        /// Message à envoyer
        message: Vec<String>,
    },
    /// Lister les canaux
    Channels,
    /// Informations du dispositif
    Device,
    /// Batterie du dispositif
    Battery {
        /// Chimie: lipo, lifepo4, nimh
        #[arg(default_value = "lipo")]
        chemistry: String,
    },
}

#[derive(clap::Subcommand)]
enum ContactsAction {
    /// Synchroniser depuis le dispositif
    Sync,
    /// Lister les contacts (par défaut)
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let is_interactive = cli.command.is_none();
    let use_tui = is_interactive && !cli.repl;

    // En mode TUI : logs redirigés vers fichier (pas de pollution écran)
    // En mode REPL/one-shot : logs vers stderr
    let data_dir = cli.data_dir.clone().unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("meshcore")
    });
    let filter = if cli.verbose {
        "info,meshcore=debug"
    } else {
        "warn"
    };

    if use_tui {
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            eprintln!("Erreur création data_dir : {}", e);
            std::process::exit(1);
        }
        let log_path = data_dir.join("tui.log");
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => {
                tracing_subscriber::fmt()
                    .with_env_filter(
                        EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| EnvFilter::new(filter)),
                    )
                    .with_writer(file)
                    .with_ansi(false)
                    .with_target(false)
                    .init();
            }
            Err(e) => {
                eprintln!("Erreur ouverture log TUI : {}", e);
                std::process::exit(1);
            }
        }
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
            )
            .with_target(false)
            .init();
    }

    let state = match AppState::new(data_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur initialisation : {}", e);
            std::process::exit(1);
        }
    };

    // Auto-connexion explicite si un target CLI est fourni
    let has_cli_target = cli.port.is_some() || cli.tcp.is_some() || cli.ble.is_some();
    if let Some(ref port) = cli.port {
        let target = meshcore_transport::manager::ConnectionTarget::Serial {
            port: port.clone(),
            baud_rate: cli.baud,
        };
        if let Err(e) = meshcore_service::connection::connect(&state, target).await {
            eprintln!("Erreur connexion série : {}", e);
            if cli.command.is_some() {
                std::process::exit(1);
            }
        }
    } else if let Some(ref tcp) = cli.tcp {
        let parts: Vec<&str> = tcp.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let port = parts[0].parse::<u16>().unwrap_or(4403);
            let host = parts[1].to_string();
            let target = meshcore_transport::manager::ConnectionTarget::Tcp { host, port };
            if let Err(e) = meshcore_service::connection::connect(&state, target).await {
                eprintln!("Erreur connexion TCP : {}", e);
                if cli.command.is_some() {
                    std::process::exit(1);
                }
            }
        }
    } else if let Some(ref ble) = cli.ble {
        let target = meshcore_transport::manager::ConnectionTarget::Ble {
            name_or_addr: ble.clone(),
        };
        if let Err(e) = meshcore_service::connection::connect(&state, target).await {
            eprintln!("Erreur connexion BLE : {}", e);
            if cli.command.is_some() {
                std::process::exit(1);
            }
        }
    }

    // Dispatch : TUI / REPL / one-shot
    match cli.command {
        Some(cmd) => {
            if let Err(e) = commands::execute_subcommand(&state, cmd, cli.json).await {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            if cli.repl {
                repl::run(&state, cli.json).await;
            } else {
                // Mode par défaut : TUI ratatui
                let service = Arc::new(state);
                let auto_reconnect = !has_cli_target;
                if let Err(e) = meshcore_tui::run_tui(service, auto_reconnect).await {
                    // Le terminal est déjà restauré par le Drop de Tui, on peut imprimer
                    eprintln!("Erreur TUI : {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
