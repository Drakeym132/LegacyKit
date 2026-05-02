use crate::error::AppError;
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct LogEventPayload {
    pub text: String,
    #[serde(rename = "type")]
    pub kind: String,
}

/// Emit a log_event payload to the frontend (`log_event` channel).
pub fn emit_log(app: &AppHandle, kind: &str, text: &str) {
    let _ = app.emit(
        "log_event",
        LogEventPayload {
            text: text.to_string(),
            kind: kind.to_string(),
        },
    );
}

/// Run a child process, streaming its stdout/stderr to the frontend as
/// `log_event` payloads (kind = "stdout" / "stderr"), and waiting for exit.
/// Returns Err(AppError::CommandFailed) if the process exits non-zero.
pub fn run_streaming(app: &AppHandle, binary: PathBuf, args: &[String]) -> Result<(), AppError> {
    let mut child = Command::new(&binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture process stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture process stderr".to_string()))?;

    let stdout_app = app.clone();
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            emit_log(&stdout_app, "stdout", &line);
        }
    });

    let stderr_app = app.clone();
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            emit_log(&stderr_app, "stderr", &line);
        }
    });

    let status = child.wait()?;
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    if !status.success() {
        let msg = format!(
            "{} {} exited with status {}",
            binary.display(),
            args.join(" "),
            status
        );
        emit_log(app, "stderr", &msg);
        return Err(AppError::CommandFailed(msg));
    }

    emit_log(app, "info", &format!("{} finished ok", binary.display()));
    Ok(())
}
