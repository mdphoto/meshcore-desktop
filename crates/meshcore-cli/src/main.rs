//! MeshCore CLI — client en ligne de commande pour dispositifs MeshCore
//!
//! Usage headless sur Raspberry Pi, serveur SSH, ou scripting.

mod commands;
mod display;
mod repl;

use clap::Parser;
use meshcore_service::AppState;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "meshcore-cli",
    version,
    about = "CLI pour dispositifs MeshCore LoRa"
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

    /// Sortie JSON (pour scripting)
    #[arg(long)]
    json: bool,

    /// Mode verbose (debug logs)
    #[arg(short, long)]
    verbose: bool,

    /// Commande one-shot (si absent, lance le REPL interactif)
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

    // Logging
    let filter = if cli.verbose {
        "info,meshcore=debug"
    } else {
        "warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_target(false)
        .init();

    // Répertoire de données
    let data_dir = cli.data_dir.unwrap_or_else(|| {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("meshcore")
    });

    let state = match AppState::new(data_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Erreur initialisation : {}", e);
            std::process::exit(1);
        }
    };

    // Auto-connexion si paramètres fournis
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

    // Mode one-shot ou REPL
    match cli.command {
        Some(cmd) => {
            if let Err(e) = commands::execute_subcommand(&state, cmd, cli.json).await {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            repl::run(&state, cli.json).await;
        }
    }
}
