use crate::error::AppError;
use crate::models::device::DeviceMode;
use crate::platform::resolve_binary_path;
use crate::services::device_meta::infer_processor_gen;
use crate::services::device_parser::parse_irecovery_q;
use crate::services::external_tools::{ensure_pwn_tool, ExternalTool};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GasterAction {
    Pwn,
    Reset,
}

impl GasterAction {
    fn as_arg(&self) -> &'static str {
        match self {
            GasterAction::Pwn => "pwn",
            GasterAction::Reset => "reset",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasterRequest {
    pub action: GasterAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasterResult {
    pub action: GasterAction,
    pub binary: String,
    pub args: Vec<String>,
}

#[tauri::command]
pub async fn run_gaster(
    app: AppHandle,
    request: GasterRequest,
) -> Result<GasterResult, AppError> {
    let binary = resolve_binary_path(&app, "gaster").map_err(AppError::CommandFailed)?;
    let args = vec![request.action.as_arg().to_string()];

    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Running gaster {}...", request.action.as_arg()),
    );
    crate::tools::runner::run_streaming(&app, binary.clone(), &args)?;
    crate::tools::runner::emit_log(&app, "info", "gaster finished");

    Ok(GasterResult {
        action: request.action,
        binary: binary.to_string_lossy().to_string(),
        args,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KloaderRequest {
    pub ibss_path: String,
    pub ibec_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KloaderResult {
    pub binary: String,
    pub args: Vec<String>,
}

#[tauri::command]
pub async fn run_kloader(
    app: AppHandle,
    request: KloaderRequest,
) -> Result<KloaderResult, AppError> {
    let ibss_path = request.ibss_path.trim();
    if ibss_path.is_empty() {
        return Err(AppError::Parse("Patched iBSS path is required".to_string()));
    }
    if !Path::new(ibss_path).exists() {
        return Err(AppError::Parse(format!(
            "Patched iBSS does not exist: {ibss_path}"
        )));
    }

    let ibec_path = request
        .ibec_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(ibec) = ibec_path {
        if !Path::new(ibec).exists() {
            return Err(AppError::Parse(format!(
                "Patched iBEC does not exist: {ibec}"
            )));
        }
    }

    let mut args = vec![ibss_path.to_string()];
    if let Some(ibec) = ibec_path {
        args.push(ibec.to_string());
    }

    let binary = resolve_binary_path(&app, "kloader").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Booting patched components with kloader: {}", args.join(" ")),
    );
    crate::tools::runner::run_streaming(&app, binary.clone(), &args)?;
    crate::tools::runner::emit_log(&app, "info", "kloader finished");

    Ok(KloaderResult {
        binary: binary.to_string_lossy().to_string(),
        args,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UntetherRequest {
    /// Optional extra flags (e.g. `["-v"]`). Passed verbatim after any required positional args.
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UntetherResult {
    pub binary: String,
    pub args: Vec<String>,
}

#[tauri::command]
pub async fn run_g1lbertjb(
    app: AppHandle,
    request: UntetherRequest,
) -> Result<UntetherResult, AppError> {
    run_untether(&app, "g1lbertJB", request.extra_args).await
}

#[tauri::command]
pub async fn run_evasi0n(
    app: AppHandle,
    request: UntetherRequest,
) -> Result<UntetherResult, AppError> {
    run_untether(&app, "evasi0n", request.extra_args).await
}

async fn run_untether(
    app: &AppHandle,
    binary_name: &str,
    extra_args: Vec<String>,
) -> Result<UntetherResult, AppError> {
    let args: Vec<String> = extra_args
        .into_iter()
        .map(|arg| arg.trim().to_string())
        .filter(|arg| !arg.is_empty())
        .collect();

    let binary = resolve_binary_path(app, binary_name).map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Running {} {}", binary_name, args.join(" ")),
    );
    crate::tools::runner::run_streaming(app, binary.clone(), &args)?;
    crate::tools::runner::emit_log(app, "info", &format!("{binary_name} finished"));

    Ok(UntetherResult {
        binary: binary.to_string_lossy().to_string(),
        args,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterPwnDfuRequest {
    pub product_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterPwnDfuResult {
    /// The pwn tool that was actually run (e.g. "gaster", "ipwnder", "primepwn").
    pub tool: String,
    /// Args passed to the tool.
    pub args: Vec<String>,
    /// Value of the `PWND` field from `irecovery -q` after pwning, if present.
    pub pwnd: Option<String>,
    /// Resulting device mode, as detected by `parse_irecovery_q`.
    pub mode: DeviceMode,
}

/// Picks the right pwn tool for `(processor_gen, os, arch, product_type)` and runs it,
/// then verifies the result via `irecovery -q`.
///
/// Mirrors `device_enter_mode pwnDFU` in `restore.sh`. Requires the device to already
/// be in DFU mode — caller is responsible for that step (DfuHelper in the UI).
#[tauri::command]
pub async fn enter_pwndfu(
    app: AppHandle,
    request: EnterPwnDfuRequest,
) -> Result<EnterPwnDfuResult, AppError> {
    let product_type = request.product_type.trim();
    if product_type.is_empty() {
        return Err(AppError::Parse(
            "product_type is required to pick a pwn tool".to_string(),
        ));
    }

    let proc_gen = infer_processor_gen(product_type)
        .ok_or_else(|| AppError::Parse(format!("Unknown processor for {product_type}")))?;

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let plan = pick_pwn_tool(proc_gen, os, arch, product_type)?;

    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!(
            "Placing {product_type} (A{proc_gen}) into pwnDFU using {} {}",
            plan.label,
            plan.args.join(" ")
        ),
    );

    let binary = resolve_pwn_tool(&app, &plan.source).await?;
    crate::tools::runner::run_streaming(&app, binary.clone(), &plan.args)?;

    if plan.run_gaster_reset {
        if let Ok(gaster) = resolve_binary_path(&app, "gaster") {
            crate::tools::runner::emit_log(&app, "info", "gaster reset");
            // Reset is best-effort — non-zero exit shouldn't undo a successful pwn.
            let _ = crate::tools::runner::run_streaming(&app, gaster, &["reset".to_string()]);
        }
    }

    // Re-query irecovery to confirm the device is pwned. Capture stdout directly
    // (the streaming runner only emits to the log channel).
    let irecovery = resolve_binary_path(&app, "irecovery").map_err(AppError::CommandFailed)?;
    let output = Command::new(&irecovery).arg("-q").output()?;
    if !output.status.success() {
        return Err(AppError::CommandFailed(format!(
            "irecovery -q exited {} after pwn",
            output.status
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = parse_irecovery_q(&stdout);

    let pwn_succeeded = info.pwnd.as_deref().is_some_and(|s| !s.is_empty())
        || (proc_gen == 6
            && matches!(info.mode, DeviceMode::DFU)
            && info
                .srtg
                .as_deref()
                .is_some_and(|s| !s.starts_with("[iBoot")));

    if !pwn_succeeded {
        return Err(AppError::CommandFailed(format!(
            "Failed to enter pwnDFU using {}. Force-restart the device, re-enter DFU, and try again.",
            plan.label
        )));
    }

    if let Some(p) = info.pwnd.as_deref() {
        crate::tools::runner::emit_log(&app, "info", &format!("Pwned: {p}"));
    } else {
        crate::tools::runner::emit_log(&app, "info", "Found device in pwned iBSS mode.");
    }

    // For A6 devices, the pwn is indicated by SRTG (pwned iBSS) rather than PWND field.
    // If we detected a successful pwn via the pwn_succeeded check above but PWND is empty,
    // explicitly promote DFU → pwnDFU so the mode reflects the actual state.
    if matches!(info.mode, DeviceMode::DFU) && proc_gen == 6 && info.pwnd.is_none() {
        info.mode = DeviceMode::PwnDFU;
    }

    Ok(EnterPwnDfuResult {
        tool: plan.label.to_string(),
        args: plan.args,
        pwnd: info.pwnd,
        mode: info.mode,
    })
}

#[derive(Debug)]
enum ToolSource {
    /// Bundled with the app, resolved via `platform::resolve_binary_path`.
    Bundled(&'static str),
    /// Downloaded on demand from a published mirror, resolved via `external_tools::ensure_pwn_tool`.
    External(ExternalTool),
}

#[derive(Debug)]
struct PwnPlan {
    source: ToolSource,
    args: Vec<String>,
    run_gaster_reset: bool,
    /// Stable label returned to the frontend (e.g. "gaster", "litera1n").
    label: &'static str,
}

async fn resolve_pwn_tool(app: &AppHandle, source: &ToolSource) -> Result<PathBuf, AppError> {
    match source {
        ToolSource::Bundled(name) => resolve_binary_path(app, name).map_err(AppError::CommandFailed),
        ToolSource::External(tool) => ensure_pwn_tool(app, *tool).await,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPwnToolRequest {
    /// Logical tool name. Currently only "kuroutadori" is supported.
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadPwnToolResult {
    pub tool: String,
    pub binary_path: String,
}

/// Idempotently fetches an external pwn tool (kuroutadori → litera1n) into the
/// workspace's `tools/` directory and returns the resolved binary path. Mirrors
/// `kuroutadori_init` in `restore.sh:2405-2428`.
#[tauri::command]
pub async fn download_pwn_tool(
    app: AppHandle,
    request: DownloadPwnToolRequest,
) -> Result<DownloadPwnToolResult, AppError> {
    let tool_name = request.tool.trim();
    let tool = ExternalTool::from_name(tool_name)
        .ok_or_else(|| AppError::Parse(format!("Unknown external pwn tool: {tool_name}")))?;
    let path = ensure_pwn_tool(&app, tool).await?;
    Ok(DownloadPwnToolResult {
        tool: tool_name.to_string(),
        binary_path: path.to_string_lossy().to_string(),
    })
}

/// Pure tool-selection logic, factored out so it's testable without a Tauri app handle.
/// Mirrors the cascade in `restore.sh:2236-2288`.
fn pick_pwn_tool(
    proc_gen: u8,
    os: &str,
    arch: &str,
    product_type: &str,
) -> Result<PwnPlan, AppError> {
    match proc_gen {
        4 => {
            let mut args = vec![];
            // restore.sh: macOS-only --use-limera1n for the original A4 lineup.
            if os == "macos"
                && (product_type == "iPad1,1"
                    || product_type.starts_with("iPhone3,")
                    || product_type == "iPod4,1")
            {
                args.push("--use-limera1n".to_string());
            }
            Ok(PwnPlan {
                source: ToolSource::Bundled("primepwn"),
                args,
                run_gaster_reset: false,
                label: "primepwn",
            })
        }
        5 => Err(AppError::Parse(
            "A5/A5X devices need external hardware (Arduino+USB Host Shield or Pi Pico for \
             checkm8-a5). Use the SSH Ramdisk → kDFU path instead, or jailbreak with OpenSSH."
                .to_string(),
        )),
        6 => match os {
            "macos" => Ok(PwnPlan {
                source: ToolSource::Bundled("ipwnder"),
                args: vec!["-p".to_string()],
                run_gaster_reset: false,
                label: "ipwnder",
            }),
            "linux" => {
                // restore.sh:2247-2252 picks `a6meowing` for iPhone5,* on Linux, otherwise
                // `litera1n` (from the kuroutadori bundle). a6meowing isn't fetched by this
                // module yet — flag it explicitly until we ship it.
                if product_type.starts_with("iPhone5,") {
                    Err(AppError::Parse(
                        "A6 iPhone5,* on Linux needs `a6meowing`, which isn't fetchable yet. \
                         Use the SSH Ramdisk → kDFU path for now."
                            .to_string(),
                    ))
                } else {
                    Ok(PwnPlan {
                        source: ToolSource::External(ExternalTool::Kuroutadori),
                        args: vec!["-D".to_string()],
                        run_gaster_reset: false,
                        label: "litera1n",
                    })
                }
            }
            _ => Err(AppError::Parse(format!(
                "Unsupported platform for A6 pwn: {os}"
            ))),
        },
        7 => {
            // restore.sh prefers ipwnder on macOS arm64 for A7; gaster everywhere else.
            if os == "macos" && arch == "aarch64" {
                Ok(PwnPlan {
                    source: ToolSource::Bundled("ipwnder"),
                    args: vec!["-p".to_string()],
                    run_gaster_reset: true,
                    label: "ipwnder",
                })
            } else {
                Ok(PwnPlan {
                    source: ToolSource::Bundled("gaster"),
                    args: vec!["pwn".to_string()],
                    run_gaster_reset: true,
                    label: "gaster",
                })
            }
        }
        8..=10 => Ok(PwnPlan {
            source: ToolSource::Bundled("gaster"),
            args: vec!["pwn".to_string()],
            run_gaster_reset: true,
            label: "gaster",
        }),
        _ => Err(AppError::Parse(format!(
            "No auto-pwn path for processor generation A{proc_gen}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bundled(plan: &PwnPlan, expected: &str) {
        match &plan.source {
            ToolSource::Bundled(name) => assert_eq!(*name, expected),
            other => panic!("expected bundled {expected}, got {other:?}"),
        }
    }

    #[test]
    fn a4_picks_primepwn_with_limera1n_on_macos_for_legacy_devices() {
        let plan = pick_pwn_tool(4, "macos", "x86_64", "iPhone3,1").unwrap();
        assert_bundled(&plan, "primepwn");
        assert_eq!(plan.label, "primepwn");
        assert_eq!(plan.args, vec!["--use-limera1n"]);
        assert!(!plan.run_gaster_reset);
    }

    #[test]
    fn a4_no_limera1n_on_linux() {
        let plan = pick_pwn_tool(4, "linux", "x86_64", "iPhone3,1").unwrap();
        assert!(plan.args.is_empty());
    }

    #[test]
    fn a5_returns_external_hardware_error() {
        assert!(pick_pwn_tool(5, "macos", "aarch64", "iPhone4,1").is_err());
    }

    #[test]
    fn a6_macos_uses_ipwnder() {
        let plan = pick_pwn_tool(6, "macos", "aarch64", "iPhone5,1").unwrap();
        assert_bundled(&plan, "ipwnder");
        assert_eq!(plan.args, vec!["-p"]);
        assert!(!plan.run_gaster_reset);  // A6 should not run gaster reset (only A7+)
    }

    #[test]
    fn a6_linux_iphone5_errors_until_a6meowing_supported() {
        assert!(pick_pwn_tool(6, "linux", "x86_64", "iPhone5,1").is_err());
    }

    #[test]
    fn a6_linux_non_iphone5_uses_litera1n_via_kuroutadori() {
        let plan = pick_pwn_tool(6, "linux", "x86_64", "iPad3,4").unwrap();
        match &plan.source {
            ToolSource::External(ExternalTool::Kuroutadori) => {}
            other => panic!("expected External(Kuroutadori), got {other:?}"),
        }
        assert_eq!(plan.label, "litera1n");
        assert_eq!(plan.args, vec!["-D"]);
    }

    #[test]
    fn a7_macos_arm64_uses_ipwnder() {
        let plan = pick_pwn_tool(7, "macos", "aarch64", "iPhone6,1").unwrap();
        assert_bundled(&plan, "ipwnder");
    }

    #[test]
    fn a7_macos_intel_uses_gaster() {
        let plan = pick_pwn_tool(7, "macos", "x86_64", "iPhone6,1").unwrap();
        assert_bundled(&plan, "gaster");
        assert_eq!(plan.args, vec!["pwn"]);
    }

    #[test]
    fn a8_through_a10_use_gaster() {
        for gen in [8, 9, 10] {
            let plan = pick_pwn_tool(gen, "macos", "aarch64", "iPhone7,2").unwrap();
            assert_bundled(&plan, "gaster");
        }
    }
}
