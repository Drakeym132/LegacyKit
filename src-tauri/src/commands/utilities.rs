use crate::error::AppError;
use crate::models::utilities::{
    ActivationAction, ActivationRequest, ActivationResult, CommandRunResult, DiagnosticsAction,
    DiagnosticsRequest, DiagnosticsResult, ExportInfoKind, ExportInfoRequest, ExportInfoResult,
    IrecoveryCommandRequest, IrecoveryCommandResult, PairAction, PairRequest, PairResult,
    SyslogStartRequest, SyslogStatusResult, UdidRequest,
};
use crate::platform::resolve_binary_path;
use crate::tools::util::{nullable, timestamp_dir_now};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
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
    crate::tools::runner::emit_log(&app, "info", &format!("Entering recovery mode for {udid}"));
    crate::tools::runner::run_streaming(&app, binary, &args)?;
    Ok(CommandRunResult { args })
}

#[tauri::command]
pub async fn exit_recovery(app: AppHandle) -> Result<CommandRunResult, AppError> {
    let binary = resolve_binary_path(&app, "irecovery").map_err(AppError::CommandFailed)?;
    let args = vec!["-n".to_string()];
    crate::tools::runner::emit_log(&app, "info", "Exiting recovery mode (irecovery -n)");
    crate::tools::runner::run_streaming(&app, binary, &args)?;
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
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("idevicediagnostics {:?}", request.action),
    );
    crate::tools::runner::run_streaming(&app, binary, &args)?;
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
    crate::tools::runner::emit_log(&app, "info", &format!("idevicepair {:?}", request.action));
    crate::tools::runner::run_streaming(&app, binary, &args)?;
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

    crate::tools::runner::emit_log(
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
        crate::tools::runner::emit_log(&app, "stdout", &state);
        return Ok(ActivationResult {
            action: request.action,
            state: Some(state),
            args,
        });
    }

    crate::tools::runner::run_streaming(&app, binary, &args)?;
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

    let stamp = timestamp_dir_now();
    let label = request
        .label
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(label_default);
    let filename = format!("{label}-{stamp}.txt");
    let out_path = Path::new(output_dir).join(&filename);

    crate::tools::runner::emit_log(
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
    crate::tools::runner::emit_log(
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
        crate::tools::runner::emit_log(&app, "info", &format!("irecovery -c {trimmed:?}"));
        crate::tools::runner::run_streaming(&app, binary.clone(), &args)?;
        all_args.extend(args);
    }

    if request.reboot_after {
        let args = vec!["-n".to_string()];
        crate::tools::runner::emit_log(&app, "info", "irecovery -n (reboot)");
        crate::tools::runner::run_streaming(&app, binary, &args)?;
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
    crate::tools::runner::emit_log(&app, "info", "Starting idevicesyslog");

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
        crate::tools::runner::emit_log(&app, "info", "Stopped idevicesyslog");
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

fn emit_log_with_event(app: &AppHandle, event: &str, level: &str, text: &str) {
    let payload = crate::tools::runner::LogEventPayload {
        text: text.to_string(),
        kind: level.to_string(),
    };
    let _ = app.emit(event, payload);
}
