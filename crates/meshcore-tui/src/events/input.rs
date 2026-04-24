use crate::action::{Action, ModalKind, Tab};
use crate::state::{AppUiState, FocusTarget};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn map_event(event: Event, ui: &AppUiState) -> Action {
    let Event::Key(key) = event else {
        return Action::NoOp;
    };
    if key.kind != KeyEventKind::Press {
        return Action::NoOp;
    }

    if ui.is_modal_open() {
        return map_modal_key(key, ui);
    }

    map_tab_key(key, ui)
}

/// Variante globalement appelée quand une modale RepeaterAdmin est ouverte (besoin du state
/// repeater pour savoir si on est en mode password ou navigation).
pub fn map_event_with_repeater(
    event: Event,
    ui: &AppUiState,
    repeater: &crate::state::repeater::RepeaterUiState,
) -> Action {
    let Event::Key(key) = event else {
        return Action::NoOp;
    };
    if key.kind != KeyEventKind::Press {
        return Action::NoOp;
    }
    if let Some(ModalKind::RepeaterAdmin { .. }) = ui.top_modal() {
        return map_repeater_modal_key(key, repeater);
    }
    map_event(event, ui)
}

fn map_repeater_modal_key(
    key: KeyEvent,
    repeater: &crate::state::repeater::RepeaterUiState,
) -> Action {
    use crate::state::repeater::RepeaterPane;

    // Esc ferme toujours
    if matches!(key.code, KeyCode::Esc) {
        return Action::RepeaterClose;
    }

    if !repeater.logged_in {
        return match key.code {
            KeyCode::Enter => Action::RepeaterPasswordSubmit,
            KeyCode::Backspace => Action::RepeaterPasswordBackspace,
            KeyCode::Char(c) => Action::RepeaterPasswordChar(c),
            _ => Action::NoOp,
        };
    }

    // Mode logged in
    match repeater.pane {
        RepeaterPane::Cli => match key.code {
            KeyCode::Tab => Action::RepeaterNextPane,
            KeyCode::Enter => Action::RepeaterCliSubmit,
            KeyCode::Backspace => Action::RepeaterCliBackspace,
            KeyCode::Char(c) => Action::RepeaterCliChar(c),
            _ => Action::NoOp,
        },
        _ => match key.code {
            KeyCode::Tab => Action::RepeaterNextPane,
            KeyCode::Char('r') => match repeater.pane {
                RepeaterPane::Status => Action::RepeaterRefreshStatus,
                RepeaterPane::Neighbours => Action::RepeaterRefreshNeighbours,
                RepeaterPane::Acl => Action::RepeaterRefreshAcl,
                RepeaterPane::Cli => Action::NoOp,
            },
            KeyCode::Char('c') => Action::RepeaterFallbackToCli,
            KeyCode::Char('L') => Action::RepeaterLogout,
            _ => Action::NoOp,
        },
    }
}

fn map_modal_key(key: KeyEvent, ui: &AppUiState) -> Action {
    match ui.top_modal() {
        Some(ModalKind::ConfirmDeleteContact { pubkey, .. }) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Action::ContactsConfirmDelete(pubkey.clone())
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CloseModal,
            _ => Action::NoOp,
        },
        Some(ModalKind::HelpOverlay) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
                Action::CloseModal
            }
            _ => Action::NoOp,
        },
        Some(ModalKind::ConfirmDeleteChannel { idx, .. }) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Action::ChannelsConfirmDelete(*idx)
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CloseModal,
            _ => Action::NoOp,
        },
        Some(ModalKind::TcpConnect) => match key.code {
            KeyCode::Esc => Action::CloseModal,
            KeyCode::Enter => Action::ConnTcpSubmit,
            KeyCode::Backspace => Action::ConnTcpInputBackspace,
            KeyCode::Char(c) => Action::ConnTcpInputChar(c),
            _ => Action::NoOp,
        },
        Some(ModalKind::DeviceSetName) => match key.code {
            KeyCode::Esc => Action::CloseModal,
            KeyCode::Enter => Action::DeviceSubmitName,
            KeyCode::Backspace => Action::DeviceNameInputBackspace,
            KeyCode::Char(c) => Action::DeviceNameInputChar(c),
            _ => Action::NoOp,
        },
        Some(ModalKind::DeviceSetTxPower) => match key.code {
            KeyCode::Esc => Action::CloseModal,
            KeyCode::Enter => Action::DeviceSubmitTxPower,
            KeyCode::Char('+') | KeyCode::Up => Action::DeviceTxPowerInc,
            KeyCode::Char('-') | KeyCode::Down => Action::DeviceTxPowerDec,
            _ => Action::NoOp,
        },
        Some(ModalKind::ConfirmReboot) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                Action::DeviceConfirmReboot
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Action::CloseModal,
            _ => Action::NoOp,
        },
        Some(ModalKind::ChannelEdit { .. }) => match key.code {
            KeyCode::Esc => Action::CloseModal,
            KeyCode::Enter => Action::ChannelsEditSubmit,
            // Tab ne fait QUE changer de champ
            KeyCode::Tab => Action::ChannelsEditNextField,
            KeyCode::BackTab => Action::ChannelsEditPrevField,
            // Space toggle uniquement sur le champ Notifications
            KeyCode::Char(' ') => Action::ChannelsEditToggleNotifications,
            KeyCode::F(2) => Action::ChannelsEditSyncAndSubmit,
            KeyCode::Backspace => Action::ChannelsEditNameBackspace,
            KeyCode::Char(c) => Action::ChannelsEditNameChar(c),
            _ => Action::NoOp,
        },
        Some(ModalKind::RepeaterAdmin { .. }) => {
            // Routé via map_event_with_repeater depuis app.rs
            Action::NoOp
        }
        None => Action::NoOp,
    }
}

fn map_tab_key(key: KeyEvent, ui: &AppUiState) -> Action {
    // Globales prioritaires (jamais bloquées)
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Action::Quit;
    }

    // Raccourcis tabs — Alt+1..5 toujours actif (ne passe pas dans les inputs)
    if key.modifiers.contains(KeyModifiers::ALT)
        && let Some(tab) = digit_to_tab(key.code)
    {
        return Action::GotoTab(tab);
    }

    // Touches F1..F5 toujours actives (peuvent être capturées par certains terminaux)
    match key.code {
        KeyCode::F(1) => return Action::GotoTab(Tab::Connection),
        KeyCode::F(2) => return Action::GotoTab(Tab::Contacts),
        KeyCode::F(3) => return Action::GotoTab(Tab::Chat),
        KeyCode::F(4) => return Action::GotoTab(Tab::Channels),
        KeyCode::F(5) => return Action::GotoTab(Tab::Device),
        KeyCode::Tab => return Action::FocusNext,
        KeyCode::BackTab => return Action::FocusPrev,
        _ => {}
    }

    // Chiffres 1..5 = tabs, SAUF quand on saisit du texte (inputs TCP, chat, etc.)
    let in_text_input = matches!(
        ui.focus,
        crate::state::FocusTarget::ConnTcpInput | crate::state::FocusTarget::ChatInput
    );
    if !in_text_input {
        if let Some(tab) = digit_to_tab(key.code) {
            return Action::GotoTab(tab);
        }
        match key.code {
            KeyCode::Char('?') => return Action::OpenModal(ModalKind::HelpOverlay),
            KeyCode::Char('q') => return Action::Quit,
            _ => {}
        }
    }

    // Per-tab bindings
    match ui.current_tab {
        Tab::Connection => map_connection_key(key, ui),
        Tab::Contacts => map_contacts_key(key, ui),
        Tab::Chat => map_chat_key(key, ui),
        Tab::Channels => map_channels_key(key, ui),
        Tab::Device => map_device_key(key, ui),
    }
}

fn map_device_key(key: KeyEvent, _ui: &AppUiState) -> Action {
    match key.code {
        KeyCode::Char('R') => Action::DeviceRefresh,
        KeyCode::Char('n') => Action::DeviceRequestSetName,
        KeyCode::Char('p') => Action::DeviceRequestSetTxPower,
        KeyCode::Char('t') => Action::DeviceSyncTime,
        KeyCode::Char('c') => Action::DeviceCycleChemistry,
        KeyCode::Char('b') => Action::DeviceRefreshBattery,
        KeyCode::Char('a') => Action::DeviceSendAdvert { flood: false },
        KeyCode::Char('A') => Action::DeviceSendAdvert { flood: true },
        KeyCode::Char('B') => Action::DeviceRequestReboot,
        _ => Action::NoOp,
    }
}

fn map_chat_key(key: KeyEvent, ui: &AppUiState) -> Action {
    // Touches globales de la tab
    match key.code {
        KeyCode::PageUp => return Action::ChatLoadOlder,
        KeyCode::PageDown => return Action::ChatScrollDown,
        _ => {}
    }

    match ui.focus {
        crate::state::FocusTarget::ChatInput => match key.code {
            KeyCode::Enter => Action::ChatSend,
            KeyCode::Backspace => Action::ChatInputBackspace,
            KeyCode::Left => Action::ChatInputLeft,
            KeyCode::Right => Action::ChatInputRight,
            KeyCode::Char(c) => Action::ChatInputChar(c),
            KeyCode::Esc => Action::FocusPrev,
            _ => Action::NoOp,
        },
        crate::state::FocusTarget::ChatList => match key.code {
            KeyCode::Up => Action::ChatSelectPrev,
            KeyCode::Down => Action::ChatSelectNext,
            KeyCode::Enter => Action::ChatOpenSelected,
            KeyCode::Char('r') => Action::ChatRefreshConversations,
            _ => Action::NoOp,
        },
        crate::state::FocusTarget::ChatHistory => match key.code {
            KeyCode::Up => Action::ChatScrollUp,
            KeyCode::Down => Action::ChatScrollDown,
            _ => Action::NoOp,
        },
        _ => Action::NoOp,
    }
}

fn map_channels_key(key: KeyEvent, _ui: &AppUiState) -> Action {
    match key.code {
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Enter => Action::Enter,
        KeyCode::Char('e') => Action::ChannelsRequestEdit,
        KeyCode::Char('r') => Action::ChannelsMarkRead,
        KeyCode::Char('s') => Action::ChannelsSyncToDevice,
        KeyCode::Char('d') => Action::ChannelsRequestDelete,
        KeyCode::Char('R') => Action::ChannelsRefresh,
        _ => Action::NoOp,
    }
}

fn digit_to_tab(code: KeyCode) -> Option<Tab> {
    match code {
        KeyCode::Char('1') => Some(Tab::Connection),
        KeyCode::Char('2') => Some(Tab::Contacts),
        KeyCode::Char('3') => Some(Tab::Chat),
        KeyCode::Char('4') => Some(Tab::Channels),
        KeyCode::Char('5') => Some(Tab::Device),
        _ => None,
    }
}

fn map_connection_key(key: KeyEvent, ui: &AppUiState) -> Action {
    if matches!(ui.focus, FocusTarget::ConnTcpInput) {
        return match key.code {
            KeyCode::Enter => Action::ConnTcpSubmit,
            KeyCode::Backspace => Action::ConnTcpInputBackspace,
            KeyCode::Char(c) => Action::ConnTcpInputChar(c),
            KeyCode::Esc => Action::FocusPrev,
            _ => Action::NoOp,
        };
    }
    match key.code {
        // Navigation entre sous-panneaux : flèches horizontales
        KeyCode::Left => Action::ConnPrevSubPane,
        KeyCode::Right => Action::ConnNextSubPane,

        // Navigation dans la liste
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,

        // Actions contextuelles
        KeyCode::Char('s') => Action::ConnScanCurrent, // scan pane actif
        KeyCode::Char('r') => Action::ConnRefreshList,
        KeyCode::Char('R') => Action::ConnReconnectLast, // reconnect au dernier companion
        KeyCode::Char('d') => Action::ConnDisconnectPrimary,
        KeyCode::Enter => Action::ConnConnectSelected,

        _ => Action::NoOp,
    }
}

fn map_contacts_key(key: KeyEvent, _ui: &AppUiState) -> Action {
    match key.code {
        KeyCode::Char('s') => Action::ContactsSync,
        KeyCode::Char('f') => Action::ContactsToggleFav,
        KeyCode::Char('d') => Action::ContactsRequestDelete,
        KeyCode::Char('r') => Action::ContactsRefresh,
        KeyCode::Char('t') => Action::ContactsCycleSort,
        // R majuscule sur un contact repeater : ouvre la modale admin
        KeyCode::Char('R') => Action::ContactsOpenRepeater,
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Enter => Action::Enter,
        _ => Action::NoOp,
    }
}
