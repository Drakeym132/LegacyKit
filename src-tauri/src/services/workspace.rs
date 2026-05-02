use crate::error::AppError;
use crate::services::app_settings;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct WorkspaceLayout {
    root: PathBuf,
}

impl WorkspaceLayout {
    pub fn from_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ipsw_dir(&self, device: Option<&str>) -> PathBuf {
        let base = self.root.join("ipsw");
        match sanitize_component(device) {
            Some(dev) => base.join(dev),
            None => base.join("unknown-device"),
        }
    }

    pub fn custom_ipsw_dir(&self, device: Option<&str>) -> PathBuf {
        let base = self.root.join("ipsw-custom");
        match sanitize_component(device) {
            Some(dev) => base.join(dev),
            None => base.join("unknown-device"),
        }
    }

    pub fn shsh_dir(&self, ecid: Option<&str>) -> PathBuf {
        let base = self.root.join("shsh");
        match sanitize_component(ecid) {
            Some(id) => base.join(id),
            None => base.join("unknown-ecid"),
        }
    }

    pub fn extracted_dir(&self, device: Option<&str>, build: Option<&str>) -> PathBuf {
        let dev = sanitize_component(device).unwrap_or_else(|| "unknown-device".to_string());
        let bld = sanitize_component(build).unwrap_or_else(|| "unknown-build".to_string());
        self.root.join("extracted").join(format!("{dev}_{bld}"))
    }

    pub fn ssh_binaries_dir(&self) -> PathBuf {
        self.root.join("ssh-binaries")
    }

    pub fn backups_dir(&self, device: Option<&str>) -> PathBuf {
        let base = self.root.join("backups");
        match sanitize_component(device) {
            Some(dev) => base.join(dev),
            None => base.join("unknown-device"),
        }
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn tmp_dir(&self) -> PathBuf {
        self.root.join("tmp")
    }

    pub fn tools_dir(&self) -> PathBuf {
        self.root.join("tools")
    }

    pub fn ensure_layout(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(self.root.join("ipsw"))?;
        fs::create_dir_all(self.root.join("ipsw-custom"))?;
        fs::create_dir_all(self.root.join("shsh"))?;
        fs::create_dir_all(self.root.join("extracted"))?;
        fs::create_dir_all(self.ssh_binaries_dir())?;
        fs::create_dir_all(self.root.join("backups"))?;
        fs::create_dir_all(self.logs_dir())?;
        fs::create_dir_all(self.tmp_dir())?;
        fs::create_dir_all(self.tools_dir())?;
        Ok(())
    }

    pub fn ensure_dir<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, AppError> {
        let pb = path.as_ref().to_path_buf();
        fs::create_dir_all(&pb)?;
        Ok(pb)
    }
}

pub fn get_layout(app: &AppHandle) -> Result<WorkspaceLayout, AppError> {
    let settings = app_settings::load(app)?;
    let root = settings
        .workspace_root
        .ok_or_else(|| AppError::Parse("Workspace is not configured yet".to_string()))?;
    Ok(WorkspaceLayout::from_root(root))
}

fn sanitize_component(value: Option<&str>) -> Option<String> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}
