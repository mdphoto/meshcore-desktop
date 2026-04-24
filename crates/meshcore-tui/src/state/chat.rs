use meshcore_storage::models::StoredMessage;
use ratatui::widgets::ListState;
use std::collections::HashMap;
use tui_input::Input;

pub const PAGE_SIZE: u32 = 50;
pub const MAX_IN_MEMORY: usize = 500;

/// Identifiant logique d'une conversation (DM ou canal)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversationId {
    Dm(String),
    Channel(u8),
}

/// Un résumé de conversation pour la liste latérale
#[derive(Debug, Clone)]
pub struct ConversationSummary {
    pub id: ConversationId,
    pub display_name: String,
    pub last_message: Option<String>,
    pub last_timestamp: Option<String>,
    pub unread: u32,
}

pub struct ChatUiState {
    /// Conversation actuellement sélectionnée
    pub active: Option<ConversationId>,
    /// Index UI dans la liste des conversations
    pub conversations_list_state: ListState,
    /// Cache des messages par conversation (chronologique ascendant)
    pub messages: HashMap<ConversationId, Vec<StoredMessage>>,
    /// Flag « la conversation a été entièrement chargée » (plus de messages anciens)
    pub fully_loaded: HashMap<ConversationId, bool>,
    /// État de saisie
    pub input: Input,
    /// Position de scroll dans l'historique (0 = bas)
    pub scroll_offset: u16,
    /// Mode de focus interne (liste / historique / input)
    pub focus: ChatFocus,
    /// Messages non lus par conversation (calcul client-side pour DM, read de channel.unread_count sinon)
    pub unread: HashMap<ConversationId, u32>,
    /// État du popup d'autocomplétion `@mention` (None si fermé)
    pub mention: Option<MentionState>,
}

/// État du popup d'autocomplétion @mention. Ouvert quand l'utilisateur tape `@`
/// en début de mot, fermé sur Esc / Enter / Backspace au début.
pub struct MentionState {
    /// Index (en caractères) du `@` dans la valeur de l'input
    pub start_pos: usize,
    /// Filtre tapé après le `@` (en lowercase pour comparaison)
    pub query: String,
    /// Liste complète des candidats chargés depuis la DB
    pub candidates: Vec<String>,
    /// Index sélectionné dans la liste filtrée
    pub selected: usize,
}

impl MentionState {
    /// Retourne les candidats filtrés par la query actuelle
    pub fn filtered(&self) -> Vec<&String> {
        if self.query.is_empty() {
            return self.candidates.iter().collect();
        }
        let q = self.query.to_lowercase();
        self.candidates
            .iter()
            .filter(|name| name.to_lowercase().contains(&q))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatFocus {
    List,
    History,
    Input,
}

impl ChatUiState {
    pub fn new() -> Self {
        Self {
            active: None,
            conversations_list_state: ListState::default(),
            messages: HashMap::new(),
            fully_loaded: HashMap::new(),
            input: Input::default(),
            scroll_offset: 0,
            focus: ChatFocus::List,
            unread: HashMap::new(),
            mention: None,
        }
    }

    pub fn active_messages(&self) -> Option<&Vec<StoredMessage>> {
        self.messages.get(self.active.as_ref()?)
    }
}

impl Default for ChatUiState {
    fn default() -> Self {
        Self::new()
    }
}
