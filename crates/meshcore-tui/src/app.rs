use crate::action::{
    Action, AsyncResult, ConnectionSubPane, ModalKind, Tab, ToastLevel,
};
use crate::actions::contacts::ContactRow;
use crate::actions::{
    channels as channel_actions, companions as companion_actions, connection as conn_actions,
    contacts as contact_actions, device as device_actions, messaging as messaging_actions,
    repeater as repeater_actions,
};
use crate::state::chat::{ChatFocus, ConversationId, ConversationSummary, MAX_IN_MEMORY};
use crate::state::device::DeviceUiState;
use crate::events::input;
use crate::state::chat::ChatUiState;
use crate::state::connection::ConnectionUiState;
use crate::state::repeater::RepeaterUiState;
use crate::state::{AppUiState, FocusTarget};
use crate::tui::Tui;
use crate::ui;
use anyhow::Result;
use crossterm::event::EventStream;
use futures::StreamExt;
use meshcore_service::{AppEvent, AppState};
use meshcore_storage::models::StoredContact;
use meshcore_transport::manager::ConnectionTarget;
use ratatui::widgets::ListState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

fn cli_help_lines() -> &'static [&'static str] {
    &[
        "Aide — commandes repeater MeshCore (extraites du projet) :",
        "",
        "  Informations (lecture) :",
        "    ver                              Version du firmware",
        "    get name                         Nom courant",
        "    get radio                        Fréquence / BW / SF / CR",
        "    get tx                           Puissance TX actuelle (dBm)",
        "    clock                            Heure courante du repeater",
        "    neighbors                        Lister les voisins radio",
        "    advert                           Envoyer un advert simple",
        "    clear stats                      Réinitialiser les stats radio",
        "",
        "  Identité / accès :",
        "    set name MonRepeater             Changer le nom (advert émis)",
        "    password NouveauMDP              Changer le mot de passe admin",
        "    set guest.password hello         Mot de passe invité",
        "",
        "  Radio :",
        "    set radio 869.525,125,9,5        Fréq(MHz),BW(kHz),SF,CR",
        "    set tx 20                        Puissance TX en dBm",
        "",
        "  Localisation / GPS :",
        "    set lat 44.7500                  Latitude",
        "    set lon 4.8500                   Longitude",
        "    gps on                           Activer le GPS",
        "    gps off                          Désactiver le GPS",
        "    gps sync                         Synchro position",
        "",
        "  Comportement :",
        "    set repeat on / off              Activer / désactiver la répétition",
        "    set flood.max 6                  Max hops pour les floods",
        "    set advert.interval 120          Intervalle advert (min)",
        "    set flood.advert.interval 12     Intervalle flood advert (h)",
        "    set multi.acks 1                 Multi-acks (0/1)",
        "    set allow.read.only on           Mode lecture seule",
        "    set adc.multiplier 1.0           Multiplicateur batterie",
        "",
        "  Régions / zones géographiques (firmware ≥ 1.10.0) :",
        "    region                           Lister les régions définies + permissions",
        "    region home                      Afficher la région 'home' actuelle",
        "    region home us-ca                Définir la région 'home'",
        "    region get us                    Info d'une région (ou '*' pour toutes)",
        "    region put us-ca us              Ajouter us-ca comme enfant de us",
        "    region put us *                  Ajouter us sans parent",
        "    region remove us-ca              Supprimer une région (match exact)",
        "    region allowf us-ca              Autoriser le flood pour cette région",
        "    region denyf us-ca               Refuser le flood pour cette région",
        "    region default us-ca             Scope par défaut du nœud (firmware ≥ 1.15)",
        "    region default <null>            Réinitialiser le scope par défaut",
        "    region list allowed              Régions autorisées (firmware ≥ 1.12)",
        "    region list denied               Régions bloquées (firmware ≥ 1.12)",
        "    region save                      Persister la config en flash",
        "",
        "  Scope par message / canal (runtime, pas de persistance) :",
        "    scope us-ca                      Bascule le scope de diffusion courant",
        "    to <dest>%us-ca                  Envoi DM limité à un scope",
        "    chan 0%us-ca bonjour             Msg canal 0 limité au scope us-ca",
        "    login%#Morbihan                  Cmd login limitée à une région",
        "    (le suffixe %* = broadcast classique sans limite)",
        "",
        "  Logs :",
        "    log start                        Démarrer la capture de logs",
        "    log stop                         Arrêter la capture",
        "    log erase                        Effacer le buffer de logs",
        "",
        "  Maintenance :",
        "    set dfu on                       Mode mise à jour firmware",
        "    reboot                           Redémarrer le repeater",
        "",
        "  Local (TUI, non envoyé au repeater) :",
        "    help, ?                          Afficher cette aide",
        "    clear, cls                       Effacer la sortie du terminal",
        "",
        "Sources : frontend/src/views/RepeaterView.tsx (local) + wiki MeshCore.",
        "Doc régions : github.com/meshcore-dev/MeshCore/wiki — section Region.",
        "Si une commande renvoie « unknown », elle n'est pas supportée par votre firmware.",
    ]
}

fn find_header_index(rows: &[ContactRow], node_type: u8) -> Option<usize> {
    rows.iter().position(|r| {
        matches!(r, ContactRow::Header { node_type: n, .. } if *n == node_type)
    })
}

fn cycle_sub_pane(current: &ConnectionSubPane, forward: bool) -> ConnectionSubPane {
    let order = [
        ConnectionSubPane::BleScan,
        ConnectionSubPane::SerialList,
        ConnectionSubPane::TcpInput,
        ConnectionSubPane::Active,
    ];
    let idx = order
        .iter()
        .position(|p| std::mem::discriminant(p) == std::mem::discriminant(current))
        .unwrap_or(0);
    let len = order.len();
    let next_idx = if forward {
        (idx + 1) % len
    } else {
        (idx + len - 1) % len
    };
    order[next_idx].clone()
}

pub struct App {
    pub service: Arc<AppState>,
    pub ui: AppUiState,
    pub connection_ui: ConnectionUiState,
    pub chat_ui: ChatUiState,
    pub repeater_ui: RepeaterUiState,
    pub device_ui: DeviceUiState,
    pub contacts: Vec<StoredContact>,
    pub contact_rows: Vec<ContactRow>,
    pub contacts_list_state: ListState,
    pub channels: Vec<meshcore_storage::channels::StoredChannel>,
    pub channels_list_state: ListState,
    /// Scope par canal persistant (stocké dans settings avec clé "channel.scope.N")
    pub channel_scopes: std::collections::HashMap<u8, String>,
    /// Pubkeys des contacts ayant des messages DM en DB, triées par dernier message (desc).
    /// Chargé depuis la DB au démarrage et après chaque Connected/réception — permet
    /// de lister dans la tab Chat les conversations existantes sans clic préalable.
    pub dm_pubkeys: Vec<String>,
    pub action_tx: UnboundedSender<Action>,
    pub auto_reconnect_on_startup: bool,
}

impl App {
    pub fn new(service: Arc<AppState>, action_tx: UnboundedSender<Action>) -> Self {
        let mut contacts_list_state = ListState::default();
        contacts_list_state.select(Some(0));
        let mut channels_list_state = ListState::default();
        channels_list_state.select(Some(0));
        let mut chat_ui = ChatUiState::new();
        chat_ui.conversations_list_state.select(Some(0));
        Self {
            service,
            ui: AppUiState::new(),
            connection_ui: ConnectionUiState::new(),
            chat_ui,
            repeater_ui: RepeaterUiState::new(),
            device_ui: DeviceUiState::new(),
            contacts: Vec::new(),
            contact_rows: Vec::new(),
            contacts_list_state,
            channels: Vec::new(),
            channels_list_state,
            channel_scopes: std::collections::HashMap::new(),
            dm_pubkeys: Vec::new(),
            action_tx,
            auto_reconnect_on_startup: false,
        }
    }

    fn spawn_auto_reconnect(&self) {
        let svc = self.service.clone();
        let tx = self.action_tx.clone();
        tokio::spawn(async move {
            // D'abord : est-on déjà connecté ? (cas CLI avec --ble réussi ailleurs)
            let already_connected = {
                let conn = svc.connection.read().await;
                conn.connection_count() > 0
            };
            if already_connected {
                return;
            }

            // Lister les companions
            let companions = svc
                .db
                .with_conn(meshcore_storage::companions::get_all_companions);
            let companions = match companions {
                Ok(list) => list,
                Err(e) => {
                    let _ = tx.send(Action::Toast(
                        format!("DB companions : {}", e),
                        ToastLevel::Error,
                    ));
                    return;
                }
            };

            let Some(last) = companions.first() else {
                let _ = tx.send(Action::Toast(
                    "Aucun device connu — scan BLE manuellement".into(),
                    ToastLevel::Info,
                ));
                return;
            };

            let Some(target) =
                crate::actions::companions::target_from_companion(last)
            else {
                let _ = tx.send(Action::Toast(
                    format!(
                        "Companion « {} » type transport inconnu",
                        last.name
                    ),
                    ToastLevel::Warn,
                ));
                return;
            };

            let _ = tx.send(Action::Toast(
                format!("Reconnexion à {}…", last.name),
                ToastLevel::Info,
            ));

            match meshcore_service::connection::connect(&svc, target).await {
                Ok(()) => {
                    // AppEvent::Connected arrivera via broadcast
                }
                Err(e) => {
                    let _ = tx.send(Action::Toast(
                        format!("Auto-reconnexion échouée : {}", e),
                        ToastLevel::Warn,
                    ));
                }
            }
        });
    }

    /// Reconstruit la liste des rows affichées selon le mode de tri et les groupes repliés.
    /// Clamp la sélection si elle dépasse la nouvelle taille.
    pub fn rebuild_contact_rows(&mut self) {
        self.contact_rows = contact_actions::build_rows(
            &self.contacts,
            self.ui.contacts_sort,
            &self.ui.contacts_collapsed_groups,
        );
        let len = self.contact_rows.len();
        let new_sel = match self.contacts_list_state.selected() {
            None if len > 0 => Some(0),
            Some(i) if i >= len => {
                if len == 0 {
                    None
                } else {
                    Some(len - 1)
                }
            }
            other => other,
        };
        self.contacts_list_state.select(new_sel);
    }

    pub async fn run(
        mut self,
        tui: &mut Tui,
        mut action_rx: UnboundedReceiver<Action>,
    ) -> Result<()> {
        let mut input_stream = EventStream::new();
        let mut backend_rx = self.service.subscribe();

        // Premier refresh
        contact_actions::reload(self.service.clone(), self.action_tx.clone());
        channel_actions::reload(self.service.clone(), self.action_tx.clone());
        messaging_actions::reload_dm_pubkeys(self.service.clone(), self.action_tx.clone());
        conn_actions::refresh_list(self.service.clone(), self.action_tx.clone());

        // Auto-reconnect au dernier companion (si demandé et aucune connexion active)
        if self.auto_reconnect_on_startup {
            self.spawn_auto_reconnect();
        }

        let mut tick = tokio::time::interval(Duration::from_millis(150));

        tui.terminal.draw(|f| ui::render(f, &self))?;

        loop {
            tokio::select! {
                maybe_event = input_stream.next() => {
                    if let Some(Ok(event)) = maybe_event {
                        let action = input::map_event_with_repeater(
                            event,
                            &self.ui,
                            &self.repeater_ui,
                        );
                        self.dispatch(action);
                    }
                }
                Ok(event) = backend_rx.recv() => {
                    self.dispatch(Action::Backend(event));
                }
                Some(action) = action_rx.recv() => {
                    self.dispatch(action);
                }
                _ = tick.tick() => {
                    self.ui.prune_toasts();
                    self.tick_receiving();
                }
            }

            if self.ui.should_quit {
                break;
            }
            tui.terminal.draw(|f| ui::render(f, &self))?;
        }
        Ok(())
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::NoOp | Action::Tick | Action::ToastTick => {}
            Action::Quit => self.ui.should_quit = true,

            Action::NextTab => {
                let idx = (self.ui.current_tab.index() + 1) % 5;
                self.goto_tab(Tab::from_index(idx));
            }
            Action::PrevTab => {
                let idx = (self.ui.current_tab.index() + 4) % 5;
                self.goto_tab(Tab::from_index(idx));
            }
            Action::GotoTab(tab) => self.goto_tab(tab),

            Action::FocusNext => self.cycle_focus(true),
            Action::FocusPrev => self.cycle_focus(false),

            Action::OpenModal(kind) => self.ui.push_modal(kind),
            Action::CloseModal => self.ui.pop_modal(),

            Action::Toast(msg, level) => self.ui.toast(msg, level),

            // --- Contacts ---
            Action::ContactsSync => {
                if !self.ui.connected {
                    self.ui
                        .toast("Non connecté — impossible de sync", ToastLevel::Warn);
                } else if !self.ui.contacts_syncing {
                    self.start_contacts_sync();
                }
            }
            Action::ContactsRefresh => {
                contact_actions::reload(self.service.clone(), self.action_tx.clone());
            }
            Action::ContactsToggleFav => {
                if let Some(c) = self.selected_contact().cloned() {
                    contact_actions::spawn_toggle_fav(
                        self.service.clone(),
                        c.public_key,
                        !c.is_favorite,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::ContactsRequestDelete => {
                if let Some(c) = self.selected_contact().cloned() {
                    self.ui.push_modal(ModalKind::ConfirmDeleteContact {
                        pubkey: c.public_key,
                        name: c.name,
                    });
                }
            }
            Action::ContactsConfirmDelete(pubkey) => {
                self.ui.pop_modal();
                contact_actions::spawn_delete(
                    self.service.clone(),
                    pubkey,
                    self.action_tx.clone(),
                );
            }
            // --- Chat ---
            Action::ChatSelectPrev => self.chat_move_selection(-1),
            Action::ChatSelectNext => self.chat_move_selection(1),
            Action::ChatOpenSelected => self.chat_open_selected(),
            Action::ChatFocusNext => self.chat_cycle_focus(),
            Action::ChatInputChar(c) => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::InsertChar(c));
            }
            Action::ChatInputBackspace => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::DeletePrevChar);
            }
            Action::ChatInputDelete => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::DeleteNextChar);
            }
            Action::ChatInputLeft => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::GoToPrevChar);
            }
            Action::ChatInputRight => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::GoToNextChar);
            }
            Action::ChatInputHome => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::GoToStart);
            }
            Action::ChatInputEnd => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::GoToEnd);
            }
            Action::ChatInputDeletePrevWord => {
                use tui_input::InputRequest;
                self.chat_ui.input.handle(InputRequest::DeletePrevWord);
            }
            Action::ChatInputClear => {
                self.chat_ui.input.reset();
            }
            Action::ChatSend => self.chat_send_current(),
            Action::ChatScrollUp => {
                self.chat_ui.scroll_offset = self.chat_ui.scroll_offset.saturating_add(1);
            }
            Action::ChatScrollDown => {
                self.chat_ui.scroll_offset = self.chat_ui.scroll_offset.saturating_sub(1);
            }
            Action::ChatLoadOlder => self.chat_load_older(),
            Action::ChatRefreshConversations => {
                channel_actions::reload(self.service.clone(), self.action_tx.clone());
                contact_actions::reload(self.service.clone(), self.action_tx.clone());
            }
            Action::ChatOpenContact(pk) => self.chat_activate(ConversationId::Dm(pk)),

            // --- Channels ---
            Action::ChannelsRefresh => {
                channel_actions::reload(self.service.clone(), self.action_tx.clone());
            }
            Action::ChannelsMarkRead => {
                if let Some(c) = self.selected_channel().cloned() {
                    channel_actions::spawn_mark_read(
                        self.service.clone(),
                        c.idx,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::ChannelsSyncToDevice => {
                if let Some(c) = self.selected_channel().cloned() {
                    channel_actions::spawn_sync_to_device(
                        self.service.clone(),
                        c,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::ChannelsRequestDelete => {
                if let Some(c) = self.selected_channel().cloned() {
                    self.ui.push_modal(ModalKind::ConfirmDeleteChannel {
                        idx: c.idx,
                        name: c.name,
                    });
                }
            }
            Action::ChannelsConfirmDelete(idx) => {
                self.ui.pop_modal();
                channel_actions::spawn_delete(
                    self.service.clone(),
                    idx,
                    self.action_tx.clone(),
                );
            }
            Action::ChannelsRequestEdit => {
                if let Some(c) = self.selected_channel().cloned() {
                    self.ui.channel_edit_name = c.name.clone();
                    self.ui.channel_edit_notifications = c.notifications_enabled;
                    self.ui.channel_edit_scope = self
                        .channel_scopes
                        .get(&c.idx)
                        .cloned()
                        .unwrap_or_default();
                    self.ui.channel_edit_psk_hex =
                        c.psk.iter().map(|b| format!("{:02x}", b)).collect();
                    self.ui.channel_edit_field = 0;
                    self.ui.push_modal(ModalKind::ChannelEdit { idx: c.idx });
                }
            }
            Action::ChannelsEditCopyPsk => {
                let psk_hex = self.ui.channel_edit_psk_hex.clone();
                if psk_hex.is_empty() {
                    self.ui.toast("Aucun PSK à copier", ToastLevel::Warn);
                } else {
                    match crate::util::format::copy_to_clipboard(&psk_hex) {
                        Ok(()) => {
                            self.ui.toast(
                                "PSK copié dans le presse-papier (OSC 52)",
                                ToastLevel::Success,
                            );
                        }
                        Err(e) => {
                            self.ui.toast(
                                format!("Échec copie : {}", e),
                                ToastLevel::Error,
                            );
                        }
                    }
                }
            }
            Action::ChannelsEditNameChar(c) => {
                match self.ui.channel_edit_field {
                    0 => self.ui.channel_edit_name.push(c),
                    2 => self.ui.channel_edit_scope.push(c),
                    _ => {}
                }
            }
            Action::ChannelsEditNameBackspace => {
                match self.ui.channel_edit_field {
                    0 => {
                        self.ui.channel_edit_name.pop();
                    }
                    2 => {
                        self.ui.channel_edit_scope.pop();
                    }
                    _ => {}
                }
            }
            Action::ChannelsEditToggleNotifications => {
                // Bascule la case à cocher — n'agit que si on est sur le champ Notifications
                if self.ui.channel_edit_field == 1 {
                    self.ui.channel_edit_notifications =
                        !self.ui.channel_edit_notifications;
                }
            }
            Action::ChannelsEditNextField => {
                self.ui.channel_edit_field = (self.ui.channel_edit_field + 1) % 3;
            }
            Action::ChannelsEditPrevField => {
                self.ui.channel_edit_field = (self.ui.channel_edit_field + 2) % 3;
            }
            Action::ChannelsEditSubmit => {
                self.submit_channel_edit(false);
            }
            Action::ChannelsEditSyncAndSubmit => {
                self.submit_channel_edit(true);
            }
            Action::ChannelsRequestNew => {
                if self.find_free_channel_idx().is_none() {
                    self.ui.toast(
                        "Aucun slot libre (max 8 canaux 0..7)",
                        ToastLevel::Warn,
                    );
                } else {
                    self.ui.channel_new_name.clear();
                    self.ui.channel_new_psk_hex.clear();
                    self.ui.channel_new_field = 0;
                    self.ui.push_modal(ModalKind::ChannelNew);
                }
            }
            Action::ChannelsNewChar(c) => {
                match self.ui.channel_new_field {
                    0 => self.ui.channel_new_name.push(c),
                    1 if c.is_ascii_hexdigit()
                        && self.ui.channel_new_psk_hex.len() < 32 =>
                    {
                        // PSK : uniquement hex, limité à 32 caractères (16 octets)
                        self.ui.channel_new_psk_hex.push(c.to_ascii_lowercase());
                    }
                    _ => {}
                }
            }
            Action::ChannelsNewBackspace => {
                match self.ui.channel_new_field {
                    0 => {
                        self.ui.channel_new_name.pop();
                    }
                    1 => {
                        self.ui.channel_new_psk_hex.pop();
                    }
                    _ => {}
                }
            }
            Action::ChannelsNewNextField => {
                self.ui.channel_new_field = (self.ui.channel_new_field + 1) % 2;
            }
            Action::ChannelsNewPrevField => {
                self.ui.channel_new_field = (self.ui.channel_new_field + 1) % 2;
            }
            Action::ChannelsNewGeneratePsk => {
                use rand::RngCore;
                let mut psk = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut psk);
                self.ui.channel_new_psk_hex =
                    psk.iter().map(|b| format!("{:02x}", b)).collect();
                self.ui.toast("PSK aléatoire généré", ToastLevel::Info);
            }
            Action::ChannelsNewDeriveFromName => {
                let name = self.ui.channel_new_name.trim().to_string();
                if name.is_empty() {
                    self.ui.toast(
                        "Saisir d'abord le nom (ex: #meteo)",
                        ToastLevel::Warn,
                    );
                } else {
                    let psk = crate::util::format::derive_hashtag_psk(&name);
                    self.ui.channel_new_psk_hex =
                        psk.iter().map(|b| format!("{:02x}", b)).collect();
                    self.ui.toast(
                        format!("PSK dérivé de « {} » (SHA256[:16])", name),
                        ToastLevel::Info,
                    );
                }
            }
            Action::ChannelsNewSubmit => {
                self.submit_channel_new();
            }

            // --- Device ---
            Action::DeviceRefresh => {
                if self.ui.connected {
                    device_actions::spawn_refresh_info(
                        self.service.clone(),
                        self.action_tx.clone(),
                    );
                    device_actions::spawn_battery(
                        self.service.clone(),
                        self.device_ui.chemistry,
                        self.action_tx.clone(),
                    );
                } else {
                    self.ui.toast("Non connecté", ToastLevel::Warn);
                }
            }
            Action::DeviceCycleChemistry => {
                self.device_ui.cycle_chemistry();
                if let Some(mv) = self.device_ui.battery_mv {
                    self.device_ui.battery_percent =
                        Some(self.device_ui.chemistry.percentage(mv));
                }
                self.ui.toast(
                    format!("Chimie : {}", self.device_ui.chemistry_label()),
                    ToastLevel::Info,
                );
            }
            Action::DeviceSyncTime => {
                if self.ui.connected {
                    device_actions::spawn_sync_time(
                        self.service.clone(),
                        self.action_tx.clone(),
                    );
                }
            }
            Action::DeviceRequestSetName => {
                self.device_ui.name_input = self
                    .device_ui
                    .info
                    .as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_default();
                self.ui.push_modal(ModalKind::DeviceSetName);
            }
            Action::DeviceSubmitName => {
                let name = self.device_ui.name_input.trim().to_string();
                if name.is_empty() {
                    self.ui.toast("Nom vide", ToastLevel::Warn);
                } else {
                    self.ui.pop_modal();
                    device_actions::spawn_set_name(
                        self.service.clone(),
                        name,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::DeviceNameInputChar(c) => {
                self.device_ui.name_input.push(c);
            }
            Action::DeviceNameInputBackspace => {
                self.device_ui.name_input.pop();
            }
            Action::DeviceRequestSetTxPower => {
                self.device_ui.tx_power_draft = self
                    .device_ui
                    .info
                    .as_ref()
                    .map(|i| i.tx_power)
                    .unwrap_or(20);
                self.ui.push_modal(ModalKind::DeviceSetTxPower);
            }
            Action::DeviceSubmitTxPower => {
                let p = self.device_ui.tx_power_draft;
                self.ui.pop_modal();
                device_actions::spawn_set_tx_power(
                    self.service.clone(),
                    p,
                    self.action_tx.clone(),
                );
            }
            Action::DeviceTxPowerInc => {
                let max = self
                    .device_ui
                    .info
                    .as_ref()
                    .map(|i| i.max_tx_power)
                    .unwrap_or(30);
                self.device_ui.tx_power_draft =
                    self.device_ui.tx_power_draft.saturating_add(1).min(max);
            }
            Action::DeviceTxPowerDec => {
                self.device_ui.tx_power_draft =
                    self.device_ui.tx_power_draft.saturating_sub(1);
            }
            Action::DeviceRequestReboot => {
                self.ui.push_modal(ModalKind::ConfirmReboot);
            }
            Action::DeviceConfirmReboot => {
                self.ui.pop_modal();
                device_actions::spawn_reboot(
                    self.service.clone(),
                    self.action_tx.clone(),
                );
            }
            Action::DeviceSendAdvert { flood } => {
                if self.ui.connected {
                    device_actions::spawn_send_advert(
                        self.service.clone(),
                        flood,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::DeviceRefreshBattery => {
                if self.ui.connected {
                    device_actions::spawn_battery(
                        self.service.clone(),
                        self.device_ui.chemistry,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::ContactsOpenRepeater => {
                if let Some(c) = self.selected_contact().cloned() {
                    if c.node_type != 2 {
                        self.ui.toast(
                            "Ce contact n'est pas un repeater",
                            ToastLevel::Warn,
                        );
                    } else if !self.ui.connected {
                        self.ui.toast("Non connecté", ToastLevel::Warn);
                    } else {
                        self.repeater_ui.reset();
                        self.ui.push_modal(ModalKind::RepeaterAdmin {
                            pubkey: c.public_key,
                            name: c.name,
                        });
                    }
                }
            }

            // --- Repeater ---
            Action::RepeaterOpen { pubkey, name } => {
                self.repeater_ui.reset();
                self.ui.push_modal(ModalKind::RepeaterAdmin {
                    pubkey,
                    name,
                });
            }
            Action::RepeaterClose => {
                self.repeater_ui.reset();
                self.ui.pop_modal();
            }
            Action::RepeaterNextPane => {
                self.repeater_ui.pane = self.repeater_ui.pane.next();
            }
            Action::RepeaterPasswordChar(c) => {
                use tui_input::InputRequest;
                self.repeater_ui
                    .password_input
                    .handle(InputRequest::InsertChar(c));
            }
            Action::RepeaterPasswordBackspace => {
                use tui_input::InputRequest;
                self.repeater_ui
                    .password_input
                    .handle(InputRequest::DeletePrevChar);
            }
            Action::RepeaterPasswordSubmit => {
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    let password = self.repeater_ui.password_input.value().to_string();
                    self.repeater_ui.login_message =
                        Some("Authentification en cours…".to_string());
                    self.repeater_ui.loading = true;
                    repeater_actions::spawn_login(
                        self.service.clone(),
                        pubkey,
                        password,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::RepeaterLogout => {
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    repeater_actions::spawn_logout(
                        self.service.clone(),
                        pubkey,
                        self.action_tx.clone(),
                    );
                    self.repeater_ui.logged_in = false;
                    self.repeater_ui.password_input.reset();
                    self.repeater_ui.login_message = None;
                }
            }
            Action::RepeaterRefreshStatus => {
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    self.repeater_ui.loading = true;
                    repeater_actions::spawn_status(
                        self.service.clone(),
                        pubkey,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::RepeaterRefreshNeighbours => {
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    self.repeater_ui.loading = true;
                    repeater_actions::spawn_neighbours(
                        self.service.clone(),
                        pubkey,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::RepeaterRefreshAcl => {
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    self.repeater_ui.loading = true;
                    repeater_actions::spawn_acl(
                        self.service.clone(),
                        pubkey,
                        self.action_tx.clone(),
                    );
                }
            }
            Action::RepeaterFallbackToCli => {
                // Bascule sur le CLI + pré-remplit une commande utile selon le panneau échoué
                // (commandes réellement supportées par le firmware MeshCore)
                let cmd = match self.repeater_ui.pane {
                    crate::state::repeater::RepeaterPane::Status => "ver",
                    crate::state::repeater::RepeaterPane::Neighbours => "neighbors",
                    crate::state::repeater::RepeaterPane::Acl => "ver", // pas de cmd ACL texte
                    crate::state::repeater::RepeaterPane::Cli => return,
                };
                self.repeater_ui.pane = crate::state::repeater::RepeaterPane::Cli;
                self.repeater_ui.cli_input =
                    tui_input::Input::default().with_value(cmd.to_string());
                self.ui.toast(
                    format!("Fallback CLI : commande « {} » prête, Entrée pour envoyer", cmd),
                    ToastLevel::Info,
                );
            }
            Action::RepeaterCliChar(c) => {
                use tui_input::InputRequest;
                self.repeater_ui
                    .cli_input
                    .handle(InputRequest::InsertChar(c));
            }
            Action::RepeaterCliBackspace => {
                use tui_input::InputRequest;
                self.repeater_ui
                    .cli_input
                    .handle(InputRequest::DeletePrevChar);
            }
            Action::RepeaterCliSubmit => {
                let cmd = self.repeater_ui.cli_input.value().trim().to_string();
                if cmd.is_empty() {
                    return;
                }
                self.repeater_ui
                    .cli_output
                    .push(format!("> {}", cmd));
                self.repeater_ui.cli_input.reset();

                // Commandes locales (help / ?) — pas envoyées au repeater
                let lower = cmd.to_lowercase();
                if lower == "help" || lower == "?" || lower == "/help" {
                    for line in cli_help_lines() {
                        self.repeater_ui.cli_output.push(line.to_string());
                    }
                    self.repeater_ui.cli_output.push(String::new());
                    while self.repeater_ui.cli_output.len() > 500 {
                        self.repeater_ui.cli_output.remove(0);
                    }
                    return;
                }
                if lower == "clear" || lower == "cls" {
                    self.repeater_ui.cli_output.clear();
                    return;
                }

                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    self.repeater_ui.loading = true;
                    repeater_actions::spawn_cli(
                        self.service.clone(),
                        pubkey,
                        cmd,
                        self.action_tx.clone(),
                    );
                }
            }

            Action::ContactsCycleSort => {
                self.ui.contacts_sort = self.ui.contacts_sort.next();
                contact_actions::sort(&mut self.contacts, self.ui.contacts_sort);
                self.contacts_list_state.select(Some(0));
                self.rebuild_contact_rows();
                self.ui.toast(
                    format!("Tri : {}", self.ui.contacts_sort.label()),
                    ToastLevel::Info,
                );
            }

            // --- Navigation listes ---
            Action::Up => self.move_list(-1),
            Action::Down => self.move_list(1),
            Action::PageUp => self.move_list(-10),
            Action::PageDown => self.move_list(10),
            Action::Home => self.move_list_to(0),
            Action::End => self.move_list_to(i32::MAX),
            Action::Enter => self.handle_enter(),
            Action::Escape => self.ui.pop_modal(),
            Action::Char(_) | Action::Backspace => {}

            // --- Connexion ---
            Action::ConnSelectSubPane(pane) => {
                self.set_sub_pane(pane);
            }
            Action::ConnPrevSubPane => {
                let next = cycle_sub_pane(&self.connection_ui.sub_pane, false);
                self.set_sub_pane(next);
            }
            Action::ConnNextSubPane => {
                let next = cycle_sub_pane(&self.connection_ui.sub_pane, true);
                self.set_sub_pane(next);
            }
            Action::ConnScanCurrent => {
                self.scan_current_pane();
            }
            Action::ConnBleScan => {
                self.start_ble_scan();
            }
            Action::ConnSerialScan => {
                self.connection_ui.serial_scanning = true;
                conn_actions::spawn_serial_scan(self.action_tx.clone());
            }
            Action::ConnConnectSelected => self.connect_selected(),
            Action::ConnConnect(target) => {
                conn_actions::spawn_connect(
                    self.service.clone(),
                    target,
                    self.action_tx.clone(),
                );
            }
            Action::ConnDisconnectPrimary => {
                conn_actions::spawn_disconnect_primary(
                    self.service.clone(),
                    self.action_tx.clone(),
                );
            }
            Action::ConnDisconnectById(id) => {
                conn_actions::spawn_disconnect_by_id(
                    self.service.clone(),
                    id,
                    self.action_tx.clone(),
                );
            }
            Action::ConnSetPrimary(id) => {
                conn_actions::spawn_set_primary(
                    self.service.clone(),
                    id,
                    self.action_tx.clone(),
                );
            }
            Action::ConnRefreshList => {
                match self.connection_ui.sub_pane {
                    ConnectionSubPane::SerialList => {
                        conn_actions::spawn_serial_scan(self.action_tx.clone());
                    }
                    _ => {
                        conn_actions::refresh_list(
                            self.service.clone(),
                            self.action_tx.clone(),
                        );
                    }
                }
            }
            Action::ConnReconnectLast => {
                self.spawn_auto_reconnect();
            }
            Action::ConnTcpInputChar(c) => self.connection_ui.tcp_input.push(c),
            Action::ConnTcpInputBackspace => {
                self.connection_ui.tcp_input.pop();
            }
            Action::ConnTcpSubmit => {
                if let Some(target) = conn_actions::parse_tcp(&self.connection_ui.tcp_input)
                {
                    self.ui.pop_modal();
                    conn_actions::spawn_connect(
                        self.service.clone(),
                        target,
                        self.action_tx.clone(),
                    );
                } else {
                    self.ui
                        .toast("Adresse TCP invalide", ToastLevel::Error);
                }
            }

            // --- Backend events ---
            Action::Backend(event) => self.handle_backend(event),

            // --- Résultats async ---
            Action::Async(result) => self.handle_async(result),
        }
    }

    /// Appelé à chaque tick (150 ms). Termine l'état « réception en cours » si aucun message
    /// n'est arrivé depuis 5 secondes, et émet un toast récapitulatif + reload final.
    fn tick_receiving(&mut self) {
        if !self.ui.receiving_messages {
            return;
        }
        const IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
        let Some(last) = self.ui.last_message_received_at else {
            return;
        };
        if last.elapsed() < IDLE_TIMEOUT {
            return;
        }
        let count = self.ui.messages_received_count;
        let duration = self
            .ui
            .receiving_messages_since
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        self.ui.receiving_messages = false;
        self.ui.receiving_messages_since = None;
        if count > 0 {
            self.ui.toast(
                format!("{} message(s) récupéré(s) en {}s", count, duration),
                ToastLevel::Success,
            );
            // Reload final de la conversation active pour voir tous les messages
            if let Some(id) = self.chat_ui.active.clone() {
                messaging_actions::load_messages(
                    self.service.clone(),
                    id,
                    0,
                    false,
                    self.action_tx.clone(),
                );
            }
            // Recharger aussi les canaux pour mettre à jour les unread_count
            channel_actions::reload(self.service.clone(), self.action_tx.clone());
            // Et la liste des DMs : de nouveaux pubkeys peuvent être apparus
            messaging_actions::reload_dm_pubkeys(
                self.service.clone(),
                self.action_tx.clone(),
            );
        }
    }

    fn start_contacts_sync(&mut self) {
        self.ui.contacts_syncing = true;
        self.ui.contacts_sync_started_at = Some(std::time::Instant::now());
        contact_actions::spawn_sync(self.service.clone(), self.action_tx.clone());
    }

    // === Chat helpers ===

    /// Construit les résumés de conversations affichés dans la tab Chat.
    ///
    /// Contient : tous les canaux + tous les contacts favoris + tous les contacts
    /// ayant des messages en DB (via `self.dm_pubkeys` rempli par `reload_dm_pubkeys`).
    /// Les DMs sans contact correspondant (pubkey jamais sync) sont affichés avec un
    /// fallback sur le prefix de la pubkey.
    pub fn chat_conversation_summaries(&self) -> Vec<ConversationSummary> {
        use std::collections::HashSet;

        let mut out: Vec<ConversationSummary> = Vec::new();
        let mut dm_added: HashSet<String> = HashSet::new();

        // Canaux (toujours visibles)
        for ch in &self.channels {
            let id = ConversationId::Channel(ch.idx);
            let (last, last_ts) = self
                .chat_ui
                .messages
                .get(&id)
                .and_then(|msgs| msgs.last())
                .map(|m| (Some(m.text.clone()), Some(m.timestamp.clone())))
                .unwrap_or((None, None));
            out.push(ConversationSummary {
                id,
                display_name: format!("#{} {}", ch.idx, ch.name),
                last_message: last,
                last_timestamp: last_ts,
                unread: ch.unread_count,
            });
        }

        // DMs : toutes les pubkeys ayant des messages en DB (dm_pubkeys), avec nom
        // issu de self.contacts si trouvé, sinon fallback short pubkey
        for pk in &self.dm_pubkeys {
            let id = ConversationId::Dm(pk.clone());
            let display_name = self
                .contacts
                .iter()
                .find(|c| c.public_key == *pk || pk.starts_with(&c.public_key))
                .map(|c| c.name.clone())
                .unwrap_or_else(|| crate::util::format::short_pubkey(pk));
            let (last, last_ts) = self
                .chat_ui
                .messages
                .get(&id)
                .and_then(|msgs| msgs.last())
                .map(|m| (Some(m.text.clone()), Some(m.timestamp.clone())))
                .unwrap_or((None, None));
            let unread = self.chat_ui.unread.get(&id).copied().unwrap_or(0);
            out.push(ConversationSummary {
                id,
                display_name,
                last_message: last,
                last_timestamp: last_ts,
                unread,
            });
            dm_added.insert(pk.clone());
        }

        // Ajouter aussi les contacts favoris qui n'ont pas encore de messages
        // (donc pas dans dm_pubkeys) — utile pour initier une conversation
        for c in &self.contacts {
            if !c.is_favorite {
                continue;
            }
            if dm_added.contains(&c.public_key) {
                continue;
            }
            let id = ConversationId::Dm(c.public_key.clone());
            out.push(ConversationSummary {
                id,
                display_name: c.name.clone(),
                last_message: None,
                last_timestamp: None,
                unread: 0,
            });
        }

        // Tri : non-lus d'abord, puis par timestamp descendant
        out.sort_by(|a, b| {
            b.unread
                .cmp(&a.unread)
                .then_with(|| b.last_timestamp.cmp(&a.last_timestamp))
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
        out
    }

    fn chat_move_selection(&mut self, delta: i32) {
        let len = self.chat_conversation_summaries().len();
        if len == 0 {
            self.chat_ui.conversations_list_state.select(None);
            return;
        }
        let current = self
            .chat_ui
            .conversations_list_state
            .selected()
            .unwrap_or(0) as i32;
        let new = (current + delta).clamp(0, len as i32 - 1) as usize;
        self.chat_ui.conversations_list_state.select(Some(new));
    }

    fn chat_open_selected(&mut self) {
        let summaries = self.chat_conversation_summaries();
        let sel = self.chat_ui.conversations_list_state.selected().unwrap_or(0);
        if let Some(s) = summaries.get(sel) {
            self.chat_activate(s.id.clone());
        }
    }

    fn chat_activate(&mut self, id: ConversationId) {
        self.chat_ui.active = Some(id.clone());
        self.chat_ui.scroll_offset = 0;
        self.chat_ui.focus = ChatFocus::Input;
        self.ui.focus = FocusTarget::ChatInput;
        // Marquer lu
        self.chat_ui.unread.insert(id.clone(), 0);
        if let ConversationId::Channel(idx) = id.clone() {
            channel_actions::spawn_mark_read(
                self.service.clone(),
                idx,
                self.action_tx.clone(),
            );
        }
        // Charger les messages (ou rafraîchir)
        messaging_actions::load_messages(
            self.service.clone(),
            id,
            0,
            false,
            self.action_tx.clone(),
        );
    }

    fn chat_cycle_focus(&mut self) {
        let next = match self.chat_ui.focus {
            ChatFocus::List => ChatFocus::Input,
            ChatFocus::Input => ChatFocus::History,
            ChatFocus::History => ChatFocus::List,
        };
        self.chat_ui.focus = next;
        self.ui.focus = match next {
            ChatFocus::List => FocusTarget::ChatList,
            ChatFocus::History => FocusTarget::ChatHistory,
            ChatFocus::Input => FocusTarget::ChatInput,
        };
    }

    fn chat_send_current(&mut self) {
        let text = self.chat_ui.input.value().trim().to_string();
        if text.is_empty() {
            return;
        }
        let Some(id) = self.chat_ui.active.clone() else {
            self.ui
                .toast("Aucune conversation active", ToastLevel::Warn);
            return;
        };
        if !self.ui.connected {
            self.ui
                .toast("Non connecté — impossible d'envoyer", ToastLevel::Warn);
            return;
        }
        match id {
            ConversationId::Dm(pk) => {
                messaging_actions::spawn_send_dm(
                    self.service.clone(),
                    pk,
                    text,
                    self.action_tx.clone(),
                );
            }
            ConversationId::Channel(idx) => {
                messaging_actions::spawn_send_channel(
                    self.service.clone(),
                    idx,
                    text,
                    self.action_tx.clone(),
                );
            }
        }
        self.chat_ui.input.reset();
        self.chat_ui.scroll_offset = 0;
    }

    fn chat_load_older(&mut self) {
        let Some(id) = self.chat_ui.active.clone() else {
            return;
        };
        if self.chat_ui.fully_loaded.get(&id).copied().unwrap_or(false) {
            return;
        }
        let offset = self
            .chat_ui
            .messages
            .get(&id)
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        messaging_actions::load_messages(
            self.service.clone(),
            id,
            offset,
            true,
            self.action_tx.clone(),
        );
    }

    fn selected_channel(&self) -> Option<&meshcore_storage::channels::StoredChannel> {
        self.channels
            .get(self.channels_list_state.selected().unwrap_or(0))
    }

    /// Premier index 0..=7 non déjà utilisé dans self.channels
    fn find_free_channel_idx(&self) -> Option<u8> {
        (0u8..=7).find(|idx| !self.channels.iter().any(|c| c.idx == *idx))
    }

    fn submit_channel_new(&mut self) {
        let name = self.ui.channel_new_name.trim().to_string();
        if name.is_empty() {
            self.ui.toast("Le nom ne peut pas être vide", ToastLevel::Warn);
            return;
        }

        // Hashtag room : si le nom commence par # et le PSK est vide,
        // on dérive automatiquement le PSK via SHA256(name)[:16] (convention MeshCore)
        let is_hashtag = name.starts_with('#');
        let psk_hex = self.ui.channel_new_psk_hex.clone();
        let psk: [u8; 16] = if is_hashtag && psk_hex.is_empty() {
            self.ui.toast(
                format!("Hashtag room « {} » — PSK dérivé automatiquement", name),
                ToastLevel::Info,
            );
            crate::util::format::derive_hashtag_psk(&name)
        } else {
            if psk_hex.len() != 32 {
                self.ui.toast(
                    "PSK doit faire 32 caractères hex (16 octets). [F3] aléatoire, [F4] hashtag.",
                    ToastLevel::Warn,
                );
                return;
            }
            let mut psk = [0u8; 16];
            for i in 0..16 {
                let byte_str = &psk_hex[i * 2..i * 2 + 2];
                match u8::from_str_radix(byte_str, 16) {
                    Ok(b) => psk[i] = b,
                    Err(_) => {
                        self.ui.toast(
                            "PSK contient des caractères non-hex",
                            ToastLevel::Error,
                        );
                        return;
                    }
                }
            }
            psk
        };
        let Some(idx) = self.find_free_channel_idx() else {
            self.ui.toast("Aucun slot libre", ToastLevel::Error);
            return;
        };

        let new_channel = meshcore_storage::channels::StoredChannel {
            idx,
            name: name.clone(),
            channel_type: "public".to_string(),
            psk: psk.to_vec(),
            notifications_enabled: true,
            unread_count: 0,
        };

        // 1. Insère en DB locale
        if let Err(e) =
            meshcore_service::channels::upsert_channel(&self.service, &new_channel)
        {
            self.ui
                .toast(format!("Enregistrement local : {}", e), ToastLevel::Error);
            return;
        }
        self.ui.pop_modal();

        // 2. Sync sur le device (si connecté)
        if self.ui.connected {
            channel_actions::spawn_sync_to_device(
                self.service.clone(),
                new_channel,
                self.action_tx.clone(),
            );
        } else {
            self.ui.toast(
                format!("Canal « {} » créé (non connecté, pas de sync device)", name),
                ToastLevel::Warn,
            );
        }
        channel_actions::reload(self.service.clone(), self.action_tx.clone());
    }

    fn submit_channel_edit(&mut self, sync_to_device: bool) {
        let Some(ModalKind::ChannelEdit { idx }) = self.ui.top_modal().cloned() else {
            return;
        };
        let Some(existing) = self.channels.iter().find(|c| c.idx == idx).cloned() else {
            self.ui.pop_modal();
            return;
        };
        let name = self.ui.channel_edit_name.trim().to_string();
        if name.is_empty() {
            self.ui.toast("Le nom du canal ne peut pas être vide", ToastLevel::Warn);
            return;
        }

        // Mise à jour DB locale (upsert : préserve PSK et unread_count)
        let updated = meshcore_storage::channels::StoredChannel {
            idx,
            name: name.clone(),
            channel_type: existing.channel_type.clone(),
            psk: existing.psk.clone(),
            notifications_enabled: self.ui.channel_edit_notifications,
            unread_count: existing.unread_count,
        };
        if let Err(e) = meshcore_service::channels::upsert_channel(&self.service, &updated) {
            self.ui
                .toast(format!("Enregistrement local : {}", e), ToastLevel::Error);
            return;
        }

        // Persister le scope dans settings (ou le supprimer s'il est vide)
        let scope = self.ui.channel_edit_scope.trim().to_string();
        let scope_key = format!("channel.scope.{}", idx);
        if scope.is_empty() {
            let _ = self
                .service
                .db
                .with_conn(|c| meshcore_storage::settings::delete(c, &scope_key));
            self.channel_scopes.remove(&idx);
        } else {
            let scope_clone = scope.clone();
            let _ = self
                .service
                .db
                .with_conn(|c| meshcore_storage::settings::set(c, &scope_key, &scope_clone));
            self.channel_scopes.insert(idx, scope);
        }

        self.ui.pop_modal();

        if sync_to_device {
            channel_actions::spawn_sync_to_device(
                self.service.clone(),
                updated.clone(),
                self.action_tx.clone(),
            );
        } else {
            self.ui
                .toast("Canal enregistré localement", ToastLevel::Success);
        }
        channel_actions::reload(self.service.clone(), self.action_tx.clone());
    }

    fn set_sub_pane(&mut self, pane: ConnectionSubPane) {
        self.connection_ui.sub_pane = pane.clone();
        self.ui.focus = match pane {
            ConnectionSubPane::TcpInput => FocusTarget::ConnTcpInput,
            _ => FocusTarget::ConnList,
        };
    }

    fn start_ble_scan(&mut self) {
        if self.connection_ui.ble_scanning {
            return;
        }
        self.connection_ui.ble_scanning = true;
        self.ui
            .toast("Scan BLE en cours (5s)…", ToastLevel::Info);
        conn_actions::spawn_ble_scan(self.action_tx.clone());
    }

    fn scan_current_pane(&mut self) {
        match self.connection_ui.sub_pane {
            ConnectionSubPane::BleScan => self.start_ble_scan(),
            ConnectionSubPane::SerialList => {
                self.connection_ui.serial_scanning = true;
                conn_actions::spawn_serial_scan(self.action_tx.clone());
            }
            ConnectionSubPane::TcpInput => {
                self.ui.focus = FocusTarget::ConnTcpInput;
            }
            ConnectionSubPane::Active => {
                conn_actions::refresh_list(self.service.clone(), self.action_tx.clone());
            }
        }
    }

    fn goto_tab(&mut self, tab: Tab) {
        self.ui.current_tab = tab.clone();
        self.ui.focus = match tab {
            Tab::Connection => FocusTarget::ConnSubPane,
            Tab::Contacts => FocusTarget::ContactsList,
            Tab::Chat => match self.chat_ui.focus {
                ChatFocus::List => FocusTarget::ChatList,
                ChatFocus::History => FocusTarget::ChatHistory,
                ChatFocus::Input => FocusTarget::ChatInput,
            },
            Tab::Channels => FocusTarget::ChannelsList,
            Tab::Device => FocusTarget::DeviceBody,
        };
        // Auto-refresh infos device quand on arrive sur la tab 5
        if matches!(self.ui.current_tab, Tab::Device) && self.ui.connected {
            device_actions::spawn_refresh_info(
                self.service.clone(),
                self.action_tx.clone(),
            );
        }
    }

    fn cycle_focus(&mut self, forward: bool) {
        if matches!(self.ui.current_tab, Tab::Chat) {
            self.chat_cycle_focus();
            return;
        }
        self.ui.focus = match (&self.ui.current_tab, self.ui.focus, forward) {
            (Tab::Connection, FocusTarget::ConnSubPane, true) => {
                if matches!(self.connection_ui.sub_pane, ConnectionSubPane::TcpInput) {
                    FocusTarget::ConnTcpInput
                } else {
                    FocusTarget::ConnList
                }
            }
            (Tab::Connection, FocusTarget::ConnList, true)
            | (Tab::Connection, FocusTarget::ConnTcpInput, true) => FocusTarget::ConnSubPane,
            (Tab::Connection, _, false) => FocusTarget::ConnSubPane,
            (Tab::Contacts, _, _) => FocusTarget::ContactsList,
            (Tab::Channels, _, _) => FocusTarget::ChannelsList,
            _ => FocusTarget::Body,
        };
    }

    fn handle_enter(&mut self) {
        match self.ui.current_tab {
            Tab::Connection => self.connect_selected(),
            Tab::Contacts => self.toggle_contact_group_if_header(),
            Tab::Chat => match self.ui.focus {
                FocusTarget::ChatList => self.chat_open_selected(),
                FocusTarget::ChatInput => self.chat_send_current(),
                _ => {}
            },
            Tab::Channels => {
                if let Some(c) = self.selected_channel().cloned() {
                    self.chat_activate(ConversationId::Channel(c.idx));
                    self.goto_tab(Tab::Chat);
                }
            }
            _ => {}
        }
    }

    fn toggle_contact_group_if_header(&mut self) {
        let Some(i) = self.contacts_list_state.selected() else {
            return;
        };
        let Some(row) = self.contact_rows.get(i) else {
            return;
        };
        if let ContactRow::Header { node_type, .. } = row {
            let nt = *node_type;
            if self.ui.contacts_collapsed_groups.contains(&nt) {
                self.ui.contacts_collapsed_groups.remove(&nt);
            } else {
                self.ui.contacts_collapsed_groups.insert(nt);
            }
            // Garder la sélection sur ce header après rebuild
            let header_index_after =
                find_header_index(&self.contact_rows, nt).or(Some(i));
            self.rebuild_contact_rows();
            // Si le header est toujours présent, on replace le curseur dessus
            let final_idx = self
                .contact_rows
                .iter()
                .position(|r| {
                    matches!(r, ContactRow::Header { node_type: n, .. } if *n == nt)
                })
                .or(header_index_after);
            self.contacts_list_state.select(final_idx);
        }
    }

    fn connect_selected(&mut self) {
        match self.connection_ui.sub_pane {
            ConnectionSubPane::BleScan => {
                let sel = self.connection_ui.ble_list_state.selected().unwrap_or(0);
                if let Some(dev) = self.connection_ui.ble_devices.get(sel) {
                    let target = ConnectionTarget::Ble {
                        name_or_addr: dev.name.clone(),
                    };
                    self.ui
                        .toast(format!("Connexion à {}…", dev.name), ToastLevel::Info);
                    conn_actions::spawn_connect(
                        self.service.clone(),
                        target,
                        self.action_tx.clone(),
                    );
                }
            }
            ConnectionSubPane::SerialList => {
                let sel = self.connection_ui.serial_list_state.selected().unwrap_or(0);
                if let Some(port) = self.connection_ui.serial_ports.get(sel) {
                    let target = ConnectionTarget::Serial {
                        port: port.clone(),
                        baud_rate: 115_200,
                    };
                    self.ui
                        .toast(format!("Connexion à {}…", port), ToastLevel::Info);
                    conn_actions::spawn_connect(
                        self.service.clone(),
                        target,
                        self.action_tx.clone(),
                    );
                }
            }
            ConnectionSubPane::TcpInput => {
                if let Some(target) = conn_actions::parse_tcp(&self.connection_ui.tcp_input)
                {
                    conn_actions::spawn_connect(
                        self.service.clone(),
                        target,
                        self.action_tx.clone(),
                    );
                } else {
                    self.ui
                        .toast("Adresse TCP invalide", ToastLevel::Error);
                }
            }
            ConnectionSubPane::Active => {
                let sel = self.connection_ui.active_list_state.selected().unwrap_or(0);
                if let Some(c) = self.connection_ui.active_connections.get(sel).cloned() {
                    conn_actions::spawn_set_primary(
                        self.service.clone(),
                        c.id,
                        self.action_tx.clone(),
                    );
                }
            }
        }
    }

    fn move_list(&mut self, delta: i32) {
        match self.ui.current_tab {
            Tab::Connection => self.connection_ui.move_selection(delta),
            Tab::Contacts => {
                let len = self.contact_rows.len();
                if len == 0 {
                    self.contacts_list_state.select(None);
                    return;
                }
                let current = self.contacts_list_state.selected().unwrap_or(0) as i32;
                let new = (current + delta).clamp(0, len as i32 - 1) as usize;
                self.contacts_list_state.select(Some(new));
            }
            Tab::Chat => match self.ui.focus {
                FocusTarget::ChatList => self.chat_move_selection(delta),
                FocusTarget::ChatHistory => {
                    if delta < 0 {
                        self.chat_ui.scroll_offset =
                            self.chat_ui.scroll_offset.saturating_add((-delta) as u16);
                    } else {
                        self.chat_ui.scroll_offset =
                            self.chat_ui.scroll_offset.saturating_sub(delta as u16);
                    }
                }
                _ => {}
            },
            Tab::Channels => {
                let len = self.channels.len();
                if len == 0 {
                    self.channels_list_state.select(None);
                    return;
                }
                let current = self.channels_list_state.selected().unwrap_or(0) as i32;
                let new = (current + delta).clamp(0, len as i32 - 1) as usize;
                self.channels_list_state.select(Some(new));
            }
            _ => {}
        }
    }

    fn move_list_to(&mut self, pos: i32) {
        let delta = match pos {
            0 => i32::MIN / 2,
            _ => i32::MAX / 2,
        };
        self.move_list(delta);
    }

    fn selected_contact(&self) -> Option<&StoredContact> {
        let i = self.contacts_list_state.selected()?;
        match self.contact_rows.get(i)? {
            ContactRow::Contact(idx) => self.contacts.get(*idx),
            ContactRow::Header { .. } => None,
        }
    }

    fn handle_backend(&mut self, event: AppEvent) {
        match event {
            AppEvent::Connected { device_name } => {
                self.ui.connected = true;
                self.ui.device_name = Some(device_name.clone());
                self.ui
                    .toast(format!("Connecté à {}", device_name), ToastLevel::Success);
                conn_actions::refresh_list(self.service.clone(), self.action_tx.clone());
                // Recharger les canaux : le service les a rafraîchis depuis le device
                // pendant le handshake (get_channel 0..8) → l'UI doit les relire en DB
                channel_actions::reload(self.service.clone(), self.action_tx.clone());
                // Recharger la liste des DMs existants pour que les conversations
                // apparaissent dans la tab Chat sans clic préalable
                messaging_actions::reload_dm_pubkeys(
                    self.service.clone(),
                    self.action_tx.clone(),
                );
                // Activer l'indicateur de réception — le device va vider sa file d'attente
                // de messages via mc.start_auto_message_fetching() (potentiellement plusieurs
                // minutes en BLE multi-hop)
                self.ui.receiving_messages = true;
                self.ui.receiving_messages_since = Some(std::time::Instant::now());
                self.ui.last_message_received_at = Some(std::time::Instant::now());
                self.ui.messages_received_count = 0;
                // Auto-sync des contacts dès la connexion établie
                if !self.ui.contacts_syncing {
                    self.start_contacts_sync();
                }
                // Si l'utilisateur est sur la tab Connexion, basculer sur le sous-panneau
                // « Actives » pour qu'il voie la nouvelle connexion (sinon il reste sur BLE
                // qui affiche « Aucun périphérique trouvé » car aucun scan fait)
                if matches!(self.ui.current_tab, Tab::Connection) {
                    self.set_sub_pane(ConnectionSubPane::Active);
                }
                // Sauvegarde le companion pour la prochaine auto-reconnexion
                let svc = self.service.clone();
                let name = device_name;
                tokio::spawn(async move {
                    companion_actions::save_current_primary(svc, name).await;
                });
            }
            AppEvent::Disconnected => {
                self.ui.connected = false;
                self.ui.toast("Déconnexion", ToastLevel::Warn);
                conn_actions::refresh_list(self.service.clone(), self.action_tx.clone());
            }
            AppEvent::Reconnecting { attempt } => {
                self.ui
                    .toast(format!("Reconnexion (essai {})…", attempt), ToastLevel::Info);
            }
            AppEvent::ContactsSynced { count } => {
                tracing::debug!("Broadcast ContactsSynced({})", count);
            }
            AppEvent::DirectMessageReceived {
                sender_pubkey,
                sender_name,
                text,
                snr,
            } => {
                tracing::info!(
                    "tui: DirectMessageReceived from={} len={}",
                    sender_pubkey,
                    text.len()
                );
                let msg = meshcore_storage::models::StoredMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    direction: "incoming".to_string(),
                    sender_pubkey: Some(sender_pubkey.clone()),
                    sender_name: sender_name.clone(),
                    recipient_pubkey: None,
                    channel_idx: None,
                    text: text.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    status: "received".to_string(),
                    snr,
                    rssi: None,
                    path_len: None,
                    attempt: 0,
                    reply_to: None,
                    reaction: None,
                };
                match self.service.db.with_conn(|c| {
                    meshcore_storage::messages::insert_message(c, &msg)
                }) {
                    Ok(()) => tracing::info!("tui: DM inserted ok"),
                    Err(e) => tracing::error!("tui: DM INSERT FAILED : {}", e),
                }

                // Maj de l'indicateur de réception
                self.ui.last_message_received_at = Some(std::time::Instant::now());
                self.ui.messages_received_count = self.ui.messages_received_count.saturating_add(1);

                let id = ConversationId::Dm(sender_pubkey);
                if self.chat_ui.active.as_ref() != Some(&id) {
                    *self.chat_ui.unread.entry(id.clone()).or_insert(0) += 1;
                    // Toast uniquement si on n'est pas en plein rattrapage (sinon spam)
                    if !self.ui.receiving_messages && !matches!(self.ui.current_tab, Tab::Chat) {
                        self.ui.toast(
                            "Nouveau message — tab Chat",
                            ToastLevel::Info,
                        );
                    }
                }
                // Pendant le rattrapage, on évite de recharger à chaque message (gaspillage DB)
                // sauf si c'est la conversation active
                if self.chat_ui.active.as_ref() == Some(&id) || !self.ui.receiving_messages {
                    messaging_actions::load_messages(
                        self.service.clone(),
                        id,
                        0,
                        false,
                        self.action_tx.clone(),
                    );
                }
            }
            AppEvent::ChannelMessageReceived {
                channel_idx,
                sender_name,
                text,
            } => {
                tracing::info!(
                    "tui: ChannelMessageReceived channel={} sender='{}' text_len={} text='{}'",
                    channel_idx,
                    sender_name,
                    text.len(),
                    text
                );
                let msg = meshcore_storage::models::StoredMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    direction: "incoming".to_string(),
                    sender_pubkey: None,
                    sender_name: sender_name.clone(),
                    recipient_pubkey: None,
                    channel_idx: Some(channel_idx),
                    text: text.clone(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    status: "received".to_string(),
                    snr: None,
                    rssi: None,
                    path_len: None,
                    attempt: 0,
                    reply_to: None,
                    reaction: None,
                };
                let msg_id = msg.id.clone();
                match self.service.db.with_conn(|c| {
                    meshcore_storage::messages::insert_message(c, &msg)
                }) {
                    Ok(()) => tracing::info!(
                        "tui: channel msg inserted id={} channel={}",
                        msg_id,
                        channel_idx
                    ),
                    Err(e) => tracing::error!(
                        "tui: channel msg INSERT FAILED id={} channel={} : {}",
                        msg_id,
                        channel_idx,
                        e
                    ),
                }

                // Maj de l'indicateur de réception
                self.ui.last_message_received_at = Some(std::time::Instant::now());
                self.ui.messages_received_count = self.ui.messages_received_count.saturating_add(1);

                let id = ConversationId::Channel(channel_idx);
                if self.chat_ui.active.as_ref() != Some(&id) {
                    if let Some(ch) =
                        self.channels.iter_mut().find(|c| c.idx == channel_idx)
                    {
                        ch.unread_count += 1;
                    }
                    if !self.ui.receiving_messages && !matches!(self.ui.current_tab, Tab::Chat) {
                        self.ui.toast(
                            format!("Nouveau message sur canal #{}", channel_idx),
                            ToastLevel::Info,
                        );
                    }
                }
                if self.chat_ui.active.as_ref() == Some(&id) || !self.ui.receiving_messages {
                    messaging_actions::load_messages(
                        self.service.clone(),
                        id,
                        0,
                        false,
                        self.action_tx.clone(),
                    );
                }
            }
            AppEvent::MessageSent { .. } => {
                // Reload de la conversation active (statut passe pending→sent)
                if let Some(id) = self.chat_ui.active.clone() {
                    messaging_actions::load_messages(
                        self.service.clone(),
                        id,
                        0,
                        false,
                        self.action_tx.clone(),
                    );
                }
            }
            AppEvent::MessageDelivered { .. } => {
                if let Some(id) = self.chat_ui.active.clone() {
                    messaging_actions::load_messages(
                        self.service.clone(),
                        id,
                        0,
                        false,
                        self.action_tx.clone(),
                    );
                }
            }
            AppEvent::MessageFailed { reason, .. } => {
                self.ui
                    .toast(format!("Échec envoi : {}", reason), ToastLevel::Error);
                if let Some(id) = self.chat_ui.active.clone() {
                    messaging_actions::load_messages(
                        self.service.clone(),
                        id,
                        0,
                        false,
                        self.action_tx.clone(),
                    );
                }
            }
            AppEvent::ContactDiscovered { name, .. } => {
                self.ui
                    .toast(format!("Nouveau contact : {}", name), ToastLevel::Info);
                contact_actions::reload(self.service.clone(), self.action_tx.clone());
            }
            AppEvent::BatteryUpdate { percent, .. } => {
                self.ui.battery_percent = Some(percent);
            }
            AppEvent::StatsReceived { last_rssi, .. } => {
                self.ui.last_rssi = Some(last_rssi);
            }
            AppEvent::Error { message } => {
                self.ui.toast(message, ToastLevel::Error);
            }
            _ => {}
        }
    }

    fn handle_async(&mut self, result: AsyncResult) {
        match result {
            AsyncResult::BleScanDone(devices) => {
                self.connection_ui.ble_scanning = false;
                let count = devices.len();
                self.connection_ui.ble_devices = devices;
                if count > 0 {
                    self.connection_ui.ble_list_state.select(Some(0));
                } else {
                    self.connection_ui.ble_list_state.select(None);
                }
                self.ui
                    .toast(format!("{} device(s) BLE trouvé(s)", count), ToastLevel::Success);
            }
            AsyncResult::BleScanFailed(e) => {
                self.connection_ui.ble_scanning = false;
                self.ui.toast(format!("Scan BLE : {}", e), ToastLevel::Error);
            }
            AsyncResult::SerialScanDone(ports) => {
                self.connection_ui.serial_scanning = false;
                let count = ports.len();
                self.connection_ui.serial_ports = ports;
                if count > 0 {
                    self.connection_ui.serial_list_state.select(Some(0));
                } else {
                    self.connection_ui.serial_list_state.select(None);
                }
            }
            AsyncResult::SerialScanFailed(e) => {
                self.connection_ui.serial_scanning = false;
                self.ui
                    .toast(format!("Scan série : {}", e), ToastLevel::Error);
            }
            AsyncResult::ContactsReloaded(list) => {
                self.contacts = list;
                contact_actions::sort(&mut self.contacts, self.ui.contacts_sort);
                self.rebuild_contact_rows();
            }
            AsyncResult::DeviceInfoLoaded(Ok(info)) => {
                self.device_ui.info = Some(info);
            }
            AsyncResult::DeviceInfoLoaded(Err(e)) => {
                self.ui
                    .toast(format!("Info device : {}", e), ToastLevel::Error);
            }
            AsyncResult::BatteryLoaded(Ok((mv, pct))) => {
                self.device_ui.battery_mv = Some(mv);
                self.device_ui.battery_percent = Some(pct);
                self.ui.battery_percent = Some(pct);
            }
            AsyncResult::BatteryLoaded(Err(e)) => {
                self.ui
                    .toast(format!("Batterie : {}", e), ToastLevel::Error);
            }
            AsyncResult::RepeaterLoginResult(Ok(msg)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.logged_in = true;
                self.repeater_ui.login_message = Some(msg);
                self.repeater_ui.password_input.reset();
                // Pré-charger le status
                if let Some(ModalKind::RepeaterAdmin { pubkey, .. }) =
                    self.ui.top_modal().cloned()
                {
                    repeater_actions::spawn_status(
                        self.service.clone(),
                        pubkey,
                        self.action_tx.clone(),
                    );
                }
            }
            AsyncResult::RepeaterLoginResult(Err(e)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.login_message = Some(format!("Échec : {}", e));
            }
            AsyncResult::RepeaterStatusLoaded(Ok(status)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.status = Some(status);
                self.repeater_ui.status_error = None;
            }
            AsyncResult::RepeaterStatusLoaded(Err(e)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.status_error = Some(e);
            }
            AsyncResult::RepeaterNeighboursLoaded(Ok(list)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.neighbours = list;
                self.repeater_ui.neighbours_error = None;
            }
            AsyncResult::RepeaterNeighboursLoaded(Err(e)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.neighbours_error = Some(e);
            }
            AsyncResult::RepeaterAclLoaded(Ok(list)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.acl = list;
                self.repeater_ui.acl_error = None;
            }
            AsyncResult::RepeaterAclLoaded(Err(e)) => {
                self.repeater_ui.loading = false;
                self.repeater_ui.acl_error = Some(e);
            }
            AsyncResult::RepeaterCliResult { command, result } => {
                self.repeater_ui.loading = false;
                match result {
                    Ok(output) => {
                        for line in output.split('\n') {
                            self.repeater_ui.cli_output.push(line.to_string());
                        }
                        self.repeater_ui.cli_output.push(String::new());
                    }
                    Err(e) => {
                        self.repeater_ui
                            .cli_output
                            .push(format!("[erreur sur « {} »] {}", command, e));
                        self.repeater_ui.cli_output.push(String::new());
                    }
                }
                // Limite raisonnable d'historique
                while self.repeater_ui.cli_output.len() > 500 {
                    self.repeater_ui.cli_output.remove(0);
                }
            }
            AsyncResult::DmPubkeysLoaded(list) => {
                self.dm_pubkeys = list;
            }
            AsyncResult::ChannelsReloaded(list) => {
                self.channels = list;
                if self.channels.is_empty() {
                    self.channels_list_state.select(None);
                } else if self.channels_list_state.selected().is_none() {
                    self.channels_list_state.select(Some(0));
                }
                // Charger les scopes persistés (tolérant à l'échec)
                self.channel_scopes.clear();
                for ch in &self.channels {
                    let key = format!("channel.scope.{}", ch.idx);
                    if let Ok(Some(val)) = self
                        .service
                        .db
                        .with_conn(|c| meshcore_storage::settings::get(c, &key))
                        && !val.is_empty()
                    {
                        self.channel_scopes.insert(ch.idx, val);
                    }
                }
            }
            AsyncResult::MessagesLoaded {
                conversation,
                messages,
                prepend,
                fully_loaded,
            } => {
                let entry = self
                    .chat_ui
                    .messages
                    .entry(conversation.clone())
                    .or_default();
                if prepend {
                    // Pagination : on insère en tête, puis on clamp à MAX_IN_MEMORY
                    let mut new_list = messages;
                    new_list.append(entry);
                    // On coupe par le bas pour garder les plus récents
                    if new_list.len() > MAX_IN_MEMORY {
                        let drop_count = new_list.len() - MAX_IN_MEMORY;
                        new_list.drain(..drop_count);
                    }
                    *entry = new_list;
                } else {
                    *entry = messages;
                }
                self.chat_ui.fully_loaded.insert(conversation, fully_loaded);
            }
            AsyncResult::ContactsSyncDone(result) => {
                self.ui.contacts_syncing = false;
                let duration = self
                    .ui
                    .contacts_sync_started_at
                    .take()
                    .map(|t| t.elapsed().as_secs())
                    .unwrap_or(0);
                match result {
                    Ok(n) => self.ui.toast(
                        format!("{} contacts synchronisés en {}s", n, duration),
                        ToastLevel::Success,
                    ),
                    Err(e) => self
                        .ui
                        .toast(format!("Sync échouée : {}", e), ToastLevel::Error),
                }
            }
            AsyncResult::ConnectionsListed(list) => {
                self.connection_ui.active_connections = list;
                if self.connection_ui.active_connections.is_empty() {
                    self.connection_ui.active_list_state.select(None);
                } else if self.connection_ui.active_list_state.selected().is_none() {
                    self.connection_ui.active_list_state.select(Some(0));
                }
                // Maj état connecté via list non vide
                self.ui.connected = !self.connection_ui.active_connections.is_empty();
            }
            AsyncResult::Generic(Ok(msg)) => {
                self.ui.toast(msg, ToastLevel::Success);
            }
            AsyncResult::Generic(Err(e)) => {
                self.ui.toast(e, ToastLevel::Error);
            }
        }
    }
}
