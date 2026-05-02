use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreOptionKind {
    OtaDowngrade,
    Powdersnow,
    Latest,
    BlobRestore,
    Tethered,
    CustomIpsw,
    DfuIpsw,
    SetNonce,
    IpswDownloader,
    MoreVersions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOption {
    pub kind: RestoreOptionKind,
    pub title: String,
    pub description: String,
    pub target_version: Option<String>,
    pub requires_blobs: bool,
    pub requires_dfu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOptionsResponse {
    pub product_type: Option<String>,
    pub processor_generation: Option<u8>,
    pub options: Vec<RestoreOption>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswDownloadRequest {
    pub url: String,
    pub output_dir: String,
    pub device_identifier: Option<String>,
    pub file_name: Option<String>,
    pub expected_sha1: Option<String>,
    pub download_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswDownloadResult {
    pub path: String,
    pub sha1: String,
    pub expected_sha1: Option<String>,
    pub sha1_matches: Option<bool>,
    pub download_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareListRequest {
    pub device_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareListEntry {
    pub version: String,
    pub build_id: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size_bytes: Option<u64>,
    pub signed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareListResult {
    pub device_identifier: String,
    pub fetched_at_unix: i64,
    pub cached: bool,
    pub firmwares: Vec<FirmwareListEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckIpswSigningRequest {
    pub device_identifier: String,
    pub build_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckIpswSigningResult {
    pub device_identifier: String,
    pub build_id: String,
    pub signed: bool,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelIpswDownloadRequest {
    pub download_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelIpswDownloadResult {
    pub download_id: String,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswVerifyRequest {
    pub path: String,
    pub expected_sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswVerifyResult {
    pub path: String,
    pub calculated_sha1: String,
    pub expected_sha1: Option<String>,
    pub matches: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RestoreTool {
    IdeviceRestore,
    FutureRestore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRunRequest {
    pub tool: RestoreTool,
    pub ipsw_path: String,
    pub shsh_path: Option<String>,
    pub erase: bool,
    pub update: bool,
    pub use_pwndfu: bool,
    pub skip_blob: bool,
    pub set_nonce: bool,
    pub no_baseband: bool,
    pub latest_sep: bool,
    pub latest_baseband: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreCommandPreview {
    pub supported: bool,
    pub tool: RestoreTool,
    pub binary: String,
    pub args: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswPrepareRequest {
    pub ipsw_path: String,
    pub output_dir: String,
    pub device_identifier: Option<String>,
    pub shsh_path: Option<String>,
    pub device_ecid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpswPrepareResult {
    pub output_path: String,
}
