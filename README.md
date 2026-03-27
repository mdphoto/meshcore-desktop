# MeshCore Desktop

Client desktop cross-platform pour les réseaux mesh LoRa [MeshCore](https://meshcore.co.uk/).
Communiquez hors réseau via BLE, USB ou TCP avec vos dispositifs MeshCore.

**[English version below](#english)**

---

## Fonctionnalités

- **Messagerie chiffrée** : messages directs P2P et canaux publics/privés
- **Cartographie** : visualisation des nœuds sur OpenStreetMap, topographique ou satellite
- **Administration repeater** : connexion admin, terminal CLI, statut, voisins, configuration radio
- **Analyse ligne de vue** : profil d'élévation SRTM avec zone de Fresnel
- **Multi-connexion** : BLE, USB série et TCP simultanément
- **Room Servers** : login, chat de groupe, administration
- **Export GPX** : waypoints de tous les nœuds du réseau
- **12 langues** : FR, EN, ES, DE, IT, PT, NL, PL, JA, ZH, KO, RU
- **Notifications OS** : alertes natives pour les messages entrants
- **Deep links** : schéma `meshcore://` pour ouvrir contacts et canaux
- **Thème sombre/clair**

## Captures d'écran

*Bientôt disponibles*

## Téléchargement v0.1.0

| Plateforme | Téléchargement |
|---|---|
| Linux (Debian/Ubuntu) | [MeshCore.Desktop_0.1.0_amd64.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_amd64.deb) |
| Linux (AppImage) | [MeshCore.Desktop_0.1.0_amd64.AppImage](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_amd64.AppImage) |
| Windows | [MeshCore.Desktop_0.1.0_x64-setup.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_x64-setup.exe) |
| macOS (Apple Silicon) | [MeshCore.Desktop_0.1.0_aarch64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_aarch64.dmg) |
| macOS (Intel) | [MeshCore.Desktop_0.1.0_x64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_x64.dmg) |

## Installation

### Linux

```bash
# Debian/Ubuntu
sudo dpkg -i MeshCore.Desktop_0.1.0_amd64.deb

# Ou AppImage (pas d'installation requise)
chmod +x MeshCore.Desktop_0.1.0_amd64.AppImage
./MeshCore.Desktop_0.1.0_amd64.AppImage
```

### Windows

Lancez `MeshCore.Desktop_0.1.0_x64-setup.exe` et suivez l'installeur.

### macOS

Ouvrez le `.dmg` et glissez MeshCore Desktop dans Applications.

## Compilation depuis les sources

### Prérequis

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI : `cargo install tauri-cli --version "^2"`

#### Linux (dépendances système)

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev pkg-config
```

### Build

```bash
# Cloner le projet
git clone https://github.com/mdphoto/meshcore-desktop.git
cd meshcore-desktop

# Installer les dépendances frontend
cd frontend && npm install && cd ..

# Build en mode développement
cd frontend && npm run dev &
cargo tauri dev

# Build release (produit .deb, .rpm, binaire)
cd frontend && npm run build && cd ..
cargo tauri build
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
│   └── meshcore-app/         # Application Tauri, 50 commandes IPC
└── frontend/                 # React 19, TypeScript, Tailwind CSS
    └── src/
        ├── views/            # 7 vues (Connection, Contacts, Chat, Map, Device, Repeater, Settings)
        ├── components/       # ElevationProfile, EmojiPicker, QrCode, etc.
        └── hooks/            # useEvents (temps réel), useTauri (IPC)
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

## Features

- **Encrypted messaging**: direct P2P messages and public/private channels
- **Mapping**: node visualization on OpenStreetMap, topographic or satellite tiles
- **Repeater admin**: admin login, CLI terminal, status, neighbours, radio configuration
- **Line-of-sight analysis**: SRTM elevation profile with Fresnel zone
- **Multi-connection**: BLE, USB serial and TCP simultaneously
- **Room Servers**: login, group chat, administration
- **GPX export**: waypoints for all mesh nodes
- **12 languages**: FR, EN, ES, DE, IT, PT, NL, PL, JA, ZH, KO, RU
- **OS notifications**: native alerts for incoming messages
- **Deep links**: `meshcore://` scheme to open contacts and channels
- **Dark/light theme**

## Download v0.1.0

| Platform | Download |
|---|---|
| Linux (Debian/Ubuntu) | [MeshCore.Desktop_0.1.0_amd64.deb](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_amd64.deb) |
| Linux (AppImage) | [MeshCore.Desktop_0.1.0_amd64.AppImage](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_amd64.AppImage) |
| Windows | [MeshCore.Desktop_0.1.0_x64-setup.exe](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_x64-setup.exe) |
| macOS (Apple Silicon) | [MeshCore.Desktop_0.1.0_aarch64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_aarch64.dmg) |
| macOS (Intel) | [MeshCore.Desktop_0.1.0_x64.dmg](https://github.com/mdphoto/meshcore-desktop/releases/download/v0.1.0/MeshCore.Desktop_0.1.0_x64.dmg) |

## Installation

### Linux

```bash
# Debian/Ubuntu
sudo dpkg -i MeshCore.Desktop_0.1.0_amd64.deb

# Or AppImage (no installation required)
chmod +x MeshCore.Desktop_0.1.0_amd64.AppImage
./MeshCore.Desktop_0.1.0_amd64.AppImage
```

### Windows

Run `MeshCore.Desktop_0.1.0_x64-setup.exe` and follow the installer.

### macOS

Open the `.dmg` and drag MeshCore Desktop to Applications.

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- Tauri CLI: `cargo install tauri-cli --version "^2"`

#### Linux (system dependencies)

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev pkg-config
```

### Build

```bash
git clone https://github.com/mdphoto/meshcore-desktop.git
cd meshcore-desktop

# Install frontend dependencies
cd frontend && npm install && cd ..

# Development mode
cd frontend && npm run dev &
cargo tauri dev

# Release build (produces .deb, .rpm, binary)
cd frontend && npm run build && cd ..
cargo tauri build
```

### Tests

```bash
# Rust tests (21 tests)
cargo test --workspace

# Frontend tests (41 tests)
cd frontend && npm test
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
│   └── meshcore-app/         # Tauri application, 50 IPC commands
└── frontend/                 # React 19, TypeScript, Tailwind CSS
    └── src/
        ├── views/            # 7 views (Connection, Contacts, Chat, Map, Device, Repeater, Settings)
        ├── components/       # ElevationProfile, EmojiPicker, QrCode, etc.
        └── hooks/            # useEvents (real-time), useTauri (IPC)
```

Built on top of [meshcore-rs](https://crates.io/crates/meshcore-rs) for the MeshCore protocol.

## Compatible hardware

All MeshCore-supported devices: Heltec, RAK Wireless, Seeed, nRF52.
Full list at [flasher.meshcore.co.uk](https://flasher.meshcore.co.uk/).

## License

MIT

## Author

Michel Dessenne — [IELOW SAS](https://ielow.fr)
