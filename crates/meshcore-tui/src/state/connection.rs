use crate::action::{BleDevice, ConnectionInfo, ConnectionSubPane};
use ratatui::widgets::ListState;

pub struct ConnectionUiState {
    pub sub_pane: ConnectionSubPane,
    pub ble_scanning: bool,
    pub ble_devices: Vec<BleDevice>,
    pub ble_list_state: ListState,

    pub serial_scanning: bool,
    pub serial_ports: Vec<String>,
    pub serial_list_state: ListState,

    pub tcp_input: String,

    pub active_connections: Vec<ConnectionInfo>,
    pub active_list_state: ListState,
}

impl ConnectionUiState {
    pub fn new() -> Self {
        let mut s = Self {
            sub_pane: ConnectionSubPane::BleScan,
            ble_scanning: false,
            ble_devices: Vec::new(),
            ble_list_state: ListState::default(),
            serial_scanning: false,
            serial_ports: Vec::new(),
            serial_list_state: ListState::default(),
            tcp_input: String::from("192.168.1.50:4403"),
            active_connections: Vec::new(),
            active_list_state: ListState::default(),
        };
        s.ble_list_state.select(Some(0));
        s.serial_list_state.select(Some(0));
        s.active_list_state.select(Some(0));
        s
    }

    pub fn move_selection(&mut self, delta: i32) {
        let (state, len) = match self.sub_pane {
            ConnectionSubPane::BleScan => (&mut self.ble_list_state, self.ble_devices.len()),
            ConnectionSubPane::SerialList => {
                (&mut self.serial_list_state, self.serial_ports.len())
            }
            ConnectionSubPane::Active => (
                &mut self.active_list_state,
                self.active_connections.len(),
            ),
            ConnectionSubPane::TcpInput => return,
        };
        if len == 0 {
            state.select(None);
            return;
        }
        let current = state.selected().unwrap_or(0) as i32;
        let new = (current + delta).clamp(0, len as i32 - 1) as usize;
        state.select(Some(new));
    }
}

impl Default for ConnectionUiState {
    fn default() -> Self {
        Self::new()
    }
}
