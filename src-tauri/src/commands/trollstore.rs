use crate::error::AppError;
use crate::models::trollstore::{
    TrollStoreEligibilityRequest, TrollStoreEligibilityResult, TrollStorePath,
    TrollStorePrepareRequest, TrollStorePrepareResult,
};
use crate::platform::resolve_binary_path;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::AppHandle;

const RELEASES_API: &str = "https://api.github.com/repos/opa334/TrollStore/releases/latest";

#[tauri::command]
pub async fn prepare_trollstore_assets(
    app: AppHandle,
    request: TrollStorePrepareRequest,
) -> Result<TrollStorePrepareResult, AppError> {
    let saved_dir = request.saved_dir.trim();
    if saved_dir.is_empty() {
        return Err(AppError::Parse("Saved directory is required".into()));
    }
    fs::create_dir_all(saved_dir)?;

    let dest = Path::new(saved_dir);
    let tar_path = dest.join("TrollStore.tar");
    let helper_path = dest.join("PersistenceHelper_Embedded");
    let version_stamp = dest.join("TrollStore_version");

    let latest = match request.force_version {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => fetch_latest_version(&app)?,
    };
    crate::tools::runner::emit_log(&app, "info", &format!("Latest TrollStore version: {latest}"));

    let cached_version = fs::read_to_string(&version_stamp)
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let assets_present = tar_path.exists()
        && helper_path.exists()
        && tar_path.metadata().map(|m| m.len() > 0).unwrap_or(false)
        && helper_path.metadata().map(|m| m.len() > 0).unwrap_or(false);

    if cached_version == latest && assets_present {
        crate::tools::runner::emit_log(
            &app,
            "info",
            &format!("Using cached TrollStore {latest} assets"),
        );
        return Ok(TrollStorePrepareResult {
            version: latest,
            tar_path: tar_path.to_string_lossy().to_string(),
            helper_path: helper_path.to_string_lossy().to_string(),
            cached: true,
        });
    }

    let _ = fs::remove_file(&tar_path);
    let _ = fs::remove_file(&helper_path);

    let tar_url = format!(
        "https://github.com/opa334/TrollStore/releases/download/{latest}/TrollStore.tar"
    );
    let helper_url = format!(
        "https://github.com/opa334/TrollStore/releases/download/{latest}/PersistenceHelper_Embedded"
    );

    let aria2c = resolve_binary_path(&app, "aria2c").map_err(AppError::CommandFailed)?;
    download_with_aria2(&app, &aria2c, &tar_url, dest, "TrollStore.tar")?;
    download_with_aria2(&app, &aria2c, &helper_url, dest, "PersistenceHelper_Embedded")?;

    if !tar_path.exists() || !helper_path.exists() {
        return Err(AppError::CommandFailed(
            "TrollStore download finished but expected files are missing".into(),
        ));
    }

    fs::write(&version_stamp, format!("{latest}\n"))?;
    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!("Downloaded TrollStore {latest} assets to {}", dest.display()),
    );

    Ok(TrollStorePrepareResult {
        version: latest,
        tar_path: tar_path.to_string_lossy().to_string(),
        helper_path: helper_path.to_string_lossy().to_string(),
        cached: false,
    })
}

#[tauri::command]
pub async fn check_trollstore_eligibility(
    request: TrollStoreEligibilityRequest,
) -> Result<TrollStoreEligibilityResult, AppError> {
    let major = request
        .ios_version
        .as_deref()
        .and_then(parse_ios_major);
    let product = request.product_type.as_deref().map(str::to_lowercase);

    let Some(major) = major else {
        return Ok(TrollStoreEligibilityResult {
            path: TrollStorePath::Unknown,
            reason: "iOS version unknown — connect a paired device first.".into(),
            ios_major: None,
        });
    };

    if major < 14 {
        return Ok(TrollStoreEligibilityResult {
            path: TrollStorePath::Incompatible,
            reason: format!(
                "TrollStore requires iOS 14 or newer (detected iOS {major}.x)."
            ),
            ios_major: Some(major),
        });
    }

    if (14..=15).contains(&major) {
        let helper = product
            .as_deref()
            .map(|p| p.starts_with("iphone") || p.starts_with("ipad") || p.starts_with("ipod"))
            .unwrap_or(true);
        if !helper {
            return Ok(TrollStoreEligibilityResult {
                path: TrollStorePath::Unknown,
                reason: "Unrecognized device type for iOS 14/15 install path.".into(),
                ios_major: Some(major),
            });
        }
        return Ok(TrollStoreEligibilityResult {
            path: TrollStorePath::Ios14To15Ramdisk,
            reason:
                "Install via SSH ramdisk (iOS 14/15): boot SSH ramdisk, mount filesystems, then \
                 push TrollStore.tar + helper into Tips.app and run trollstorehelper."
                    .into(),
            ios_major: Some(major),
        });
    }

    Ok(TrollStoreEligibilityResult {
        path: TrollStorePath::Ios16PlusTrollRestore,
        reason: "iOS 16+ install path is via TrollRestore (Python). Make sure Tips.app is \
                 installed on the device, then run trollstore.py from the venv."
            .into(),
        ios_major: Some(major),
    })
}

fn fetch_latest_version(app: &AppHandle) -> Result<String, AppError> {
    let curl = which("curl").ok_or_else(|| {
        AppError::CommandFailed("curl is required to query GitHub releases".into())
    })?;
    crate::tools::runner::emit_log(app, "info", "Querying GitHub for latest TrollStore version");
    let output = Command::new(&curl)
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github+json",
            "-A",
            "legacykit",
            RELEASES_API,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(AppError::CommandFailed(if stderr.is_empty() {
            format!("curl exited with {}", output.status)
        } else {
            stderr
        }));
    }
    let body = String::from_utf8_lossy(&output.stdout);
    parse_tag_name(&body).ok_or_else(|| {
        AppError::Parse("Could not extract tag_name from GitHub API response".into())
    })
}

fn download_with_aria2(
    app: &AppHandle,
    aria2c: &Path,
    url: &str,
    dest_dir: &Path,
    file_name: &str,
) -> Result<(), AppError> {
    let args = vec![
        "--continue=true".to_string(),
        "--max-connection-per-server=8".to_string(),
        "--split=8".to_string(),
        "--summary-interval=1".to_string(),
        "--allow-overwrite=true".to_string(),
        "--dir".to_string(),
        dest_dir.to_string_lossy().to_string(),
        "--out".to_string(),
        file_name.to_string(),
        url.to_string(),
    ];
    crate::tools::runner::emit_log(app, "info", &format!("Downloading {file_name}"));
    crate::tools::runner::run_streaming(app, aria2c.to_path_buf(), &args)
}

fn parse_tag_name(body: &str) -> Option<String> {
    // Avoid pulling in a json crate for one field. The release JSON always has
    // a top-level "tag_name" string field.
    let needle = "\"tag_name\"";
    let start = body.find(needle)?;
    let after = &body[start + needle.len()..];
    let colon = after.find(':')?;
    let rest = &after[colon + 1..];
    let q1 = rest.find('"')?;
    let after_q1 = &rest[q1 + 1..];
    let q2 = after_q1.find('"')?;
    let value = &after_q1[..q2];
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_ios_major(version: &str) -> Option<u32> {
    let trimmed = version.trim();
    if trimmed.is_empty() {
        return None;
    }
    let head = trimmed.split('.').next()?;
    head.parse::<u32>().ok()
}

fn which(bin: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_name_from_release_payload() {
        let payload = r#"{"url":"...","tag_name":"2.1.5","name":"TrollStore 2.1.5"}"#;
        assert_eq!(parse_tag_name(payload).as_deref(), Some("2.1.5"));
    }

    #[test]
    fn parses_tag_name_with_whitespace() {
        let payload = r#"{ "tag_name" : "v2.0.10" }"#;
        assert_eq!(parse_tag_name(payload).as_deref(), Some("v2.0.10"));
    }

    #[test]
    fn ios_major_extraction() {
        assert_eq!(parse_ios_major("14.4.2"), Some(14));
        assert_eq!(parse_ios_major("16"), Some(16));
        assert_eq!(parse_ios_major("17.1"), Some(17));
        assert_eq!(parse_ios_major(""), None);
        assert_eq!(parse_ios_major("invalid"), None);
    }

    #[test]
    fn parse_tag_name_missing_returns_none() {
        assert_eq!(parse_tag_name(r#"{"name":"x"}"#), None);
    }
}
