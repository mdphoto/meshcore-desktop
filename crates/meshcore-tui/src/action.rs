use meshcore_service::AppEvent;
use meshcore_transport::manager::ConnectionTarget;

#[derive(Debug, Clone)]
pub enum Tab {
    Connection,
    Contacts,
    Chat,
    Channels,
    Device,
}

impl Tab {
    pub fn all() -> [Tab; 5] {
        [
            Tab::Connection,
            Tab::Contacts,
            Tab::Chat,
            Tab::Channels,
            Tab::Device,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Connection => 0,
            Tab::Contacts => 1,
            Tab::Chat => 2,
            Tab::Channels => 3,
            Tab::Device => 4,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Connection => "1 Connexion",
            Tab::Contacts => "2 Contacts",
            Tab::Chat => "3 Chat",
            Tab::Channels => "4 Canaux",
            Tab::Device => "5 Device",
        }
    }

    pub fn from_index(i: usize) -> Tab {
        match i % 5 {
            0 => Tab::Connection,
            1 => Tab::Contacts,
            2 => Tab::Chat,
            3 => Tab::Channels,
            _ => Tab::Device,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub enum ModalKind {
    ConfirmDeleteContact { pubkey: String, name: String },
    ConfirmDeleteChannel { idx: u8, name: String },
    HelpOverlay,
    TcpConnect,
    DeviceSetName,
    DeviceSetTxPower,
    ConfirmReboot,
    /// Édition d'un canal (nom + notifications)
    ChannelEdit { idx: u8 },
    /// Création d'un nouveau canal (nom + PSK hex)
    ChannelNew,
    /// Administration repeater plein-écran : pubkey du repeater ciblé + nom affiché
    RepeaterAdmin { pubkey: String, name: String },
}

#[derive(Debug, Clone)]
pub enum ConnectionSubPane {
    BleScan,
    SerialList,
    TcpInput,
    Active,
}

#[derive(Debug, Clone)]
pub enum AsyncResult {
    BleScanDone(Vec<BleDevice>),
    BleScanFailed(String),
    SerialScanDone(Vec<String>),
    SerialScanFailed(String),
    ContactsReloaded(Vec<meshcore_storage::models::StoredContact>),
    ContactsSyncDone(Result<usize, String>),
    ConnectionsListed(Vec<ConnectionInfo>),
    ChannelsReloaded(Vec<meshcore_storage::channels::StoredChannel>),
    DmPubkeysLoaded(Vec<String>),
    ChannelSenderNamesLoaded {
        channel_idx: u8,
        names: Vec<String>,
    },
    MessagesLoaded {
        conversation: crate::state::chat::ConversationId,
        messages: Vec<meshcore_storage::models::StoredMessage>,
        /// true si ces messages s'ajoutent en tête (pagination), false = replace complet
        prepend: bool,
        /// si prepend : nombre reçu < limit → plus rien à charger
        fully_loaded: bool,
    },
    DeviceInfoLoaded(Result<meshcore_service::device::DeviceInfoSummary, String>),
    BatteryLoaded(Result<(u16, u8), String>),
    RepeaterLoginResult(Result<String, String>),
    RepeaterStatusLoaded(Result<meshcore_service::repeater::RepeaterStatus, String>),
    RepeaterNeighboursLoaded(Result<Vec<meshcore_service::repeater::RepeaterNeighbour>, String>),
    RepeaterAclLoaded(Result<Vec<meshcore_service::repeater::AclEntry>, String>),
    RepeaterCliResult {
        command: String,
        result: Result<String, String>,
    },
    Generic(Result<String, String>),
}

#[derive(Debug, Clone)]
pub struct BleDevice {
    pub name: String,
    pub address: String,
    pub rssi: Option<i16>,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: String,
    pub label: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub enum Action {
    // Lifecycle
    Quit,
    NoOp,
    Tick,

    // Navigation globale
    NextTab,
    PrevTab,
    GotoTab(Tab),
    FocusNext,
    FocusPrev,

    // Navigation interne liste
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Escape,
    Char(char),
    Backspace,

    // Modales
    OpenModal(ModalKind),
    CloseModal,

    // Toast
    Toast(String, ToastLevel),
    ToastTick,

    // Contacts
    ContactsSync,
    ContactsRefresh,
    ContactsToggleFav,
    ContactsRequestDelete,
    ContactsConfirmDelete(String),
    ContactsCycleSort,

    // Chat
    ChatSelectPrev,
    ChatSelectNext,
    ChatOpenSelected,
    ChatFocusNext,
    ChatInputChar(char),
    ChatInputBackspace,
    ChatInputDelete,
    ChatInputLeft,
    ChatInputRight,
    ChatInputHome,
    ChatInputEnd,
    ChatInputDeletePrevWord,
    ChatInputClear,
    /// Navigation dans le popup @mention (si ouvert)
    ChatMentionNext,
    ChatMentionPrev,
    /// Insère le candidat sélectionné dans l'input et ferme le popup
    ChatMentionInsert,
    /// Ferme le popup sans modifier l'input (retire le `@` et la query saisis)
    ChatMentionCancel,
    ChatSend,
    ChatScrollUp,
    ChatScrollDown,
    ChatLoadOlder,
    ChatRefreshConversations,
    ChatOpenContact(String),

    // Channels
    ChannelsRefresh,
    ChannelsMarkRead,
    ChannelsSyncToDevice,
    ChannelsRequestDelete,
    ChannelsConfirmDelete(u8),
    ChannelsRequestEdit,
    ChannelsEditNameChar(char),
    ChannelsEditNameBackspace,
    ChannelsEditToggleNotifications,
    ChannelsEditNextField,
    ChannelsEditPrevField,
    ChannelsEditSubmit,
    ChannelsEditSyncAndSubmit,
    ChannelsEditCopyPsk,
    ChannelsRequestNew,
    ChannelsNewChar(char),
    ChannelsNewBackspace,
    ChannelsNewNextField,
    ChannelsNewPrevField,
    ChannelsNewGeneratePsk,
    ChannelsNewDeriveFromName,
    ChannelsNewSubmit,

    // Device
    DeviceRefresh,
    DeviceCycleChemistry,
    DeviceSyncTime,
    DeviceRequestSetName,
    DeviceSubmitName,
    DeviceNameInputChar(char),
    DeviceNameInputBackspace,
    DeviceRequestSetTxPower,
    DeviceSubmitTxPower,
    DeviceTxPowerInc,
    DeviceTxPowerDec,
    DeviceRequestReboot,
    DeviceConfirmReboot,
    DeviceSendAdvert { flood: bool },
    DeviceRefreshBattery,
    /// Déclencheur « R sur contact repeater sélectionné » — App résout le pubkey
    ContactsOpenRepeater,

    // Repeater
    RepeaterOpen { pubkey: String, name: String },
    RepeaterClose,
    RepeaterNextPane,
    RepeaterPasswordChar(char),
    RepeaterPasswordBackspace,
    RepeaterPasswordSubmit,
    RepeaterLogout,
    RepeaterRefreshStatus,
    RepeaterRefreshNeighbours,
    RepeaterRefreshAcl,
    RepeaterFallbackToCli,
    RepeaterCliChar(char),
    RepeaterCliBackspace,
    RepeaterCliSubmit,

    // Connection
    ConnSelectSubPane(ConnectionSubPane),
    ConnPrevSubPane,
    ConnNextSubPane,
    ConnScanCurrent,
    ConnBleScan,
    ConnSerialScan,
    ConnConnectSelected,
    ConnConnect(ConnectionTarget),
    ConnDisconnectPrimary,
    ConnDisconnectById(String),
    ConnSetPrimary(String),
    ConnRefreshList,
    ConnReconnectLast,
    ConnTcpInputChar(char),
    ConnTcpInputBackspace,
    ConnTcpSubmit,

    // Events backend / résultats async
    Backend(AppEvent),
    Async(AsyncResult),
}
