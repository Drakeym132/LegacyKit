//! Lazy on-demand fetch + cache for pwn tools that aren't bundled with the app.
//!
//! Mirrors the `kuroutadori_init` workflow from the legacy bash implementation:
//! download a `.tar.gz` from `sep.lol/files/legacypreviews`, verify SHA-1, extract
//! to a known cache directory, and hand back the binary path. Idempotent — repeat
//! calls are a no-op once the sha1check sentinel matches.

use crate::error::AppError;
use crate::services::sha1::sha1_file;
use crate::services::workspace::WorkspaceLayout;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;

/// Names this module knows how to fetch. Keep this list narrow — every entry is a
/// download URL we trust.
#[derive(Debug, Clone, Copy)]
pub enum ExternalTool {
    /// `kuroutadori` ships a `litera1n` binary used for A6/A7 pwn on Linux and some
    /// macOS restore flows.
    Kuroutadori,
}

impl ExternalTool {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "kuroutadori" => Some(ExternalTool::Kuroutadori),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ToolSpec {
    /// Stable directory name under `<workspace>/tools/`. Matches the naming convention
    /// from the legacy bash implementation: `kuroutadori="kuroutadori_${platform}"`.
    pub install_dir_name: String,
    pub url: &'static str,
    pub sha1: &'static str,
    /// Path inside the extracted archive to the primary executable.
    pub bin_subpath: &'static str,
}

/// Resolves the right URL + sha1 + install dir for the running platform.
/// Mirrors the `kuroutadori_init` workflow from the legacy bash implementation.
pub fn kuroutadori_spec(os: &str, arch: &str) -> Result<ToolSpec, AppError> {
    match (os, arch) {
        ("macos", _) => Ok(ToolSpec {
            install_dir_name: "kuroutadori_macos".to_string(),
            url: "https://sep.lol/files/legacypreviews/v1.0.1/32fcc405b9e55c66619865663e4c0fe5bf8374d98046f7e3e704c0c04f7f63791da2465c603d5bf3e9b0ffee94fc2ee3/kurouta_dori_v1.0.1_7efbf6a4_legacymacosx.tar.gz",
            sha1: "96d49341752b326443623277cdc4f3ea5974714a",
            bin_subpath: "bin/litera1n",
        }),
        ("linux", "x86_64") => Ok(ToolSpec {
            install_dir_name: "kuroutadori_linux-x86_64".to_string(),
            url: "https://sep.lol/files/legacypreviews/v1.0.1/c796056b432ba24759a14590927c0e4b9cf07b488ccd554bdfeb952421e3d71a005c18545343e4fd3c61c120dda1eff9/kurouta_dori_v1.0.1_7efbf6a4_linux-amd64.tar.gz",
            sha1: "508221fc08d6e570300e319de0a34b08dac97ba1",
            bin_subpath: "bin/litera1n",
        }),
        ("linux", "aarch64") => Ok(ToolSpec {
            install_dir_name: "kuroutadori_linux-arm64".to_string(),
            url: "https://sep.lol/files/legacypreviews/v1.0.1/4b81c4a850392f156374735eea4bbc2934dc13ae83848ef5d4599cd73e26352822fbda50ec23233b6fc081ae3b5d936c/kurouta_dori_v1.0.1_7efbf6a4_linux-arm64.tar.gz",
            sha1: "3cc645a242d65f7c0549d77278783cdd99206e81",
            bin_subpath: "bin/litera1n",
        }),
        _ => Err(AppError::Parse(format!(
            "kuroutadori has no published build for {os}/{arch}"
        ))),
    }
}

pub fn spec_for(tool: ExternalTool, os: &str, arch: &str) -> Result<ToolSpec, AppError> {
    match tool {
        ExternalTool::Kuroutadori => kuroutadori_spec(os, arch),
    }
}

/// Downloads and extracts the tool if missing or stale, then returns the path to
/// the primary binary inside the install dir.
///
/// Idempotent: if `<install_dir>/sha1check` matches the spec sha1, no work is done.
pub async fn ensure_tool(
    layout: &WorkspaceLayout,
    spec: &ToolSpec,
    log: impl Fn(&str),
) -> Result<PathBuf, AppError> {
    let tools_root = layout.tools_dir();
    fs::create_dir_all(&tools_root)?;
    let install_dir = tools_root.join(&spec.install_dir_name);
    let sha_marker = install_dir.join("sha1check");
    let bin_path = install_dir.join(spec.bin_subpath);

    if bin_path.exists() && read_marker(&sha_marker) == Some(spec.sha1.to_string()) {
        log(&format!(
            "Reusing cached {} at {}",
            spec.install_dir_name,
            install_dir.display()
        ));
        return Ok(bin_path);
    }

    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;

    let archive_path = layout
        .tmp_dir()
        .join(format!("{}.tar.gz", spec.install_dir_name));
    fs::create_dir_all(layout.tmp_dir())?;
    log(&format!(
        "Downloading {} from {}",
        spec.install_dir_name, spec.url
    ));
    download_to_file(spec.url, &archive_path).await?;

    let actual_sha = sha1_file(&archive_path.to_string_lossy())
        .map_err(|err| AppError::CommandFailed(format!("Failed to hash archive: {err}")))?;
    if !actual_sha.eq_ignore_ascii_case(spec.sha1) {
        let _ = fs::remove_file(&archive_path);
        return Err(AppError::CommandFailed(format!(
            "SHA-1 mismatch for {}: expected {}, got {actual_sha}",
            spec.install_dir_name, spec.sha1
        )));
    }

    log(&format!(
        "Extracting {} to {}",
        spec.install_dir_name,
        install_dir.display()
    ));
    extract_tar_gz(&archive_path, &install_dir)?;
    let _ = fs::remove_file(&archive_path);

    if !bin_path.exists() {
        return Err(AppError::CommandFailed(format!(
            "Archive extracted but expected binary {} not found",
            bin_path.display()
        )));
    }

    fs::write(&sha_marker, spec.sha1)?;
    Ok(bin_path)
}

async fn download_to_file(url: &str, dest: &Path) -> Result<(), AppError> {
    let bytes = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|err| AppError::CommandFailed(format!("HTTP GET failed: {err}")))?
        .error_for_status()
        .map_err(|err| AppError::CommandFailed(format!("HTTP error: {err}")))?
        .bytes()
        .await
        .map_err(|err| AppError::CommandFailed(format!("Failed to read response body: {err}")))?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest, bytes)?;
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<(), AppError> {
    // POSIX `tar` reliably handles .tar.gz on both macOS and Linux (auto-detects
    // gzip via `-z` or transparently via the `-x` reader). Avoid pulling in
    // `tar`/`flate2` Cargo deps for the same job.
    let status = Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .map_err(|err| AppError::CommandFailed(format!("Failed to spawn tar: {err}")))?;
    if !status.success() {
        return Err(AppError::CommandFailed(format!(
            "tar exited with status {status} while extracting {}",
            archive.display()
        )));
    }
    Ok(())
}

fn read_marker(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

/// Convenience wrapper used by Tauri commands and `enter_pwndfu`.
pub async fn ensure_pwn_tool(app: &AppHandle, tool: ExternalTool) -> Result<PathBuf, AppError> {
    let layout = crate::services::workspace::get_layout(app)?;
    let spec = spec_for(tool, std::env::consts::OS, std::env::consts::ARCH)?;
    ensure_tool(&layout, &spec, |msg| {
        crate::tools::runner::emit_log(app, "info", msg);
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_spec_matches_restore_sh() {
        let spec = kuroutadori_spec("macos", "aarch64").unwrap();
        assert_eq!(spec.install_dir_name, "kuroutadori_macos");
        assert_eq!(spec.sha1, "96d49341752b326443623277cdc4f3ea5974714a");
        assert!(spec.url.ends_with("legacymacosx.tar.gz"));
    }

    #[test]
    fn linux_x86_64_spec() {
        let spec = kuroutadori_spec("linux", "x86_64").unwrap();
        assert_eq!(spec.install_dir_name, "kuroutadori_linux-x86_64");
        assert_eq!(spec.sha1, "508221fc08d6e570300e319de0a34b08dac97ba1");
    }

    #[test]
    fn linux_arm64_spec() {
        let spec = kuroutadori_spec("linux", "aarch64").unwrap();
        assert_eq!(spec.install_dir_name, "kuroutadori_linux-arm64");
    }

    #[test]
    fn unknown_platform_errors() {
        assert!(kuroutadori_spec("windows", "x86_64").is_err());
    }

    #[test]
    fn from_name_known_and_unknown() {
        assert!(matches!(
            ExternalTool::from_name("kuroutadori"),
            Some(ExternalTool::Kuroutadori)
        ));
        assert!(ExternalTool::from_name("nope").is_none());
    }
}
