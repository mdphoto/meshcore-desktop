# meshcorex-tui

TUI (Terminal User Interface) [ratatui](https://ratatui.rs/) pour MeshCoreX.

Ce crate est une **library** — la TUI est embarquée dans le binaire `meshcorex` (crate `meshcorex-cli`).

## Invocation

```bash
meshcorex                             # TUI avec auto-reconnect
meshcorex --ble MeshCore-AB12         # force BLE
meshcorex --port /dev/ttyUSB0         # force série
meshcorex --tcp 192.168.1.50:4403     # force TCP
meshcorex --data-dir ~/meshcore-alt   # autre répertoire de données
```

Depuis l'extérieur (embarquer la TUI dans un autre binaire Rust) :

```rust
use meshcorex_tui::run_tui;
// tokio runtime déjà démarré
run_tui(state, opts).await?;
```

## Architecture

MVU (Model-View-Update) avec boucle `tokio::select!` :

```
crates/meshcorex-tui/src/
├── lib.rs              # entrypoint run_tui()
├── app.rs              # App struct, dispatcher Action → mutation d'état
├── action.rs           # enum Action, enum Tab, enum ModalKind
├── state/              # state structs (AppUiState, chat, connection, repeater, device)
├── events/input.rs     # crossterm KeyEvent → Action
├── actions/            # spawners tokio (contacts, channels, messaging, device, repeater)
├── ui/                 # rendu ratatui
│   ├── mod.rs          # frame::render dispatcher
│   ├── tab_bar.rs      # barre d'onglets
│   ├── status_bar.rs   # barre d'état bas
│   ├── tabs/           # rendu de chaque tab (connection, contacts, chat, channels, device, settings)
│   └── widgets/        # composants réutilisables (modal, chat_view, contact_info, etc.)
├── theme.rs            # palette couleurs / styles
└── util/
    ├── i18n.rs         # système de traduction FR/EN
    ├── format.rs       # short_pubkey, relative_time, node_type_*, OSC 52 clipboard
    └── unicode.rs      # truncate sans couper les graphèmes
```

Une event loop unique dans `App::run` multiplexe via `tokio::select!` :

1. Événements backend (`AppEvent` depuis `meshcorex-service`)
2. Entrées clavier (`crossterm::EventStream`)
3. Résultats de tâches async (`AsyncResult` → `Action::Async`)
4. Tick périodique (spinner, toasts TTL, sticky-bottom chat)

## Onglets

| Tab | Nom | Contenu |
|---|---|---|
| 1 | Connexion / Connection | Scan BLE (`s`), scan série (`s`), saisie TCP, liste des connexions actives |
| 2 | Contacts | Liste triable (favoris / type / nom), sections repliables par type, `i` info, `f` favori, `d` suppr, `R` admin repeater |
| 3 | Chat | Split conversations / historique / saisie, sticky-bottom, pagination PgUp, autocomplete `@mention`, OSC 52 copier |
| 4 | Canaux / Channels | CRUD, `n` nouveau, `e` éditer, `r` marquer lu, `s` sync, `d` suppr, hashtag rooms (SHA256(nom)[:16]) |
| 5 | Device | Infos radio/GPS/TX, jauge batterie, `n` renommer, `p` TX power, `t` sync heure, `a`/`A` advert, `b` batterie, `B` reboot |
| 6 | Paramètres / Settings | Langue FR/EN (cyclable via `←`/`→`), version, chemin DB, état connexion |

## Raccourcis clavier

### Navigation globale

| Touches | Action |
|---|---|
| `F1`-`F6` | Changer d'onglet (1-6) |
| `1`-`6` | Idem (sauf dans les inputs TCP/chat) |
| `Alt+1`-`Alt+6` | Idem (actif même dans les inputs) |
| `Tab` / `Shift-Tab` | Cycle focus dans l'onglet courant |
| `?` | Overlay d'aide |
| `q` | Quitter (sauf dans les inputs) |
| `Ctrl+C` | Quitter (toujours) |
| `Esc` | Ferme la modale active |

### Chat (tab 3)

| Touches | Action |
|---|---|
| `Enter` | Envoyer le message (ou ouvrir la conversation sélectionnée) |
| `PgUp` | Charger les messages plus anciens |
| `@` puis lettres | Autocomplete participants, validation par `Tab`/`Enter` |
| `Ctrl+A` / `Ctrl+E` | Début / fin de ligne (readline) |
| `Ctrl+W` | Supprimer le mot précédent |
| `Ctrl+U` | Effacer la saisie |

### Administration repeater (depuis tab 2)

| Touches | Action |
|---|---|
| `R` sur un contact repeater | Ouvre l'admin CLI plein-écran |
| `Tab` | Cycle entre panneaux (Statut / Voisins / ACL / CLI) |
| `r` | Rafraîchir le panneau actif |
| `c` | Bascule sur le panneau CLI texte |
| `L` | Logout |
| `help` (dans CLI) | Liste des commandes firmware |

## Plateformes

### Linux

- **x86_64** : build natif fonctionnel, testé Arch + Debian/Ubuntu
- **ARM64 (Pi 4/5)** : cross-compile via [cross](https://github.com/cross-rs/cross) :

```bash
cross build --release --target aarch64-unknown-linux-gnu -p meshcorex-cli
```

Build natif sur Raspberry Pi 3 (1 Go RAM) : insuffisant pour le linker. Utiliser cross.

### Windows

- Terminal recommandé : **Windows Terminal** (pas `cmd.exe`). Les bordures Unicode et les emojis sortent cassés sinon.
- BLE via `btleplug` + WinRT : conditionnel par `cfg(target_os = "windows")` pour éviter l'init BlueZ au démarrage.

### macOS

- Première connexion BLE : macOS demande l'autorisation *Confidentialité & sécurité → Bluetooth* pour Terminal.app / iTerm2.

## Internationalisation

Le système i18n est dans `src/util/i18n.rs` :

```rust
use crate::util::i18n::t;
let label = t("contacts.title");  // → "Contacts" (FR) ou "Contacts" (EN)
```

- Langue par défaut : FR
- Persistée via `settings::set(conn, "app.language", code)` dans la DB
- Changement immédiat (RwLock statique, lu à chaque appel `t()`)
- Fallback : langue courante → FR → clé brute

Pour ajouter une langue : étendre `enum Lang` + ajouter les paires `(Lang::X, "clé") => Some("...")` dans `translate()`.

## Notes d'implémentation

- **Curseur** : `Frame::set_cursor_position` pour un vrai curseur terminal (pas un `_` simulé)
- **OSC 52** : copie dans le presse-papier via escape sequence `\x1b]52;c;<base64>\x07`, fonctionne en SSH
- **Sticky-bottom** : si `scroll_offset == 0`, l'historique chat ancre automatiquement en bas aux nouveaux messages
- **Sender name** : le protocole MeshCore ne transporte pas d'identité sur les messages canal ; parsé depuis le préfixe "Nom: " du texte (blacklist : rejette `:`, `>`, control chars)
- **Room login** : attend la réponse serveur réelle (packets `LoginSuccess` 0x85 / `LoginFailed` 0x86) via `tokio::select!` avec timeout 30s

## Tests

```bash
cargo test -p meshcorex-tui
cargo clippy -p meshcorex-tui -- -D warnings
```

## Dépendances principales

- `ratatui` 0.29 + `crossterm` 0.28
- `tokio` (runtime async partagé)
- `tui-input` (widgets texte avec cursor management)
- `sha2` (dérivation PSK hashtag rooms)
- `chrono` (timestamps relatifs)

## Licence

MIT
