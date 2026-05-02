use crate::error::AppError;
use crate::models::data::{
    BackupCreateRequest, BackupCreateResult, BackupEncryptionRequest, BackupEncryptionResult,
    BackupEntry, BackupRestoreRequest, BackupRestoreResult, EraseDeviceRequest, EraseDeviceResult,
    ListBackupsRequest, ListBackupsResult,
};
use crate::platform::resolve_binary_path;
use crate::tools::util::{nullable, timestamp_dir_now};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::AppHandle;

const ERASE_CONFIRMATION: &str = "Yes, do as I say";

#[tauri::command]
pub async fn create_backup(
    app: AppHandle,
    request: BackupCreateRequest,
) -> Result<BackupCreateResult, AppError> {
    let backup_root = request.backup_root.trim();
    if backup_root.is_empty() {
        return Err(AppError::Parse("Backup root directory is required".into()));
    }
    fs::create_dir_all(backup_root)?;

    let stamp = timestamp_dir_now();
    let backup_path = Path::new(backup_root).join(&stamp);
    fs::create_dir_all(&backup_path)?;

    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.push("backup".into());
    if request.full {
        args.push("--full".into());
    }
    args.push(backup_path.to_string_lossy().to_string());

    let binary = resolve_binary_path(&app, "idevicebackup2").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Starting idevicebackup2 backup → {}", backup_path.display()),
    );
    crate::tools::runner::run_streaming(&app, binary, &args)?;
    crate::tools::runner::emit_log(&app, "info", "Backup completed");

    Ok(BackupCreateResult {
        backup_path: backup_path.to_string_lossy().to_string(),
        args,
    })
}

#[tauri::command]
pub async fn restore_backup(
    app: AppHandle,
    request: BackupRestoreRequest,
) -> Result<BackupRestoreResult, AppError> {
    let backup_path = request.backup_path.trim();
    if backup_path.is_empty() {
        return Err(AppError::Parse("Backup path is required".into()));
    }
    if !Path::new(backup_path).exists() {
        return Err(AppError::Parse(format!(
            "Backup path does not exist: {backup_path}"
        )));
    }

    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.push("restore".into());
    if request.system {
        args.push("--system".into());
    }
    if request.settings {
        args.push("--settings".into());
    }
    if request.reboot {
        args.push("--reboot".into());
    }
    args.push(backup_path.to_string());

    let binary = resolve_binary_path(&app, "idevicebackup2").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Restoring backup from {backup_path}"),
    );
    crate::tools::runner::run_streaming(&app, binary, &args)?;
    crate::tools::runner::emit_log(&app, "info", "Restore completed");

    Ok(BackupRestoreResult {
        backup_path: backup_path.to_string(),
        args,
    })
}

#[tauri::command]
pub async fn erase_device(
    app: AppHandle,
    request: EraseDeviceRequest,
) -> Result<EraseDeviceResult, AppError> {
    if request.confirmation.trim() != ERASE_CONFIRMATION {
        return Err(AppError::Parse(format!(
            "Confirmation phrase must exactly match: {ERASE_CONFIRMATION}"
        )));
    }

    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.push("erase".into());

    let binary = resolve_binary_path(&app, "idevicebackup2").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "warn",
        "Erasing device (Erase All Content and Settings)",
    );
    crate::tools::runner::run_streaming(&app, binary, &args)?;
    crate::tools::runner::emit_log(&app, "info", "Erase request issued");

    Ok(EraseDeviceResult { args })
}

#[tauri::command]
pub async fn set_backup_encryption(
    app: AppHandle,
    request: BackupEncryptionRequest,
) -> Result<BackupEncryptionResult, AppError> {
    let mut args: Vec<String> = vec!["-i".into()];
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.extend(request.action.as_args());

    let binary = resolve_binary_path(&app, "idevicebackup2").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Backup encryption: {:?}", request.action),
    );
    crate::tools::runner::run_streaming(&app, binary, &args)?;

    Ok(BackupEncryptionResult {
        action: request.action,
        args,
    })
}

#[tauri::command]
pub async fn list_backups(request: ListBackupsRequest) -> Result<ListBackupsResult, AppError> {
    let backup_root = request.backup_root.trim().to_string();
    if backup_root.is_empty() {
        return Err(AppError::Parse("Backup root directory is required".into()));
    }
    let root = Path::new(&backup_root);
    if !root.exists() {
        return Ok(ListBackupsResult {
            backup_root,
            backups: Vec::new(),
        });
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let metadata = entry.metadata()?;
        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            continue;
        }
        backups.push(BackupEntry {
            path: path.to_string_lossy().to_string(),
            name,
            size_bytes: dir_size(&path).unwrap_or(0),
            modified_unix,
        });
    }

    backups.sort_by_key(|b| std::cmp::Reverse(b.modified_unix));
    Ok(ListBackupsResult {
        backup_root,
        backups,
    })
}

fn dir_size(dir: &Path) -> Option<u64> {
    let mut total: u64 = 0;
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(sub) = dir_size(&path) {
                total += sub;
            }
        } else if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    Some(total)
}
