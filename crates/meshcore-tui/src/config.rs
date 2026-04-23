use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "meshcore-tui",
    version,
    about = "TUI MeshCore — client LoRa pour terminal"
)]
pub struct Cli {
    /// Port série (ex: /dev/ttyUSB0)
    #[arg(short, long)]
    pub port: Option<String>,

    /// Baud rate (défaut: 115200)
    #[arg(short, long, default_value = "115200")]
    pub baud: u32,

    /// Connexion TCP (ex: 192.168.1.50:4403)
    #[arg(long)]
    pub tcp: Option<String>,

    /// Connexion BLE par nom (ex: MeshCore-AB12)
    #[arg(long)]
    pub ble: Option<String>,

    /// Répertoire de données (défaut: ~/.local/share/meshcore/)
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Logs verbeux (info)
    #[arg(short, long)]
    pub verbose: bool,
}

impl Cli {
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("meshcore")
        })
    }
}
