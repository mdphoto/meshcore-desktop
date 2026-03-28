# MeshCore Desktop

Client desktop cross-platform pour les réseaux mesh LoRa [MeshCore](https://meshcore.co.uk/).
Communiquez hors réseau via BLE, USB ou TCP avec vos dispositifs MeshCore.

**[English version below](#english)**

> **⚠️ Version beta — Testeurs bienvenus !**
>
> MeshCore Desktop est fonctionnel mais encore jeune. Nous avons besoin de retours sur toutes les plateformes.
> Si vous rencontrez un problème, ouvrez une [issue](https://github.com/mdphoto/meshcore-desktop/issues).
>
> | Plateforme | Connexion BLE | Connexion USB/Série | Connexion TCP/WiFi | GUI | CLI |
> |---|---|---|---|---|---|
> | Linux x86_64 | ✅ Testé | ❓ À tester | ❓ À tester | ✅ Testé | ✅ Testé |
> | Windows x64 | ❓ À tester | ❓ À tester | 🔧 Fix v0.2.1 — à valider | ❓ À tester | ❓ À tester |
> | macOS Intel | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
> | macOS Apple Silicon | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
> | Raspberry Pi (ARM64) | ❓ À tester | ❓ À tester | ❓ À tester | — | ❓ À tester |
>
> **Priorités de test :**
> 1. 🔧 **Windows TCP** : le port par défaut est maintenant 4403 — vérifier la connexion WiFi companion
> 2. ❓ **Windows BLE** : appairage et communication avec un device MeshCore
> 3. ❓ **macOS** : la GUI et la CLI se lancent-elles correctement ?
> 4. ❓ **Raspberry Pi** : la CLI fonctionne-t-elle en SSH avec un device série ?
> 5. ❓ **Linux USB/Série et TCP** : connexion série et WiFi companion

---

## Fonctionnalités

- **Messagerie chiffrée** : messages directs P2P et canaux publics/privés
- **Cartographie** : visualisation des nœuds sur OpenStreetMap, topographique ou satellite
- **Administration repeater** : connexion admin, terminal CLI, statut, voisins, configuration radio
- **Analyse ligne de vue** : profil d'élévation SRTM avec zone de Fresnel
- **Multi-connexion** : BLE, USB série et TCP simultanément
- **Room Servers** : login, chat de groupe, administration
- **CLI headless** : REPL interactif pour Raspberry Pi / serveur SSH
- **Export GPX** : waypoints de tous les nœuds du réseau
- **12 langues** : FR, EN, ES, DE, IT, PT, NL, PL, JA, ZH, KO, RU
- **Notifications OS** : alertes natives pour les messages entrants
- **Deep links** : schéma `meshcore://` pour ouvrir contacts et canaux
- **Thème sombre/clair**

## Captures d'écran

*Bientôt disponibles*

## Téléchargement v0.2.0

### Application GUI (desktop avec interface graphique)

| Plateforme | GUI | CLI |
|---|---|---|
| Linux (Debian/Ubuntu) | [MeshCore.Desktop_0.2.0_amd64.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_amd64.deb) | [meshcore-cli_linux_x86_64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_linux_x86_64) |
| Linux (AppImage) | [MeshCore.Desktop_0.2.0_amd64.AppImage](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_amd64.AppImage) | — |
| Windows | [MeshCore.Desktop_0.2.0_x64-setup.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_x64-setup.exe) | [meshcore-cli_windows_x64.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_windows_x64.exe) |
| macOS (Apple Silicon) | [MeshCore.Desktop_0.2.0_aarch64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_aarch64.dmg) | [meshcore-cli_macos_arm64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_macos_arm64) |
| macOS (Intel) | [MeshCore.Desktop_0.2.0_x64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_x64.dmg) | [meshcore-cli_macos_x64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_macos_x64) |

### Raspberry Pi (ARM64 — CLI uniquement)

| Format | Téléchargement |
|---|---|
| .deb (arm64) | [meshcore-cli_0.2.0_arm64.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_0.2.0_arm64.deb) |
| Binaire brut | [meshcore-cli_linux_arm64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_linux_arm64) |

## Installation

### GUI (Linux)

```bash
# Debian/Ubuntu
sudo dpkg -i MeshCore.Desktop_0.2.0_amd64.deb

# Ou AppImage (pas d'installation requise)
chmod +x MeshCore.Desktop_0.2.0_amd64.AppImage
./MeshCore.Desktop_0.2.0_amd64.AppImage
```

### GUI (Windows)

Lancez `MeshCore.Desktop_0.2.0_x64-setup.exe` et suivez l'installeur.

### GUI (macOS)

Ouvrez le `.dmg` et glissez MeshCore Desktop dans Applications.

### CLI (toutes plateformes)

```bash
# Linux / macOS — rendre exécutable
chmod +x meshcore-cli_linux_x86_64
./meshcore-cli_linux_x86_64 --help

# Raspberry Pi
sudo dpkg -i meshcore-cli_0.2.0_arm64.deb
meshcore-cli --help
```

## Utilisation de la CLI

### Mode interactif (REPL)

Idéal pour un Raspberry Pi accessible en SSH :

```bash
meshcore-cli --port /dev/ttyUSB0
```

```
meshcore [/dev/ttyUSB0] > contacts
Nom                  Clé            Type       Hops   Vu
Michel               abcdef123456   Client     1      2026-03-28
Repeater-01          fedcba654321   Repeater   0      2026-03-28
2 contacts

meshcore [/dev/ttyUSB0] > send Michel Salut depuis le toit !
Envoyé (id: a1b2c3d4)

meshcore [/dev/ttyUSB0] > repeater login fedcba654321 monmdp
Login envoyé

meshcore [/dev/ttyUSB0] > repeater cli fedcba654321 ver
MeshCore v1.2.3

meshcore [/dev/ttyUSB0] > quit
```

### Mode one-shot (scripts)

```bash
# Lister les contacts
meshcore-cli --port /dev/ttyUSB0 contacts list

# Envoyer un message
meshcore-cli --tcp 192.168.1.50:4403 send Michel "Hello mesh !"

# Sortie JSON pour scripting
meshcore-cli --port /dev/ttyUSB0 --json device
```

### Options de connexion

| Option | Exemple | Description |
|---|---|---|
| `--port` / `-p` | `--port /dev/ttyUSB0` | Connexion série USB |
| `--baud` / `-b` | `--baud 115200` | Baud rate (défaut: 115200) |
| `--tcp` | `--tcp 192.168.1.50:4403` | Connexion TCP |
| `--ble` | `--ble MeshCore-AB12` | Connexion Bluetooth LE |
| `--json` | | Sortie JSON |
| `--verbose` / `-v` | | Logs détaillés |

## Compilation depuis les sources

### Prérequis

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI : `cargo install tauri-cli --version "^2"`

#### Linux (dépendances système)

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev pkg-config
```

### Build GUI

```bash
git clone https://github.com/mdphoto/meshcore-desktop.git
cd meshcore-desktop
cd frontend && npm install && npm run build && cd ..
cargo tauri build
```

### Build CLI seule

```bash
cargo build --release -p meshcore-cli
./target/release/meshcore-cli --help
```

### Tests

```bash
# Tests Rust (21 tests)
cargo test --workspace

# Tests frontend (41 tests)
cd frontend && npm test
```

## Architecture

```
meshcore-desktop/
├── crates/
│   ├── meshcore-protocol/    # Types, compression SMAZ, canaux
│   ├── meshcore-crypto/      # AES-128-ECB, HMAC-SHA256, Ed25519
│   ├── meshcore-transport/   # BLE, Serial, TCP, reconnexion auto
│   ├── meshcore-storage/     # SQLite (contacts, messages, canaux)
│   ├── meshcore-service/     # Logique métier, état, événements, LOS
│   ├── meshcore-app/         # Application Tauri GUI, 50 commandes IPC
│   └── meshcore-cli/         # CLI headless (REPL + one-shot)
└── frontend/                 # React 19, TypeScript, Tailwind CSS
```

Basé sur la bibliothèque [meshcore-rs](https://crates.io/crates/meshcore-rs) pour le protocole MeshCore.

## Matériel compatible

Tous les dispositifs supportés par MeshCore : Heltec, RAK Wireless, Seeed, nRF52.
Liste complète sur [flasher.meshcore.co.uk](https://flasher.meshcore.co.uk/).

## Licence

MIT

## Auteur

Michel Dessenne — [IELOW SAS](https://ielow.fr)

---

<a id="english"></a>

# MeshCore Desktop (English)

Cross-platform desktop client for [MeshCore](https://meshcore.co.uk/) LoRa mesh networks.
Communicate off-grid via BLE, USB or TCP with your MeshCore devices.

> **⚠️ Beta version — Testers welcome!**
>
> MeshCore Desktop is functional but still young. We need feedback on all platforms.
> If you find an issue, please open an [issue](https://github.com/mdphoto/meshcore-desktop/issues).
>
> | Platform | BLE | USB/Serial | TCP/WiFi | GUI | CLI |
> |---|---|---|---|---|---|
> | Linux x86_64 | ✅ Tested | ❓ Untested | ❓ Untested | ✅ Tested | ✅ Tested |
> | Windows x64 | ❓ Untested | ❓ Untested | 🔧 Fix v0.2.1 — needs validation | ❓ Untested | ❓ Untested |
> | macOS Intel | ❓ Untested | ❓ Untested | ❓ Untested | ❓ Untested | ❓ Untested |
> | macOS Apple Silicon | ❓ Untested | ❓ Untested | ❓ Untested | ❓ Untested | ❓ Untested |
> | Raspberry Pi (ARM64) | ❓ Untested | ❓ Untested | ❓ Untested | — | ❓ Untested |
>
> **Testing priorities:**
> 1. 🔧 **Windows TCP**: default port is now 4403 — verify WiFi companion connection
> 2. ❓ **Windows BLE**: pairing and communication with a MeshCore device
> 3. ❓ **macOS**: do the GUI and CLI launch correctly?
> 4. ❓ **Raspberry Pi**: does the CLI work via SSH with a serial device?
> 5. ❓ **Linux USB/Serial and TCP**: serial and WiFi companion connections

## Features

- **Encrypted messaging**: direct P2P messages and public/private channels
- **Mapping**: node visualization on OpenStreetMap, topographic or satellite tiles
- **Repeater admin**: admin login, CLI terminal, status, neighbours, radio configuration
- **Line-of-sight analysis**: SRTM elevation profile with Fresnel zone
- **Multi-connection**: BLE, USB serial and TCP simultaneously
- **Room Servers**: login, group chat, administration
- **Headless CLI**: interactive REPL for Raspberry Pi / SSH server
- **GPX export**: waypoints for all mesh nodes
- **12 languages**: FR, EN, ES, DE, IT, PT, NL, PL, JA, ZH, KO, RU
- **OS notifications**: native alerts for incoming messages
- **Deep links**: `meshcore://` scheme to open contacts and channels
- **Dark/light theme**

## Download v0.2.0

### GUI Application (desktop with graphical interface)

| Platform | GUI | CLI |
|---|---|---|
| Linux (Debian/Ubuntu) | [.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_amd64.deb) | [meshcore-cli_linux_x86_64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_linux_x86_64) |
| Linux (AppImage) | [.AppImage](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_amd64.AppImage) | — |
| Windows | [.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_x64-setup.exe) | [meshcore-cli_windows_x64.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_windows_x64.exe) |
| macOS (Apple Silicon) | [.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_aarch64.dmg) | [meshcore-cli_macos_arm64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_macos_arm64) |
| macOS (Intel) | [.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/MeshCore.Desktop_0.1.0_x64.dmg) | [meshcore-cli_macos_x64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_macos_x64) |

### Raspberry Pi (ARM64 — CLI only)

| Format | Download |
|---|---|
| .deb (arm64) | [meshcore-cli_0.2.0_arm64.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_0.2.0_arm64.deb) |
| Raw binary | [meshcore-cli_linux_arm64](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.2.0/meshcore-cli_linux_arm64) |

## Installation

### GUI (Linux)

```bash
sudo dpkg -i MeshCore.Desktop_0.2.0_amd64.deb
# Or AppImage:
chmod +x MeshCore.Desktop_0.2.0_amd64.AppImage && ./MeshCore.Desktop_0.2.0_amd64.AppImage
```

### GUI (Windows)

Run `MeshCore.Desktop_0.2.0_x64-setup.exe` and follow the installer.

### GUI (macOS)

Open the `.dmg` and drag MeshCore Desktop to Applications.

### CLI (all platforms)

```bash
chmod +x meshcore-cli_linux_x86_64
./meshcore-cli_linux_x86_64 --help

# Raspberry Pi
sudo dpkg -i meshcore-cli_0.2.0_arm64.deb
meshcore-cli --help
```

## CLI Usage

### Interactive mode (REPL)

Ideal for a Raspberry Pi accessible via SSH:

```bash
meshcore-cli --port /dev/ttyUSB0
```

```
meshcore [/dev/ttyUSB0] > contacts
meshcore [/dev/ttyUSB0] > send Michel Hello from the roof!
meshcore [/dev/ttyUSB0] > repeater login fedcba654321 mypassword
meshcore [/dev/ttyUSB0] > repeater cli fedcba654321 ver
meshcore [/dev/ttyUSB0] > quit
```

### One-shot mode (scripting)

```bash
meshcore-cli --port /dev/ttyUSB0 contacts list
meshcore-cli --tcp 192.168.1.50:4403 --json device
meshcore-cli --port /dev/ttyUSB0 send Michel "Hello mesh!"
```

### Connection options

| Option | Example | Description |
|---|---|---|
| `--port` / `-p` | `--port /dev/ttyUSB0` | Serial USB connection |
| `--baud` / `-b` | `--baud 115200` | Baud rate (default: 115200) |
| `--tcp` | `--tcp 192.168.1.50:4403` | TCP connection |
| `--ble` | `--ble MeshCore-AB12` | Bluetooth LE connection |
| `--json` | | JSON output |
| `--verbose` / `-v` | | Detailed logs |

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI: `cargo install tauri-cli --version "^2"`

#### Linux (system dependencies)

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev pkg-config
```

### Build GUI

```bash
git clone https://github.com/mdphoto/meshcore-desktop.git
cd meshcore-desktop
cd frontend && npm install && npm run build && cd ..
cargo tauri build
```

### Build CLI only

```bash
cargo build --release -p meshcore-cli
./target/release/meshcore-cli --help
```

### Tests

```bash
cargo test --workspace     # 21 Rust tests
cd frontend && npm test    # 41 frontend tests
```

## Architecture

```
meshcore-desktop/
├── crates/
│   ├── meshcore-protocol/    # Types, SMAZ compression, channels
│   ├── meshcore-crypto/      # AES-128-ECB, HMAC-SHA256, Ed25519
│   ├── meshcore-transport/   # BLE, Serial, TCP, auto-reconnect
│   ├── meshcore-storage/     # SQLite (contacts, messages, channels)
│   ├── meshcore-service/     # Business logic, state, events, LOS
│   ├── meshcore-app/         # Tauri GUI application, 50 IPC commands
│   └── meshcore-cli/         # Headless CLI (REPL + one-shot)
└── frontend/                 # React 19, TypeScript, Tailwind CSS
```

Built on top of [meshcore-rs](https://crates.io/crates/meshcore-rs) for the MeshCore protocol.

## Compatible hardware

All MeshCore-supported devices: Heltec, RAK Wireless, Seeed, nRF52.
Full list at [flasher.meshcore.co.uk](https://flasher.meshcore.co.uk/).

## License

MIT

## Author

Michel Dessenne — [IELOW SAS](https://ielow.fr)
