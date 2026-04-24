use meshcorex_protocol::types::BatteryChemistry;
use meshcorex_service::device::DeviceInfoSummary;

pub struct DeviceUiState {
    pub info: Option<DeviceInfoSummary>,
    pub chemistry: BatteryChemistry,
    pub battery_mv: Option<u16>,
    pub battery_percent: Option<u8>,
    pub refreshing: bool,
    /// Input en cours dans les modales (set name, tx power)
    pub name_input: String,
    pub tx_power_draft: u8,
}

impl DeviceUiState {
    pub fn new() -> Self {
        Self {
            info: None,
            chemistry: BatteryChemistry::LiPo,
            battery_mv: None,
            battery_percent: None,
            refreshing: false,
            name_input: String::new(),
            tx_power_draft: 20,
        }
    }

    pub fn cycle_chemistry(&mut self) {
        self.chemistry = match self.chemistry {
            BatteryChemistry::LiPo => BatteryChemistry::LiFePO4,
            BatteryChemistry::LiFePO4 => BatteryChemistry::NiMH,
            BatteryChemistry::NiMH => BatteryChemistry::LiPo,
        };
    }

    pub fn chemistry_label(&self) -> &'static str {
        match self.chemistry {
            BatteryChemistry::LiPo => "LiPo",
            BatteryChemistry::LiFePO4 => "LiFePO4",
            BatteryChemistry::NiMH => "NiMH",
        }
    }
}

impl Default for DeviceUiState {
    fn default() -> Self {
        Self::new()
    }
}
