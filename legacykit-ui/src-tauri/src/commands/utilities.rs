use crate::error::AppError;
use crate::models::utilities::{
    ActivationAction, ActivationRequest, ActivationResult, CommandRunResult, DiagnosticsAction,
    DiagnosticsRequest, DiagnosticsResult, ExportInfoKind, ExportInfoRequest, ExportInfoResult,
    IrecoveryCommandRequest, IrecoveryCommandResult, PairAction, PairRequest, PairResult,
    SyslogStartRequest, SyslogStatusResult, UdidRequest,
};
use crate::platform::resolve_binary_path;
use serde::Serialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter};

static SYSLOG_CHILD: Mutex<Option<Child>> = Mutex::new(None);

#[tauri::command]
pub async fn enter_recovery(
    app: AppHandle,
    request: UdidRequest,
) -> Result<CommandRunResult, AppError> {
    let binary =
        resolve_binary_path(&app, "ideviceenterrecovery").map_err(AppError::CommandFailed)?;
    let udid = nullable(request.udid.as_deref()).ok_or_else(|| {
        AppError::Parse("UDID is required to enter recovery (device must be in Normal mode)".into())
    })?;
    let args = vec![udid.to_string()];
    emit_log(&app, "info", &format!("Entering recovery mode for {udid}"));
    run_process_streaming(&app, binary, &args)?;
    Ok(CommandRunResult { args })
}

#[tauri::command]
pub async fn exit_recovery(app: AppHandle) -> Result<CommandRunResult, AppError> {
    let binary = resolve_binary_path(&app, "irecovery").map_err(AppError::CommandFailed)?;
    let args = vec!["-n".to_string()];
    emit_log(&app, "info", "Exiting recovery mode (irecovery -n)");
    run_process_streaming(&app, binary, &args)?;
    Ok(CommandRunResult { args })
}

#[tauri::command]
pub async fn run_diagnostics_action(
    app: AppHandle,
    request: DiagnosticsRequest,
) -> Result<DiagnosticsResult, AppError> {
    let binary =
        resolve_binary_path(&app, "idevicediagnostics").map_err(AppError::CommandFailed)?;
    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.push(
        match request.action {
            DiagnosticsAction::Shutdown => "shutdown",
            DiagnosticsAction::Restart => "restart",
            DiagnosticsAction::Sleep => "sleep",
        }
        .to_string(),
    );
    emit_log(
        &app,
        "info",
        &format!("idevicediagnostics {:?}", request.action),
    );
    run_process_streaming(&app, binary, &args)?;
    Ok(DiagnosticsResult {
        action: request.action,
        args,
    })
}

#[tauri::command]
pub async fn pair_device(
    app: AppHandle,
    request: PairRequest,
) -> Result<PairResult, AppError> {
    let binary = resolve_binary_path(&app, "idevicepair").map_err(AppError::CommandFailed)?;
    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.push(
        match request.action {
            PairAction::Pair => "pair",
            PairAction::Unpair => "unpair",
            PairAction::Validate => "validate",
        }
        .to_string(),
    );
    emit_log(&app, "info", &format!("idevicepair {:?}", request.action));
    run_process_streaming(&app, binary, &args)?;
    Ok(PairResult {
        action: request.action,
        args,
    })
}

#[tauri::command]
pub async fn run_activation_action(
    app: AppHandle,
    request: ActivationRequest,
) -> Result<ActivationResult, AppError> {
    let binary =
        resolve_binary_path(&app, "ideviceactivation").map_err(AppError::CommandFailed)?;
    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    let action_arg = match request.action {
        ActivationAction::Activate => "activate",
        ActivationAction::Deactivate => "deactivate",
        ActivationAction::State => "state",
    };
    args.push(action_arg.to_string());

    emit_log(
        &app,
        "info",
        &format!("ideviceactivation {action_arg}"),
    );

    if matches!(request.action, ActivationAction::State) {
        let output = Command::new(&binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(AppError::CommandFailed(if stderr.is_empty() {
                format!("ideviceactivation state exited with {}", output.status)
            } else {
                stderr
            }));
        }
        let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
        emit_log(&app, "stdout", &state);
        return Ok(ActivationResult {
            action: request.action,
            state: Some(state),
            args,
        });
    }

    run_process_streaming(&app, binary, &args)?;
    Ok(ActivationResult {
        action: request.action,
        state: None,
        args,
    })
}

#[tauri::command]
pub async fn export_device_info(
    app: AppHandle,
    request: ExportInfoRequest,
) -> Result<ExportInfoResult, AppError> {
    let output_dir = request.output_dir.trim();
    if output_dir.is_empty() {
        return Err(AppError::Parse("Output directory is required".into()));
    }
    fs::create_dir_all(output_dir)?;

    let (binary_name, base_args, label_default) = match request.kind {
        ExportInfoKind::DeviceInfo => ("ideviceinfo", Vec::<String>::new(), "device-info"),
        ExportInfoKind::BatteryInfo => (
            "idevicediagnostics",
            vec!["ioregentry".into(), "AppleSmartBattery".into()],
            "battery-info",
        ),
        ExportInfoKind::DiagnosticsAll => (
            "idevicediagnostics",
            vec!["diagnostics".into(), "All".into()],
            "diagnostics-all",
        ),
    };
    let binary = resolve_binary_path(&app, binary_name).map_err(AppError::CommandFailed)?;

    let mut args: Vec<String> = Vec::new();
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    args.extend(base_args);

    let stamp = format_timestamp_dir();
    let label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(label_default);
    let filename = format!("{label}-{stamp}.txt");
    let out_path = Path::new(output_dir).join(&filename);

    emit_log(
        &app,
        "info",
        &format!("Exporting {label} → {}", out_path.display()),
    );

    let output = Command::new(&binary)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::CommandFailed(if stderr.is_empty() {
            format!("{binary_name} exited with {}", output.status)
        } else {
            stderr
        }));
    }
    fs::write(&out_path, &output.stdout)?;
    let bytes = output.stdout.len() as u64;
    emit_log(
        &app,
        "info",
        &format!("Exported {bytes} bytes to {}", out_path.display()),
    );

    Ok(ExportInfoResult {
        path: out_path.to_string_lossy().to_string(),
        bytes,
    })
}

#[tauri::command]
pub async fn run_irecovery_commands(
    app: AppHandle,
    request: IrecoveryCommandRequest,
) -> Result<IrecoveryCommandResult, AppError> {
    if request.commands.is_empty() {
        return Err(AppError::Parse("At least one irecovery command is required".into()));
    }
    let binary = resolve_binary_path(&app, "irecovery").map_err(AppError::CommandFailed)?;
    let mut all_args: Vec<String> = Vec::new();

    for cmd in &request.commands {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            continue;
        }
        let args = vec!["-c".to_string(), trimmed.to_string()];
        emit_log(&app, "info", &format!("irecovery -c {trimmed:?}"));
        run_process_streaming(&app, binary.clone(), &args)?;
        all_args.extend(args);
    }

    if request.reboot_after {
        let args = vec!["-n".to_string()];
        emit_log(&app, "info", "irecovery -n (reboot)");
        run_process_streaming(&app, binary, &args)?;
        all_args.extend(args);
    }

    Ok(IrecoveryCommandResult {
        args: all_args,
        commands: request.commands,
    })
}

/// Convenience wrapper: clears NVRAM via irecovery (must be in Recovery mode).
#[tauri::command]
pub async fn clear_nvram(app: AppHandle) -> Result<IrecoveryCommandResult, AppError> {
    run_irecovery_commands(
        app,
        IrecoveryCommandRequest {
            commands: vec!["setenv auto-boot true".into(), "saveenv".into()],
            reboot_after: true,
        },
    )
    .await
}

#[tauri::command]
pub async fn start_syslog(
    app: AppHandle,
    request: SyslogStartRequest,
) -> Result<SyslogStatusResult, AppError> {
    {
        let guard = SYSLOG_CHILD
            .lock()
            .map_err(|_| AppError::CommandFailed("Syslog mutex poisoned".into()))?;
        if guard.is_some() {
            return Err(AppError::CommandFailed(
                "Syslog is already running. Stop it first.".into(),
            ));
        }
    }

    let binary = resolve_binary_path(&app, "idevicesyslog").map_err(AppError::CommandFailed)?;
    let mut args: Vec<String> = vec!["-q".into()];
    if let Some(udid) = nullable(request.udid.as_deref()) {
        args.push("-u".into());
        args.push(udid.to_string());
    }
    emit_log(&app, "info", "Starting idevicesyslog");

    let mut child = Command::new(&binary)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let pid = child.id();

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture syslog stdout".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture syslog stderr".into()))?;

    let app_out = app.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            emit_log_with_event(&app_out, "syslog_event", "stdout", &line);
        }
    });
    let app_err = app.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            emit_log_with_event(&app_err, "syslog_event", "stderr", &line);
        }
    });

    {
        let mut guard = SYSLOG_CHILD
            .lock()
            .map_err(|_| AppError::CommandFailed("Syslog mutex poisoned".into()))?;
        *guard = Some(child);
    }

    Ok(SyslogStatusResult {
        running: true,
        pid: Some(pid),
    })
}

#[tauri::command]
pub async fn stop_syslog(app: AppHandle) -> Result<SyslogStatusResult, AppError> {
    let mut guard = SYSLOG_CHILD
        .lock()
        .map_err(|_| AppError::CommandFailed("Syslog mutex poisoned".into()))?;
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
        emit_log(&app, "info", "Stopped idevicesyslog");
    }
    Ok(SyslogStatusResult {
        running: false,
        pid: None,
    })
}

#[tauri::command]
pub async fn syslog_status() -> Result<SyslogStatusResult, AppError> {
    let guard = SYSLOG_CHILD
        .lock()
        .map_err(|_| AppError::CommandFailed("Syslog mutex poisoned".into()))?;
    Ok(match guard.as_ref() {
        Some(child) => SyslogStatusResult {
            running: true,
            pid: Some(child.id()),
        },
        None => SyslogStatusResult {
            running: false,
            pid: None,
        },
    })
}

fn nullable(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn run_process_streaming(
    app: &AppHandle,
    binary: PathBuf,
    args: &[String],
) -> Result<(), AppError> {
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
        return Err(AppError::CommandFailed(format!(
            "{} exited with status {}",
            binary.display(),
            status
        )));
    }
    Ok(())
}

fn emit_log(app: &AppHandle, level: &str, text: &str) {
    emit_log_with_event(app, "log_event", level, text);
}

fn emit_log_with_event(app: &AppHandle, event: &str, level: &str, text: &str) {
    let payload = LogEventPayload {
        text: text.to_string(),
        kind: level.to_string(),
    };
    let _ = app.emit(event, payload);
}

#[derive(Clone, Serialize)]
struct LogEventPayload {
    text: String,
    #[serde(rename = "type")]
    kind: String,
}

fn format_timestamp_dir() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (year, month, day, hour, minute) = unix_to_components(now);
    format!("{year:04}-{month:02}-{day:02}-{hour:02}{minute:02}")
}

fn unix_to_components(secs: u64) -> (u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let seconds_of_day = (secs % 86_400) as u32;
    let hour = seconds_of_day / 3600;
    let minute = (seconds_of_day % 3600) / 60;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year as u32, m as u32, d as u32, hour, minute)
}
