use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JustBootEntry {
    pub id: String,
    pub ecid: String,
    pub product_type: String,
    pub device_name: Option<String>,
    pub build_id: String,
    pub ios_version: Option<String>,
    pub boot_args: Option<String>,
    pub repacked_ibss_path: Option<String>,
    pub repacked_ibec_path: Option<String>,
    pub source_ipsw_path: Option<String>,
    pub created_at: String,
    pub last_booted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JustBootEntryInput {
    pub ecid: String,
    pub product_type: String,
    pub device_name: Option<String>,
    pub build_id: String,
    pub ios_version: Option<String>,
    pub boot_args: Option<String>,
    pub repacked_ibss_path: Option<String>,
    pub repacked_ibec_path: Option<String>,
    pub source_ipsw_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareAndJustBootRequest {
    pub ecid: String,
    pub product_type: String,
    pub device_name: Option<String>,
    pub build_id: String,
    pub ios_version: Option<String>,
    pub ipsw_path: String,
    pub boot_args: Option<String>,
    pub processor_generation: Option<String>,
    pub include_ibec: bool,
}
