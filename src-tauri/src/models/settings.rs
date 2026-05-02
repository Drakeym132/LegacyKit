use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub workspace_root: Option<PathBuf>,
    pub onboarded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWorkspaceRootRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacePaths {
    pub root: String,
    pub ipsw: String,
    pub ipsw_custom: String,
    pub shsh: String,
    pub extracted: String,
    pub ssh_binaries: String,
    pub backups: String,
    pub logs: String,
    pub tmp: String,
}
