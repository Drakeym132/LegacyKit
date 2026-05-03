use crate::error::AppError;
use crate::models::device::DeviceInfo;
use crate::models::restore::{
    CancelIpswDownloadRequest, CancelIpswDownloadResult, CheckIpswSigningRequest,
    CheckIpswSigningResult, FirmwareListEntry, FirmwareListRequest, FirmwareListResult,
    IpswDownloadRequest, IpswDownloadResult, IpswPrepareRequest, IpswPrepareResult,
    IpswVerifyRequest, IpswVerifyResult, RestoreCommandPreview, RestoreOptionsResponse,
    RestoreRunRequest, RestoreTool,
};
use crate::platform::resolve_binary_path;
use crate::services::ipsw_prep;
use crate::services::restore_options::determine_restore_options;
use crate::services::sha1::sha1_file;
use crate::services::workspace;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

const FIRMWARE_CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct IpswDownloadProgressEvent {
    download_id: String,
    percent: Option<f64>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
    speed_bps: Option<u64>,
    eta_seconds: Option<u64>,
}

#[derive(Clone)]
struct FirmwareCacheEntry {
    fetched_at_unix: i64,
    inserted_at: Instant,
    firmwares: Vec<FirmwareListEntry>,
}

static ACTIVE_DOWNLOAD_PIDS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
static FIRMWARE_CACHE: OnceLock<Mutex<HashMap<String, FirmwareCacheEntry>>> = OnceLock::new();

fn active_download_pids() -> &'static Mutex<HashMap<String, u32>> {
    ACTIVE_DOWNLOAD_PIDS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn firmware_cache() -> &'static Mutex<HashMap<String, FirmwareCacheEntry>> {
    FIRMWARE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub async fn get_restore_options(device: DeviceInfo) -> Result<RestoreOptionsResponse, AppError> {
    Ok(determine_restore_options(device))
}

#[tauri::command]
pub async fn list_firmwares(request: FirmwareListRequest) -> Result<FirmwareListResult, AppError> {
    let device_identifier = request.device_identifier.trim().to_string();
    if device_identifier.is_empty() {
        return Err(AppError::Parse(
            "Device identifier is required (for example: iPhone3,1)".to_string(),
        ));
    }

    if let Some(cached) = cached_firmwares_for(&device_identifier) {
        return Ok(cached);
    }

    let url = format!("https://api.ipsw.me/v4/device/{device_identifier}?type=ipsw");
    let response_body = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "LegacyKit/1.0")
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Firmware lookup request failed: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::CommandFailed(format!("Firmware lookup failed: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed reading firmware payload: {e}")))?;

    let payload: Value = serde_json::from_str(&response_body)
        .map_err(|e| AppError::Parse(format!("Invalid firmware payload JSON: {e}")))?;
    let mut firmwares = parse_firmwares_from_payload(&payload)?;
    firmwares.sort_by(|a, b| {
        b.version
            .cmp(&a.version)
            .then_with(|| b.build_id.cmp(&a.build_id))
    });

    let fetched_at_unix = Utc::now().timestamp();
    {
        let mut cache = firmware_cache()
            .lock()
            .map_err(|_| AppError::CommandFailed("Failed to lock firmware cache".to_string()))?;
        cache.insert(
            device_identifier.clone(),
            FirmwareCacheEntry {
                fetched_at_unix,
                inserted_at: Instant::now(),
                firmwares: firmwares.clone(),
            },
        );
    }

    Ok(FirmwareListResult {
        device_identifier,
        fetched_at_unix,
        cached: false,
        firmwares,
    })
}

#[tauri::command]
pub async fn check_ipsw_signing(
    app: AppHandle,
    request: CheckIpswSigningRequest,
) -> Result<CheckIpswSigningResult, AppError> {
    let device_identifier = request.device_identifier.trim().to_string();
    let build_id = request.build_id.trim().to_string();
    if device_identifier.is_empty() {
        return Err(AppError::Parse("Device identifier is required".to_string()));
    }
    if build_id.is_empty() {
        return Err(AppError::Parse("Build ID is required".to_string()));
    }

    let binary = resolve_binary_path(&app, "tsschecker").map_err(AppError::CommandFailed)?;
    let args = vec![
        "-d".to_string(),
        device_identifier.clone(),
        "-Z".to_string(),
        build_id.clone(),
    ];

    let output = Command::new(&binary)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!(
        "{}{}{}",
        stdout,
        if stderr.is_empty() { "" } else { "\n" },
        stderr
    )
    .trim()
    .to_string();

    let signed = output.status.success();
    crate::tools::runner::emit_log(
        &app,
        if signed { "info" } else { "warn" },
        &format!(
            "Signing check {} for {} {}",
            if signed { "passed" } else { "failed" },
            device_identifier,
            build_id
        ),
    );

    Ok(CheckIpswSigningResult {
        device_identifier,
        build_id,
        signed,
        output: combined,
    })
}

#[tauri::command]
pub async fn cancel_ipsw_download(
    request: CancelIpswDownloadRequest,
) -> Result<CancelIpswDownloadResult, AppError> {
    let download_id = request.download_id.trim().to_string();
    if download_id.is_empty() {
        return Err(AppError::Parse("downloadId is required".to_string()));
    }

    let pid = {
        let mut guard = active_download_pids().lock().map_err(|_| {
            AppError::CommandFailed("Failed to lock active download registry".to_string())
        })?;
        guard.remove(&download_id)
    };

    let Some(pid) = pid else {
        return Ok(CancelIpswDownloadResult {
            download_id,
            cancelled: false,
        });
    };

    let status = Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .status()?;

    Ok(CancelIpswDownloadResult {
        download_id,
        cancelled: status.success(),
    })
}

#[tauri::command]
pub async fn download_ipsw(
    app: AppHandle,
    request: IpswDownloadRequest,
) -> Result<IpswDownloadResult, AppError> {
    let url = request.url.trim();
    if url.is_empty() {
        return Err(AppError::Parse("Download URL is required".to_string()));
    }

    let output_dir = {
        let requested = request.output_dir.trim();
        if requested.is_empty() {
            let layout = workspace::get_layout(&app)?;
            let dir = layout.ipsw_dir(request.device_identifier.as_deref());
            layout.ensure_dir(dir)?
        } else {
            let dir = PathBuf::from(requested);
            fs::create_dir_all(&dir)?;
            dir
        }
    };

    let download_id = request
        .download_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let file_name = request
        .file_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| file_name_from_url(url))
        .ok_or_else(|| AppError::Parse("Unable to infer IPSW filename from URL".to_string()))?;

    if !file_name.ends_with(".ipsw") {
        return Err(AppError::Parse(
            "IPSW download filename must end with .ipsw".to_string(),
        ));
    }
    if Path::new(&file_name)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(file_name.as_str())
    {
        return Err(AppError::Parse(
            "IPSW download filename cannot include path separators".to_string(),
        ));
    }

    crate::tools::runner::emit_log(&app, "info", &format!("Downloading {file_name}"));
    let aria2c = resolve_binary_path(&app, "aria2c").map_err(AppError::CommandFailed)?;
    let args = vec![
        "--continue=true".to_string(),
        "--allow-overwrite=true".to_string(),
        "--max-connection-per-server=8".to_string(),
        "--split=8".to_string(),
        "--summary-interval=1".to_string(),
        "--dir".to_string(),
        output_dir.to_string_lossy().to_string(),
        "--out".to_string(),
        file_name.clone(),
        url.to_string(),
    ];

    run_aria2_with_progress(&app, &download_id, &aria2c, &args)?;

    let path = output_dir.join(file_name);
    if !path.exists() {
        return Err(AppError::CommandFailed(format!(
            "Download finished but {} was not created",
            path.display()
        )));
    }

    let calculated_sha1 = sha1_file(path.to_string_lossy().as_ref())?;
    let expected_sha1 = request
        .expected_sha1
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let sha1_matches = expected_sha1
        .as_ref()
        .map(|expected| expected == &calculated_sha1);

    Ok(IpswDownloadResult {
        path: path.to_string_lossy().to_string(),
        sha1: calculated_sha1,
        expected_sha1,
        sha1_matches,
        download_id,
    })
}

#[tauri::command]
pub async fn verify_ipsw(request: IpswVerifyRequest) -> Result<IpswVerifyResult, AppError> {
    let path = request.path.trim();
    if path.is_empty() {
        return Err(AppError::Parse("IPSW path is required".to_string()));
    }
    if !Path::new(path).exists() {
        return Err(AppError::Parse(format!("IPSW does not exist: {path}")));
    }

    let calculated_sha1 = sha1_file(path)?;
    let expected_sha1 = request
        .expected_sha1
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let matches = expected_sha1
        .as_ref()
        .map(|expected| expected == &calculated_sha1);

    Ok(IpswVerifyResult {
        path: path.to_string(),
        calculated_sha1,
        expected_sha1,
        matches,
    })
}

#[tauri::command]
pub async fn preview_restore_command(
    request: RestoreRunRequest,
) -> Result<RestoreCommandPreview, AppError> {
    build_restore_command(&request)
}

#[tauri::command]
pub async fn start_restore(
    app: AppHandle,
    request: RestoreRunRequest,
) -> Result<RestoreCommandPreview, AppError> {
    let preview = build_restore_command(&request)?;
    if request.dry_run {
        crate::tools::runner::emit_log(
            &app,
            "info",
            "Dry run requested; restore command was not started",
        );
        return Ok(preview);
    }

    let binary_path =
        resolve_binary_path(&app, &preview.binary).map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Starting {} {}", preview.binary, preview.args.join(" ")),
    );
    crate::tools::runner::run_streaming(&app, binary_path, &preview.args)?;
    crate::tools::runner::emit_log(&app, "info", "Restore tool finished");

    Ok(preview)
}

#[tauri::command]
pub async fn prepare_ipsw(
    app: AppHandle,
    request: IpswPrepareRequest,
) -> Result<IpswPrepareResult, AppError> {
    let ipsw_path = request.ipsw_path.trim();
    if ipsw_path.is_empty() {
        return Err(AppError::Parse("Source IPSW path is required".to_string()));
    }
    if !Path::new(ipsw_path).exists() {
        return Err(AppError::Parse(format!(
            "Source IPSW does not exist: {ipsw_path}"
        )));
    }

    let output_dir = {
        let requested = request.output_dir.trim();
        if requested.is_empty() {
            let layout = workspace::get_layout(&app)?;
            let dir = layout.custom_ipsw_dir(request.device_identifier.as_deref());
            layout.ensure_dir(dir)?
        } else {
            let dir = PathBuf::from(requested);
            fs::create_dir_all(&dir)?;
            dir
        }
    };

    let shsh_path = request
        .shsh_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(shsh) = shsh_path {
        if !Path::new(shsh).exists() {
            return Err(AppError::Parse(format!("SHSH blob does not exist: {shsh}")));
        }
    }

    let ecid = request
        .device_ecid
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let output_path =
        ipsw_prep::powdersn0w_output_path(ipsw_path, output_dir.to_string_lossy().as_ref())?;
    let args = ipsw_prep::build_powdersn0w_args(ipsw_path, &output_path, shsh_path, ecid);

    let binary = resolve_binary_path(&app, "powdersn0w").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(&app, "info", "Preparing custom IPSW with powdersn0w...");
    crate::tools::runner::run_streaming(&app, binary, &args)?;

    if !output_path.exists() {
        return Err(AppError::CommandFailed(format!(
            "powdersn0w finished but output IPSW was not created at {}",
            output_path.display()
        )));
    }

    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Custom IPSW ready: {}", output_path.display()),
    );
    Ok(IpswPrepareResult {
        output_path: output_path.to_string_lossy().to_string(),
    })
}

fn build_restore_command(request: &RestoreRunRequest) -> Result<RestoreCommandPreview, AppError> {
    let ipsw_path = request.ipsw_path.trim();
    if ipsw_path.is_empty() {
        return Err(AppError::Parse("Target IPSW path is required".to_string()));
    }
    if !Path::new(ipsw_path).exists() {
        return Err(AppError::Parse(format!("IPSW does not exist: {ipsw_path}")));
    }

    let mut warnings = Vec::new();
    let (binary, args) = match request.tool {
        RestoreTool::IdeviceRestore => {
            let mut args = Vec::new();
            if request.update {
                args.push("-u".to_string());
            } else if request.erase {
                args.push("-e".to_string());
            }
            args.push(ipsw_path.to_string());
            ("idevicerestore".to_string(), args)
        }
        RestoreTool::FutureRestore => {
            let shsh_path = request
                .shsh_path
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Parse("FutureRestore requires a target SHSH blob".to_string())
                })?;
            if !Path::new(shsh_path).exists() {
                return Err(AppError::Parse(format!(
                    "SHSH blob does not exist: {shsh_path}"
                )));
            }

            let mut args = Vec::new();
            if request.no_baseband {
                args.push("--no-baseband".to_string());
            }
            if request.latest_sep {
                args.push("--latest-sep".to_string());
            }
            if request.latest_baseband {
                args.push("--latest-baseband".to_string());
            }
            if request.use_pwndfu {
                args.push("--use-pwndfu".to_string());
                warnings.push(
                    "Pwned DFU restore assumes the device is already in the required state."
                        .to_string(),
                );
            }
            if request.skip_blob {
                args.push("--skip-blob".to_string());
            }
            if request.set_nonce {
                args.push("--set-nonce".to_string());
            }
            args.push("-t".to_string());
            args.push(shsh_path.to_string());
            args.push(ipsw_path.to_string());
            ("futurerestore_new".to_string(), args)
        }
    };

    if matches!(request.tool, RestoreTool::IdeviceRestore) && !request.erase && !request.update {
        warnings.push(
            "idevicerestore will run without erase/update flags; confirm this is intended."
                .to_string(),
        );
    }

    Ok(RestoreCommandPreview {
        supported: true,
        tool: request.tool.clone(),
        binary,
        args,
        warnings,
    })
}

fn file_name_from_url(url: &str) -> Option<String> {
    strip_url_query_and_fragment(url)
        .split('/')
        .next_back()
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
}

fn strip_url_query_and_fragment(url: &str) -> &str {
    let trimmed = url.trim();
    let no_query = trimmed.split('?').next().unwrap_or(trimmed);
    no_query.split('#').next().unwrap_or(no_query)
}

fn cached_firmwares_for(device_identifier: &str) -> Option<FirmwareListResult> {
    let mut cache = firmware_cache().lock().ok()?;
    let entry = cache.get(device_identifier)?.clone();
    if entry.inserted_at.elapsed() > FIRMWARE_CACHE_TTL {
        cache.remove(device_identifier);
        return None;
    }
    Some(FirmwareListResult {
        device_identifier: device_identifier.to_string(),
        fetched_at_unix: entry.fetched_at_unix,
        cached: true,
        firmwares: entry.firmwares,
    })
}

fn parse_firmwares_from_payload(payload: &Value) -> Result<Vec<FirmwareListEntry>, AppError> {
    let entries = payload
        .get("firmwares")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Parse("Firmware payload missing `firmwares` array".to_string()))?;

    let mut out = Vec::with_capacity(entries.len());
    for item in entries {
        let Some(url) = item.get("url").and_then(Value::as_str).map(str::trim) else {
            continue;
        };
        let Some(version) = item.get("version").and_then(Value::as_str).map(str::trim) else {
            continue;
        };
        let Some(build_id) = item.get("buildid").and_then(Value::as_str).map(str::trim) else {
            continue;
        };

        out.push(FirmwareListEntry {
            version: version.to_string(),
            build_id: build_id.to_string(),
            url: url.to_string(),
            sha1: item
                .get("sha1sum")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_ascii_lowercase()),
            size_bytes: parse_filesize(item.get("filesize")),
            signed: item.get("signed").and_then(Value::as_bool),
        });
    }
    Ok(out)
}

fn parse_filesize(value: Option<&Value>) -> Option<u64> {
    match value {
        Some(Value::Number(n)) => n.as_u64(),
        Some(Value::String(s)) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn run_aria2_with_progress(
    app: &AppHandle,
    download_id: &str,
    binary: &Path,
    args: &[String],
) -> Result<(), AppError> {
    let mut child = Command::new(binary)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut guard = active_download_pids().lock().map_err(|_| {
            AppError::CommandFailed("Failed to register active IPSW download".to_string())
        })?;
        guard.insert(download_id.to_string(), child.id());
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture aria2 stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::CommandFailed("Failed to capture aria2 stderr".to_string()))?;

    let stdout_app = app.clone();
    let stdout_download_id = download_id.to_string();
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            crate::tools::runner::emit_log(&stdout_app, "stdout", &line);
            if let Some(progress) = parse_aria2_progress_line(&line) {
                let payload = IpswDownloadProgressEvent {
                    download_id: stdout_download_id.clone(),
                    percent: progress.percent,
                    downloaded_bytes: progress.downloaded_bytes,
                    total_bytes: progress.total_bytes,
                    speed_bps: progress.speed_bps,
                    eta_seconds: progress.eta_seconds,
                };
                let _ = stdout_app.emit("ipsw-download-progress", payload);
            }
        }
    });

    let stderr_app = app.clone();
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            crate::tools::runner::emit_log(&stderr_app, "stderr", &line);
        }
    });

    let status = child.wait()?;
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();
    let mut guard = active_download_pids().lock().map_err(|_| {
        AppError::CommandFailed("Failed to clean active IPSW download registry".to_string())
    })?;
    guard.remove(download_id);

    if !status.success() {
        return Err(AppError::CommandFailed(format!(
            "aria2c exited with status {status}"
        )));
    }

    Ok(())
}

#[derive(Default)]
struct ParsedAria2Progress {
    percent: Option<f64>,
    downloaded_bytes: Option<u64>,
    total_bytes: Option<u64>,
    speed_bps: Option<u64>,
    eta_seconds: Option<u64>,
}

fn parse_aria2_progress_line(line: &str) -> Option<ParsedAria2Progress> {
    let size_idx = line.find("SIZE:")?;
    let size_slice = &line[size_idx + 5..];
    let slash_idx = size_slice.find('/')?;
    let downloaded_raw = size_slice[..slash_idx].trim();

    let after_slash = &size_slice[slash_idx + 1..];
    let total_end = after_slash.find('(').unwrap_or(after_slash.len());
    let total_raw = after_slash[..total_end].trim();

    let percent = if let Some(open) = line.find('(') {
        let pct_segment = &line[open + 1..];
        let pct_token = pct_segment.split('%').next().unwrap_or("").trim();
        pct_token.parse::<f64>().ok()
    } else {
        None
    };

    let speed_bps = line
        .find("DL:")
        .and_then(|idx| line.get(idx + 3..))
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(parse_speed_to_bps);

    let eta_seconds = line
        .find("ETA:")
        .and_then(|idx| line.get(idx + 4..))
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(parse_eta_seconds);

    Some(ParsedAria2Progress {
        percent,
        downloaded_bytes: parse_size_to_bytes(downloaded_raw),
        total_bytes: parse_size_to_bytes(total_raw),
        speed_bps,
        eta_seconds,
    })
}

fn parse_size_to_bytes(raw: &str) -> Option<u64> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }

    let split_at = token
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(token.len());
    let number_part = &token[..split_at];
    let unit_part = token[split_at..].trim().to_ascii_lowercase();
    let value = number_part.parse::<f64>().ok()?;

    let multiplier = match unit_part.as_str() {
        "" | "b" => 1_f64,
        "k" | "kb" | "kib" => 1024_f64,
        "m" | "mb" | "mib" => 1024_f64 * 1024_f64,
        "g" | "gb" | "gib" => 1024_f64 * 1024_f64 * 1024_f64,
        "t" | "tb" | "tib" => 1024_f64 * 1024_f64 * 1024_f64 * 1024_f64,
        _ => return None,
    };

    Some((value * multiplier).round() as u64)
}

fn parse_speed_to_bps(raw: &str) -> Option<u64> {
    parse_size_to_bytes(raw.trim_end_matches("/s"))
}

fn parse_eta_seconds(raw: &str) -> Option<u64> {
    let value = raw.trim();
    if value.is_empty() || value == "--" {
        return None;
    }

    if value.contains(':') {
        let parts: Vec<_> = value.split(':').collect();
        return match parts.len() {
            2 => {
                let mins = parts[0].parse::<u64>().ok()?;
                let secs = parts[1].parse::<u64>().ok()?;
                Some(mins * 60 + secs)
            }
            3 => {
                let hours = parts[0].parse::<u64>().ok()?;
                let mins = parts[1].parse::<u64>().ok()?;
                let secs = parts[2].parse::<u64>().ok()?;
                Some(hours * 3600 + mins * 60 + secs)
            }
            _ => None,
        };
    }

    if let Some(stripped) = value.strip_suffix('s') {
        return stripped.parse::<u64>().ok();
    }
    if let Some(stripped) = value.strip_suffix('m') {
        return stripped.parse::<u64>().ok().map(|m| m * 60);
    }
    if let Some(stripped) = value.strip_suffix('h') {
        return stripped.parse::<u64>().ok().map(|h| h * 3600);
    }

    value.parse::<u64>().ok()
}
