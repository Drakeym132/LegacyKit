use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DeviceMode {
    #[default]
    Normal,
    Recovery,
    DFU,
    #[serde(rename = "kDFU")]
    KDFU,
    #[serde(rename = "pwnDFU")]
    PwnDFU,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    pub connected: bool,
    pub name: Option<String>,
    pub udid: Option<String>,
    pub ecid: Option<String>,
    pub serial: Option<String>,
    pub model: Option<String>,
    pub product_type: Option<String>,
    pub ios_version: Option<String>,
    pub mode: DeviceMode,

    // Extended ideviceinfo fields (Normal Mode)
    pub build_version: Option<String>,
    pub cpu_architecture: Option<String>,
    pub hardware_platform: Option<String>,
    pub device_color: Option<String>,
    pub device_class: Option<String>,
    pub model_number: Option<String>,
    pub region_info: Option<String>,
    pub activation_state: Option<String>,
    pub baseband_version: Option<String>,
    pub firmware_version: Option<String>,
    pub total_disk_capacity: Option<u64>,
    pub total_data_available: Option<u64>,
    pub battery_current_capacity: Option<i64>,
    pub password_protected: Option<bool>,
    pub telephony_capability: Option<bool>,
    pub imei: Option<String>,
    pub wifi_address: Option<String>,
    pub bluetooth_address: Option<String>,

    // Extended irecovery fields (Recovery/DFU Mode)
    pub cpid: Option<String>,
    pub cprv: Option<String>,
    pub bdid: Option<String>,
    pub ibfl: Option<String>,
    pub apnonce: Option<String>,
    pub sepnonce: Option<String>,
    pub pwnd: Option<String>,
    pub srtg: Option<String>,
}
