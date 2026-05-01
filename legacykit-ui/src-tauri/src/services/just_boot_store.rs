use crate::error::AppError;
use crate::models::just_boot::JustBootEntry;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const HISTORY_FILE_NAME: &str = "just_boot_history.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JustBootHistory {
    pub entries: Vec<JustBootEntry>,
}

pub fn load(app: &AppHandle) -> Result<JustBootHistory, AppError> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(JustBootHistory::default());
    }

    let raw = fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(JustBootHistory::default());
    }

    serde_json::from_str::<JustBootHistory>(&raw).map_err(|err| {
        AppError::Parse(format!(
            "Failed to parse {}: {}",
            path.to_string_lossy(),
            err
        ))
    })
}

pub fn list(app: &AppHandle) -> Result<Vec<JustBootEntry>, AppError> {
    Ok(load(app)?.entries)
}

pub fn upsert(app: &AppHandle, mut entry: JustBootEntry) -> Result<JustBootEntry, AppError> {
    let mut history = load(app)?;
    let now = now_rfc3339();
    entry.ecid = normalize_ecid(&entry.ecid);
    entry.last_booted_at = now;

    if let Some(existing) = history
        .entries
        .iter_mut()
        .find(|existing| existing.ecid == entry.ecid && existing.build_id == entry.build_id)
    {
        entry.id = existing.id.clone();
        entry.created_at = existing.created_at.clone();
        *existing = entry.clone();
    } else {
        history.entries.push(entry.clone());
    }

    save(app, &history)?;
    Ok(entry)
}

pub fn touch(app: &AppHandle, id: &str) -> Result<(), AppError> {
    let mut history = load(app)?;
    let now = now_rfc3339();
    if let Some(entry) = history.entries.iter_mut().find(|entry| entry.id == id) {
        entry.last_booted_at = now;
        save(app, &history)?;
    }
    Ok(())
}

pub fn remove(app: &AppHandle, id: &str) -> Result<(), AppError> {
    let mut history = load(app)?;
    history.entries.retain(|entry| entry.id != id);
    save(app, &history)
}

pub fn clear_for_ecid(app: &AppHandle, ecid: &str) -> Result<(), AppError> {
    let normalized = normalize_ecid(ecid);
    let mut history = load(app)?;
    history.entries.retain(|entry| entry.ecid != normalized);
    save(app, &history)
}

fn save(app: &AppHandle, history: &JustBootHistory) -> Result<(), AppError> {
    let path = history_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_vec_pretty(history)
        .map_err(|err| AppError::Parse(format!("Failed to serialize just boot history: {err}")))?;
    write_atomic(&path, &serialized)
}

fn history_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| AppError::CommandFailed(format!("Failed to get app data dir: {err}")))?;
    Ok(data_dir.join(HISTORY_FILE_NAME))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::Parse("History path has no parent directory".to_string()))?;
    fs::create_dir_all(parent)?;

    let tmp_name = format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("history"),
        std::process::id()
    );
    let tmp_path = parent.join(tmp_name);

    fs::write(&tmp_path, bytes)?;
    fs::rename(&tmp_path, path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        AppError::Io(err)
    })
}

fn normalize_ecid(ecid: &str) -> String {
    ecid.trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_ascii_lowercase()
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}
