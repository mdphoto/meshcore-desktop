use crate::action::{ModalKind, Tab, ToastLevel};
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactSortMode {
    FavoritesAlpha,
    ByType,
    Alpha,
}

impl ContactSortMode {
    pub fn next(self) -> Self {
        match self {
            Self::FavoritesAlpha => Self::ByType,
            Self::ByType => Self::Alpha,
            Self::Alpha => Self::FavoritesAlpha,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::FavoritesAlpha => "favoris",
            Self::ByType => "type",
            Self::Alpha => "nom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    // Connection tab
    ConnSubPane,
    ConnList,
    ConnTcpInput,
    // Contacts tab
    ContactsList,
    // Chat tab
    ChatList,
    ChatHistory,
    ChatInput,
    // Channels tab
    ChannelsList,
    // Device tab
    DeviceBody,
    // Default
    Body,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub level: ToastLevel,
    pub created_at: Instant,
}

pub struct AppUiState {
    pub current_tab: Tab,
    pub focus: FocusTarget,
    pub modal_stack: Vec<ModalKind>,
    pub toasts: VecDeque<Toast>,
    pub should_quit: bool,
    pub device_name: Option<String>,
    pub connected: bool,
    pub last_rssi: Option<i16>,
    pub battery_percent: Option<u8>,
    pub contacts_sort: ContactSortMode,
    pub contacts_collapsed_groups: HashSet<u8>,
    pub contacts_syncing: bool,
    pub contacts_sync_started_at: Option<Instant>,
    /// Réception de messages en cours (file d'attente du device se vide via auto-fetching).
    /// Set à `true` sur AppEvent::Connected, reset ~5s après le dernier message reçu.
    pub receiving_messages: bool,
    pub receiving_messages_since: Option<Instant>,
    pub last_message_received_at: Option<Instant>,
    pub messages_received_count: u32,
    // Édition de canal en cours
    pub channel_edit_name: String,
    pub channel_edit_notifications: bool,
    pub channel_edit_scope: String,
    /// Champ actif dans la modale d'édition (0=name, 1=notif, 2=scope)
    pub channel_edit_field: u8,
    /// PSK hex du canal en cours d'édition (read-only, affiché pour copie)
    pub channel_edit_psk_hex: String,
    /// True si le popup @mention est ouvert (miroir de `chat_ui.mention.is_some()`,
    /// dupliqué ici pour que events/input.rs intercepte Tab/Up/Down avant les bindings globaux)
    pub mention_open: bool,
    // Création d'un nouveau canal
    pub channel_new_name: String,
    /// PSK 32 caractères hex = 16 octets (affiché en hex pour lisibilité)
    pub channel_new_psk_hex: String,
    /// Champ actif dans la modale création (0=name, 1=psk)
    pub channel_new_field: u8,
}

impl Default for AppUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppUiState {
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Connection,
            focus: FocusTarget::ConnSubPane,
            modal_stack: Vec::new(),
            toasts: VecDeque::new(),
            should_quit: false,
            device_name: None,
            connected: false,
            last_rssi: None,
            battery_percent: None,
            contacts_sort: ContactSortMode::FavoritesAlpha,
            contacts_collapsed_groups: HashSet::new(),
            contacts_syncing: false,
            contacts_sync_started_at: None,
            receiving_messages: false,
            receiving_messages_since: None,
            last_message_received_at: None,
            messages_received_count: 0,
            channel_edit_name: String::new(),
            channel_edit_notifications: true,
            channel_edit_scope: String::new(),
            channel_edit_field: 0,
            channel_edit_psk_hex: String::new(),
            mention_open: false,
            channel_new_name: String::new(),
            channel_new_psk_hex: String::new(),
            channel_new_field: 0,
        }
    }

    pub fn toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        let msg = message.into();
        if self.toasts.back().is_some_and(|t| t.message == msg) {
            return;
        }
        self.toasts.push_back(Toast {
            message: msg,
            level,
            created_at: Instant::now(),
        });
        while self.toasts.len() > 5 {
            self.toasts.pop_front();
        }
    }

    pub fn prune_toasts(&mut self) {
        let now = Instant::now();
        while let Some(t) = self.toasts.front() {
            if now.duration_since(t.created_at).as_secs() >= 5 {
                self.toasts.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn is_modal_open(&self) -> bool {
        !self.modal_stack.is_empty()
    }

    pub fn top_modal(&self) -> Option<&ModalKind> {
        self.modal_stack.last()
    }

    pub fn push_modal(&mut self, kind: ModalKind) {
        self.modal_stack.push(kind);
    }

    pub fn pop_modal(&mut self) {
        self.modal_stack.pop();
    }
}
