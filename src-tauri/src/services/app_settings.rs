use crate::error::AppError;
use crate::models::settings::AppSettings;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE_NAME: &str = "settings.json";

pub fn load(app: &AppHandle) -> Result<AppSettings, AppError> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let raw = fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(AppSettings::default());
    }

    serde_json::from_str::<AppSettings>(&raw).map_err(|err| {
        AppError::Parse(format!(
            "Failed to parse {}: {}",
            path.to_string_lossy(),
            err
        ))
    })
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_vec_pretty(settings)
        .map_err(|err| AppError::Parse(format!("Failed to serialize app settings: {err}")))?;
    write_atomic(&path, &serialized)
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|err| AppError::CommandFailed(format!("Failed to get app config dir: {err}")))?;
    Ok(config_dir.join(SETTINGS_FILE_NAME))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Parse("Settings path has no parent directory".to_string()))?;
    fs::create_dir_all(parent)?;

    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("settings"),
        std::process::id()
    );
    let tmp_path = parent.join(tmp_name);

    fs::write(&tmp_path, bytes)?;
    fs::rename(&tmp_path, path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        AppError::Io(err)
    })
}

