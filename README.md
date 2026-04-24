# MeshCoreX

Client cross-platform pour les réseaux mesh LoRa [MeshCore](https://meshcore.co.uk/).
Communiquez hors réseau via BLE, USB/série ou TCP avec vos dispositifs MeshCore.

**[English version below](#english)**

---

## 📢 Changement de cap à partir de v0.3.0 — TUI comme interface principale

Depuis la version **v0.3.0**, le projet bascule sur une **TUI (Terminal User Interface)** construite avec [ratatui](https://ratatui.rs/), distribuée sous forme d'un **binaire unique `meshcorex`** qui combine :

- Une **interface interactive complète** (la TUI, lancée par défaut)
- Un **mode one-shot** pour le scripting (sous-commandes `contacts`, `send`, `channels`, `device`, `battery`)
- L'**ancien REPL rustyline** (via `--repl`, pour compatibilité)

**Pourquoi ce choix ?** La GUI Tauri posait plusieurs problèmes bloquants :
- `webkit2gtk` n'est pas installé par défaut sur Arch Linux
- `webkit2gtk` ne se cross-compile pas depuis Linux → impossible de builder la GUI pour Raspberry Pi, Windows ou macOS sans un Mac / Windows physique
- Binaires GUI ~80 Mo vs ~7 Mo pour la TUI
- Cycle de release ~30 min vs ~5 min

La TUI fonctionne **partout** (Linux x86_64 et ARM64, Windows, macOS), via SSH, et utilise 100 % du backend existant.

### 🙏 Pour la GUI — contributions bienvenues

Le code de la GUI Tauri (`crates/meshcorex-app` + `frontend/`) **reste dans le repo** et continue à builder via le workflow `release-gui.yml` (manuel). Les fonctionnalités spécifiques à la GUI (cartographie OpenStreetMap, profil SRTM avec zone de Fresnel, 12 langues) ne sont **pas portées** à la TUI.

**Je n'ai pas le temps ni la capacité de maintenir la GUI pour le moment.** Si vous voulez reprendre cette partie du projet et la faire évoluer, vous êtes **très bienvenus** — ouvrez une issue ou une PR, on en discute. La base est propre, bien séparée en crates, 50 commandes IPC Tauri déjà branchées sur le backend Rust.

---

> **⚠️ Version beta — Testeurs bienvenus !**
>
> MeshCoreX est fonctionnel mais encore jeune. J'ai besoin de retours sur toutes les plateformes.
> Si vous rencontrez un problème, ouvrez une [issue](https://github.com/mdphoto/meshcorex/issues).
>
> | Plateforme | Binaire `meshcorex` (TUI + CLI) | BLE | USB/Série | TCP |
> |---|---|---|---|---|
> | Linux x86_64 | ✅ Testé | ✅ Testé | ❓ À tester | ❓ À tester |
> | Linux ARM64 (Pi 4/5) | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
> | Windows x64 | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
> | macOS Intel | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
> | macOS Apple Silicon | ❓ À tester | ❓ À tester | ❓ À tester | ❓ À tester |
>
> **Priorités de test :**
> 1. ❓ **Windows Terminal** : rendu TUI + emojis + BLE via WinRT
> 2. ❓ **macOS** : permissions Bluetooth + rendu TUI dans Terminal.app / iTerm2
> 3. ❓ **Raspberry Pi 4/5** : TUI fluide en SSH, BLE via BlueZ
> 4. ❓ **Connexions USB/Série et TCP** sur toutes les plateformes

## Fonctionnalités TUI (interface principale)

- **Tab 1 Connexion** : scan BLE, scan série, TCP, liste des connexions actives
- **Tab 2 Contacts** : liste avec 3 modes de tri (favoris, type, nom), sections repliables (Repeaters / Rooms / Clients / Sensors), sync, favoris, suppression, **modale info (`i`)** avec toutes les infos publiques (pubkey, GPS, last seen, chemin, flags)
- **Tab 3 Chat** : split conversations / historique scrollable / saisie, sticky-bottom, pagination, statuts envoi/livré/échoué, **autocomplete `@mention`**, **login rooms** avec mot de passe, copie OSC 52
- **Tab 4 Canaux** : CRUD, mark as read, sync vers device, **édition par canal** (nom, notifications, scope/région persistant), **création de hashtag rooms** (PSK auto-dérivée via `SHA256(nom)[:16]`)
- **Tab 5 Device** : infos radio/GPS/TX, jauge batterie, set name / tx power / heure / reboot / advert
- **Tab 6 Paramètres** : changement de langue **FR/EN** à chaud, version app, chemin DB, état connexion
- **Administration repeater** (touche `R` sur un contact repeater) : login, status, voisins, ACL, **CLI libre** avec aide `help` intégrée (commandes région, scope, firmware MeshCore ≥ 1.10)
- **Auto-reconnect** au dernier dispositif utilisé
- **Auto-sync contacts** à la connexion, avec spinner animé
- **Raccourcis robustes** : `F1`-`F6`, `1`-`6`, ou `Alt+1`-`Alt+6` (pour les terminaux qui captent les touches F)

## Téléchargement v0.3.0

Un seul binaire `meshcorex` par plateforme (TUI + CLI combinés).

| Plateforme | Binaire | Package |
|---|---|---|
| Linux x86_64 | [meshcorex_linux_x86_64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_linux_x86_64) | [meshcorex_0.3.0_amd64.deb](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_amd64.deb) |
| Linux ARM64 (Raspberry Pi 4/5) | [meshcorex_linux_arm64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_linux_arm64) | [meshcorex_0.3.0_arm64.deb](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_arm64.deb) |
| Windows x64 | [meshcorex_windows_x64.exe](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_windows_x64.exe) | — |
| macOS Intel | [meshcorex_macos_x64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_macos_x64) | — |
| macOS Apple Silicon | [meshcorex_macos_arm64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_macos_arm64) | — |

> **Binaire ~7 Mo**, statique (sauf libc / libdbus sur Linux), zéro install requise.

## Installation

### Linux

```bash
# Debian/Ubuntu/Pi OS
sudo apt install libdbus-1-3          # déjà présent sur la plupart des systèmes
sudo dpkg -i meshcorex_0.3.0_amd64.deb
meshcorex                               # lance la TUI

# Ou binaire brut
chmod +x meshcorex_linux_x86_64
./meshcorex_linux_x86_64
```

### Windows

1. Télécharger `meshcorex_windows_x64.exe`
2. **Utiliser Windows Terminal** (pré-installé sur Windows 11, gratuit depuis le Microsoft Store sur Windows 10). Le vieux `cmd.exe` / conhost ne gère pas correctement les emojis et les bordures Unicode
3. Double-cliquer ou lancer depuis PowerShell / Windows Terminal :

```powershell
.\meshcorex_windows_x64.exe
```

### macOS

```bash
chmod +x meshcorex_macos_arm64   # ou meshcorex_macos_x64 pour Intel
./meshcorex_macos_arm64
```

> **Permission Bluetooth** : à la première connexion BLE, macOS demandera l'autorisation Bluetooth. Autoriser le terminal (Terminal.app ou iTerm2) dans *Réglages Système → Confidentialité & sécurité → Bluetooth*.

### Raspberry Pi (ARM64)

```bash
wget https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_arm64.deb
sudo dpkg -i meshcorex_0.3.0_arm64.deb
meshcorex
```

## Utilisation

### Mode interactif (TUI)

```bash
meshcorex                               # lance la TUI avec auto-reconnect
meshcorex --ble MeshCore-AB12           # force une connexion BLE précise
meshcorex --port /dev/ttyUSB0           # force une connexion série
meshcorex --tcp 192.168.1.50:4403       # force une connexion TCP
```

Raccourcis principaux (la touche `?` affiche l'aide complète dans la TUI) :

| Touches | Action |
|---|---|
| `F1`-`F6` ou `1`-`6` ou `Alt+1`-`Alt+6` | Changer d'onglet |
| `Tab` / `Shift-Tab` | Cycle le focus dans l'onglet |
| `?` | Aide contextuelle |
| `q` ou `Ctrl-C` | Quitter |

### Mode one-shot (scripting)

```bash
meshcorex --port /dev/ttyUSB0 contacts list
meshcorex --tcp 192.168.1.50:4403 send Michel "Hello mesh !"
meshcorex --port /dev/ttyUSB0 --json device
meshcorex --ble MeshCore-AB12 battery lipo
```

### Mode REPL legacy (rustyline)

```bash
meshcorex --repl                        # retrouve l'ancien comportement
```

### Options globales

| Option | Description |
|---|---|
| `-p, --port <PORT>` | Port série (ex: `/dev/ttyUSB0`) |
| `-b, --baud <BAUD>` | Baud rate (défaut: 115200) |
| `--tcp <HOST:PORT>` | Connexion TCP |
| `--ble <NOM>` | Connexion BLE par nom (ex: `MeshCore-AB12`) |
| `--data-dir <DIR>` | Répertoire de données (défaut: `~/.local/share/meshcorex/`) |
| `--json` | Sortie JSON (mode one-shot uniquement) |
| `-v, --verbose` | Logs de debug |
| `--repl` | Ancien REPL rustyline au lieu de la TUI |

## Compilation depuis les sources

### Prérequis

- [Rust](https://rustup.rs/) stable
- Linux : `sudo apt install libdbus-1-dev pkg-config`

### Build binaire `meshcorex` (TUI + CLI)

```bash
git clone https://github.com/mdphoto/meshcorex.git
cd meshcorex
cargo build --release -p meshcorex-cli
./target/release/meshcorex
```

### Cross-compilation (depuis Linux)

```bash
# Installer cross
cargo install cross --git https://github.com/cross-rs/cross --locked

# ARM64 (Raspberry Pi 4/5)
cross build --release --target aarch64-unknown-linux-gnu -p meshcorex-cli

# Windows (MinGW)
cross build --release --target x86_64-pc-windows-gnu -p meshcorex-cli
```

macOS nécessite une machine macOS ou un runner GitHub Actions.

### Tests

```bash
cargo test --workspace --exclude meshcorex-app
```

### GUI legacy (optionnel, cherche contributeurs)

Le code de la GUI Tauri reste dans `crates/meshcorex-app/` + `frontend/`. Pour la builder :

```bash
# Deps Linux supplémentaires pour Tauri
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Frontend
cd frontend && npm install && npm run build && cd ..

# GUI
cargo install tauri-cli --version "^2"
cargo tauri build
```

Ou via le workflow GitHub Actions manuel `release-gui.yml` (onglet Actions → Run workflow).

## Architecture

```
meshcorex/
├── crates/
│   ├── meshcorex-protocol/    # Types, compression SMAZ, canaux
│   ├── meshcorex-crypto/      # AES-128-ECB, HMAC-SHA256, Ed25519
│   ├── meshcorex-transport/   # BLE, Serial, TCP, reconnexion auto
│   ├── meshcorex-storage/     # SQLite (contacts, messages, canaux, settings)
│   ├── meshcorex-service/     # Logique métier, état, événements
│   ├── meshcorex-tui/         # TUI ratatui (library, entrypoint run_tui)
│   ├── meshcorex-cli/         # Binaire unifié « meshcorex » (TUI + CLI + REPL)
│   └── meshcorex-app/         # GUI Tauri legacy (maintenance communautaire)
└── frontend/                 # React 19, TS, Tailwind (GUI legacy)
```

Basé sur [meshcore-rs](https://crates.io/crates/meshcore-rs) pour le protocole MeshCore.

## Matériel compatible

Tous les dispositifs supportés par MeshCore : Heltec, RAK Wireless, Seeed, nRF52.
Liste complète sur [flasher.meshcore.co.uk](https://flasher.meshcore.co.uk/).

## Contribuer

- **GUI Tauri** : les contributions sont très bienvenues sur la partie GUI — je n'ai pas le temps de la maintenir actuellement
- **TUI / CLI** : ouvrir une issue ou une PR, toute aide est appréciée
- **Tests multi-plateformes** : tout retour sur Windows / macOS / Raspberry Pi est précieux

## Licence

MIT

## Auteur

Michel Dessenne — [IELOW SAS](https://ielow.fr)

---

<a id="english"></a>

# MeshCoreX (English)

Cross-platform client for [MeshCore](https://meshcore.co.uk/) LoRa mesh networks.
Communicate off-grid via BLE, USB/serial or TCP with your MeshCore devices.

## 📢 Pivot in v0.3.0 — TUI as the main interface

Starting from **v0.3.0**, the project pivots to a **TUI (Terminal User Interface)** built with [ratatui](https://ratatui.rs/), shipped as a **single binary `meshcorex`** that combines:

- A **full interactive interface** (the TUI, launched by default)
- A **one-shot mode** for scripting (subcommands `contacts`, `send`, `channels`, `device`, `battery`)
- The **legacy rustyline REPL** (via `--repl`, kept for compatibility)

**Why this pivot?** The Tauri GUI had several blocking issues:
- `webkit2gtk` not installed by default on Arch Linux
- `webkit2gtk` cannot be cross-compiled from Linux → no GUI builds for Raspberry Pi, Windows or macOS without a physical Mac / Windows
- GUI binaries ~80 MB vs ~7 MB for the TUI
- Release cycle ~30 min vs ~5 min

The TUI works **everywhere** (Linux x86_64 and ARM64, Windows, macOS), over SSH, and reuses 100% of the existing backend.

### 🙏 GUI — contributions welcome

The Tauri GUI code (`crates/meshcorex-app` + `frontend/`) **stays in the repo** and still builds via the manual `release-gui.yml` workflow. GUI-specific features (OpenStreetMap, SRTM elevation profile with Fresnel zone, 12 languages) are **not ported** to the TUI.

**I don't have the time or capacity to maintain the GUI right now.** If you want to pick up this part of the project and evolve it, you are **very welcome** — open an issue or a PR. The codebase is clean, properly split into crates, and 50 Tauri IPC commands are already wired up to the Rust backend.

---

## Main TUI features

- **Tab 1 Connection**: BLE scan, serial scan, TCP input, list of active connections
- **Tab 2 Contacts**: list with 3 sort modes (favorites, type, name), collapsible sections (Repeaters / Rooms / Clients / Sensors), sync, favorites, delete, **info modal (`i`)** with all public info (pubkey, GPS, last seen, path, flags)
- **Tab 3 Chat**: split conversations / scrollable history / input, sticky-bottom, pagination, sent/delivered/failed indicators, **`@mention` autocomplete**, **room login** with password, OSC 52 copy
- **Tab 4 Channels**: CRUD, mark as read, sync to device, **per-channel editing** (name, notifications, persistent scope/region), **hashtag rooms creation** (PSK auto-derived via `SHA256(name)[:16]`)
- **Tab 5 Device**: radio/GPS/TX info, battery gauge, set name / tx power / time / reboot / advert
- **Tab 6 Settings**: hot-swap UI language **FR/EN**, app version, DB path, connection state
- **Repeater admin** (press `R` on a repeater contact): login, status, neighbours, ACL, **free CLI** with built-in `help` (region, scope, MeshCore firmware ≥ 1.10)
- **Auto-reconnect** to the last device used
- **Auto-sync contacts** on connection, with animated spinner
- **Robust keyboard shortcuts**: `F1`-`F6`, `1`-`6`, or `Alt+1`-`Alt+6` (for terminals that grab the F keys)

## Download v0.3.0

One `meshcorex` binary per platform (TUI + CLI combined).

| Platform | Binary | Package |
|---|---|---|
| Linux x86_64 | [meshcorex_linux_x86_64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_linux_x86_64) | [meshcorex_0.3.0_amd64.deb](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_amd64.deb) |
| Linux ARM64 (Raspberry Pi 4/5) | [meshcorex_linux_arm64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_linux_arm64) | [meshcorex_0.3.0_arm64.deb](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_arm64.deb) |
| Windows x64 | [meshcorex_windows_x64.exe](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_windows_x64.exe) | — |
| macOS Intel | [meshcorex_macos_x64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_macos_x64) | — |
| macOS Apple Silicon | [meshcorex_macos_arm64](https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_macos_arm64) | — |

> **~7 MB binary**, mostly static, zero install required.

## Installation

### Linux

```bash
sudo apt install libdbus-1-3
sudo dpkg -i meshcorex_0.3.0_amd64.deb
meshcorex                               # launch the TUI
```

### Windows

1. Download `meshcorex_windows_x64.exe`
2. **Use Windows Terminal** (preinstalled on Windows 11, free in Microsoft Store on Windows 10). Old `cmd.exe` / conhost does not render emojis and Unicode borders well
3. Double-click or run from PowerShell / Windows Terminal:

```powershell
.\meshcorex_windows_x64.exe
```

### macOS

```bash
chmod +x meshcorex_macos_arm64
./meshcorex_macos_arm64
```

> **Bluetooth permission**: on first BLE connection, macOS will ask for Bluetooth permission. Authorize the terminal (Terminal.app or iTerm2) in *System Settings → Privacy & Security → Bluetooth*.

### Raspberry Pi (ARM64)

```bash
wget https://github.com/mdphoto/meshcorex/releases/download/v0.3.0/meshcorex_0.3.0_arm64.deb
sudo dpkg -i meshcorex_0.3.0_arm64.deb
meshcorex
```

## Usage

### Interactive mode (TUI)

```bash
meshcorex                               # launches the TUI with auto-reconnect
meshcorex --ble MeshCore-AB12
meshcorex --port /dev/ttyUSB0
meshcorex --tcp 192.168.1.50:4403
```

Main shortcuts (press `?` for full help in the TUI):

| Keys | Action |
|---|---|
| `F1`-`F6` or `1`-`6` or `Alt+1`-`Alt+6` | Switch tab |
| `Tab` / `Shift-Tab` | Cycle focus within a tab |
| `?` | Contextual help |
| `q` or `Ctrl-C` | Quit |

### One-shot mode (scripting)

```bash
meshcorex --port /dev/ttyUSB0 contacts list
meshcorex --tcp 192.168.1.50:4403 send Michel "Hello mesh!"
meshcorex --port /dev/ttyUSB0 --json device
```

### Legacy REPL mode

```bash
meshcorex --repl                        # old rustyline REPL
```

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) stable
- Linux: `sudo apt install libdbus-1-dev pkg-config`

### Build `meshcorex` binary (TUI + CLI)

```bash
git clone https://github.com/mdphoto/meshcorex.git
cd meshcorex
cargo build --release -p meshcorex-cli
./target/release/meshcorex
```

### Cross-compilation (from Linux)

```bash
cargo install cross --git https://github.com/cross-rs/cross --locked
cross build --release --target aarch64-unknown-linux-gnu -p meshcorex-cli
cross build --release --target x86_64-pc-windows-gnu -p meshcorex-cli
```

macOS requires an actual macOS machine or GitHub Actions runner.

### Legacy GUI (optional, looking for maintainers)

The Tauri GUI code stays in `crates/meshcorex-app/` + `frontend/`. To build it:

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
cd frontend && npm install && npm run build && cd ..
cargo install tauri-cli --version "^2"
cargo tauri build
```

Or via the manual `release-gui.yml` GitHub Actions workflow.

## Architecture

```
meshcorex/
├── crates/
│   ├── meshcorex-protocol/    # Types, SMAZ compression, channels
│   ├── meshcorex-crypto/      # AES-128-ECB, HMAC-SHA256, Ed25519
│   ├── meshcorex-transport/   # BLE, Serial, TCP, auto-reconnect
│   ├── meshcorex-storage/     # SQLite (contacts, messages, channels, settings)
│   ├── meshcorex-service/     # Business logic, state, events
│   ├── meshcorex-tui/         # ratatui TUI (library, entrypoint run_tui)
│   ├── meshcorex-cli/         # Unified binary « meshcorex » (TUI + CLI + REPL)
│   └── meshcorex-app/         # Legacy Tauri GUI (community-maintained)
└── frontend/                 # React 19, TS, Tailwind (legacy GUI)
```

Built on top of [meshcore-rs](https://crates.io/crates/meshcore-rs) for the MeshCore protocol.

## Compatible hardware

All MeshCore-supported devices: Heltec, RAK Wireless, Seeed, nRF52.
Full list at [flasher.meshcore.co.uk](https://flasher.meshcore.co.uk/).

## Contributing

- **Tauri GUI**: contributions very welcome on the GUI side — I don't have bandwidth to maintain it right now
- **TUI / CLI**: open an issue or PR, any help appreciated
- **Multi-platform testing**: feedback on Windows / macOS / Raspberry Pi is very valuable

## License

MIT

## Author

Michel Dessenne — [IELOW SAS](https://ielow.fr)
