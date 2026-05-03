//! Log persistence service for post-mortem debugging.
//!
//! Writes log entries to `<workspace>/logs/legacykit.log` with automatic rotation
//! when the file exceeds 5 MiB. Keeps the last 5 rotated files.

use crate::error::AppError;
use crate::services::workspace;
use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

/// Maximum log file size before rotation (5 MiB).
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// Number of rotated files to keep (legacykit.1.log through legacykit.5.log).
const MAX_ROTATED_FILES: usize = 5;

/// Log file name.
const LOG_FILE_NAME: &str = "legacykit.log";

/// Global log writer state.
static LOG_WRITER: OnceLock<Mutex<Option<LogWriter>>> = OnceLock::new();

/// Wrapper for the buffered file writer with path tracking.
struct LogWriter {
    writer: BufWriter<File>,
    path: PathBuf,
}

/// Initialize the log persistence system.
/// Must be called once at app startup with a valid AppHandle.
pub fn init(app: &AppHandle) -> Result<PathBuf, AppError> {
    let layout = workspace::get_layout(app)?;
    let logs_dir = layout.logs_dir();

    // Ensure the logs directory exists.
    fs::create_dir_all(&logs_dir)?;

    let log_path = logs_dir.join(LOG_FILE_NAME);

    // Rotate if the existing file is too large.
    if log_path.exists() {
        if let Ok(metadata) = fs::metadata(&log_path) {
            if metadata.len() >= MAX_FILE_SIZE {
                rotate_logs(&logs_dir)?;
            }
        }
    }

    // Open the log file in append mode.
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let writer = LogWriter {
        writer: BufWriter::new(file),
        path: log_path.clone(),
    };

    // Store in global state.
    let global_writer = LOG_WRITER.get_or_init(|| Mutex::new(None));
    let mut guard = global_writer
        .lock()
        .map_err(|_| AppError::Parse("Log writer mutex poisoned".into()))?;
    *guard = Some(writer);

    Ok(log_path)
}

/// Append a log entry to the log file.
/// This function is non-blocking on errors - it will log failures to stderr
/// and continue without interrupting the application.
pub fn append(kind: &str, text: &str) {
    let global_writer = match LOG_WRITER.get() {
        Some(w) => w,
        None => {
            eprintln!("[log_persist] Writer not initialized");
            return;
        }
    };

    let mut guard = match global_writer.lock() {
        Ok(g) => g,
        Err(_) => {
            eprintln!("[log_persist] Failed to acquire lock");
            return;
        }
    };

    let writer = match guard.as_mut() {
        Some(w) => w,
        None => {
            eprintln!("[log_persist] No writer available");
            return;
        }
    };

    // Format: [YYYY-MM-DD HH:MM:SS LEVEL] message
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let line = format!("[{} {}] {}\n", timestamp, kind, text);

    // Write to buffer.
    if let Err(e) = writer.writer.write_all(line.as_bytes()) {
        eprintln!("[log_persist] Write error: {}", e);
        return;
    }

    // Flush to disk.
    if let Err(e) = writer.writer.flush() {
        eprintln!("[log_persist] Flush error: {}", e);
    }

    // Check if rotation is needed after writing.
    if let Ok(metadata) = fs::metadata(&writer.path) {
        if metadata.len() >= MAX_FILE_SIZE {
            if let Err(e) = rotate_and_reopen(writer) {
                eprintln!("[log_persist] Rotation error: {}", e);
            }
        }
    }
}

/// Get the current log file path.
/// Returns None if the persistence system hasn't been initialized.
pub fn get_log_file_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let layout = workspace::get_layout(app)?;
    Ok(layout.logs_dir().join(LOG_FILE_NAME))
}

/// Rotate log files: legacykit.log -> legacykit.1.log, etc.
/// Deletes the oldest file when MAX_ROTATED_FILES is exceeded.
fn rotate_logs(logs_dir: &PathBuf) -> Result<(), AppError> {
    // Delete the oldest rotated file if it exists.
    let oldest = logs_dir.join(format!("legacykit.{}.log", MAX_ROTATED_FILES));
    if oldest.exists() {
        fs::remove_file(&oldest)?;
    }

    // Shift existing rotated files: .N -> .N+1
    for i in (1..MAX_ROTATED_FILES).rev() {
        let current = logs_dir.join(format!("legacykit.{}.log", i));
        let next = logs_dir.join(format!("legacykit.{}.log", i + 1));
        if current.exists() {
            fs::rename(&current, &next)?;
        }
    }

    // Rename current log to .1
    let current = logs_dir.join(LOG_FILE_NAME);
    let first_rotated = logs_dir.join("legacykit.1.log");
    if current.exists() {
        fs::rename(&current, &first_rotated)?;
    }

    Ok(())
}

/// Rotate logs and reopen the writer with a new file.
fn rotate_and_reopen(writer: &mut LogWriter) -> Result<(), AppError> {
    let logs_dir = writer
        .path
        .parent()
        .ok_or_else(|| AppError::Parse("Log file has no parent directory".into()))?
        .to_path_buf();

    rotate_logs(&logs_dir)?;

    // Open a new log file.
    let new_path = logs_dir.join(LOG_FILE_NAME);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&new_path)?;

    writer.writer = BufWriter::new(file);
    writer.path = new_path;

    Ok(())
}
