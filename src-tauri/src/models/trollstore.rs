use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrollStorePrepareRequest {
    pub saved_dir: String,
    /// Optional override for testing — if Some, skip GitHub API call and use this tag directly.
    pub force_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrollStorePrepareResult {
    pub version: String,
    pub tar_path: String,
    pub helper_path: String,
    pub cached: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrollStoreEligibilityRequest {
    pub product_type: Option<String>,
    pub ios_version: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum TrollStorePath {
    #[serde(rename = "ios14-15-ramdisk")]
    Ios14To15Ramdisk,
    #[serde(rename = "ios16-trollrestore")]
    Ios16PlusTrollRestore,
    #[serde(rename = "incompatible")]
    Incompatible,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrollStoreEligibilityResult {
    pub path: TrollStorePath,
    pub reason: String,
    pub ios_major: Option<u32>,
}
