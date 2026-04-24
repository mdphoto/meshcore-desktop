use crate::action::AsyncResult;
use meshcorex_service::repeater::{AclEntry, RepeaterNeighbour, RepeaterStatus};
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeaterPane {
    Status,
    Neighbours,
    Acl,
    Cli,
}

impl RepeaterPane {
    pub fn all() -> [RepeaterPane; 4] {
        [Self::Status, Self::Neighbours, Self::Acl, Self::Cli]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Neighbours => "Voisins",
            Self::Acl => "ACL",
            Self::Cli => "CLI",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Status => Self::Neighbours,
            Self::Neighbours => Self::Acl,
            Self::Acl => Self::Cli,
            Self::Cli => Self::Status,
        }
    }
}

pub struct RepeaterUiState {
    /// Sous-panneau actif
    pub pane: RepeaterPane,
    /// Login en cours / réussi (si Some, c'est un message d'état)
    pub login_message: Option<String>,
    pub logged_in: bool,
    /// Cached data
    pub status: Option<RepeaterStatus>,
    pub neighbours: Vec<RepeaterNeighbour>,
    pub acl: Vec<AclEntry>,
    /// CLI
    pub cli_input: Input,
    pub cli_output: Vec<String>,
    /// Modale password en cours
    pub password_input: Input,
    pub password_mode: bool,
    /// Loading en cours
    pub loading: bool,
    /// Dernière erreur par panneau (affichée en place des données quand non null)
    pub status_error: Option<String>,
    pub neighbours_error: Option<String>,
    pub acl_error: Option<String>,
}

impl RepeaterUiState {
    pub fn new() -> Self {
        Self {
            pane: RepeaterPane::Status,
            login_message: None,
            logged_in: false,
            status: None,
            neighbours: Vec::new(),
            acl: Vec::new(),
            cli_input: Input::default(),
            cli_output: Vec::new(),
            password_input: Input::default(),
            password_mode: true,
            loading: false,
            status_error: None,
            neighbours_error: None,
            acl_error: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn apply_async(&mut self, result: &AsyncResult) {
        // Les mises à jour asynchrones du repeater arrivent via AsyncResult::Generic ou les variants
        // dédiés. Pour l'instant on laisse l'appelant appliquer directement.
        let _ = result;
    }
}

impl Default for RepeaterUiState {
    fn default() -> Self {
        Self::new()
    }
}
