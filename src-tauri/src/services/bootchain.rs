use crate::error::AppError;
use crate::platform::resolve_binary_path;
use std::fs::{self, File};
use std::path::Path;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct PreparedBootchain {
    pub repacked_ibss_path: String,
    pub repacked_ibec_path: Option<String>,
}

pub fn prepare_cached_bootchain(
    app: &AppHandle,
    ipsw_path: &str,
    cache_dir: &Path,
    boot_args: &str,
    use_img4: bool,
    include_ibec: bool,
) -> Result<(PreparedBootchain, bool), AppError> {
    let cached_ibss = cache_dir.join("iBSS.repacked");
    let cached_ibec = cache_dir.join("iBEC.repacked");

    if cache_is_reusable(ipsw_path, &cached_ibss, include_ibec.then_some(&cached_ibec))? {
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!(
                "Using cached Just Boot bootchain from {}",
                cache_dir.to_string_lossy()
            ),
        );
        return Ok((
            PreparedBootchain {
                repacked_ibss_path: cached_ibss.to_string_lossy().to_string(),
                repacked_ibec_path: include_ibec.then(|| cached_ibec.to_string_lossy().to_string()),
            },
            true,
        ));
    }

    fs::create_dir_all(cache_dir)?;
    let work_dir = cache_dir.join(".work");
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)?;
    }
    fs::create_dir_all(&work_dir)?;

    let ibss_component = find_component_path(ipsw_path, "iBSS")
        .ok_or_else(|| AppError::Parse("Could not locate iBSS component in IPSW".to_string()))?;
    let ibec_component = if include_ibec {
        Some(
            find_component_path(ipsw_path, "iBEC").ok_or_else(|| {
                AppError::Parse("Could not locate iBEC component in IPSW".to_string())
            })?,
        )
    } else {
        None
    };

    crate::tools::runner::emit_log(app, "info", &format!("Extracting {ibss_component}"));
    let ibss_extracted = work_dir.join("iBSS.extracted");
    extract_zip_entry(ipsw_path, &ibss_component, &ibss_extracted)?;

    let ibec_extracted = if let Some(component) = ibec_component.as_deref() {
        crate::tools::runner::emit_log(app, "info", &format!("Extracting {component}"));
        let path = work_dir.join("iBEC.extracted");
        extract_zip_entry(ipsw_path, component, &path)?;
        Some(path)
    } else {
        None
    };

    let ibss_patched = work_dir.join("iBSS.patched");
    patch_iboot32(app, &ibss_extracted, &ibss_patched, boot_args)?;
    let ibec_patched = if let Some(extracted) = ibec_extracted.as_ref() {
        let path = work_dir.join("iBEC.patched");
        patch_iboot32(app, extracted, &path, boot_args)?;
        Some(path)
    } else {
        None
    };

    if use_img4 {
        pack_img4(app, &ibss_patched, &cached_ibss)?;
        if let Some(ibec) = ibec_patched.as_ref() {
            pack_img4(app, ibec, &cached_ibec)?;
        }
    } else {
        repack_img3(app, &ibss_patched, &cached_ibss)?;
        if let Some(ibec) = ibec_patched.as_ref() {
            repack_img3(app, ibec, &cached_ibec)?;
        }
    }

    let _ = fs::remove_dir_all(&work_dir);

    Ok((
        PreparedBootchain {
            repacked_ibss_path: cached_ibss.to_string_lossy().to_string(),
            repacked_ibec_path: include_ibec.then(|| cached_ibec.to_string_lossy().to_string()),
        },
        false,
    ))
}

pub fn run_kloader_with_paths(
    app: &AppHandle,
    ibss_path: &str,
    ibec_path: Option<&str>,
) -> Result<(), AppError> {
    let mut args = vec![ibss_path.to_string()];
    if let Some(ibec) = ibec_path {
        args.push(ibec.to_string());
    }
    let binary = resolve_binary_path(app, "kloader").map_err(AppError::CommandFailed)?;
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Booting patched components with kloader: {}", args.join(" ")),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

fn patch_iboot32(app: &AppHandle, input: &Path, output: &Path, boot_args: &str) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "iBoot32Patcher").map_err(AppError::CommandFailed)?;
    let args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
        "--rsa".to_string(),
        "-b".to_string(),
        boot_args.to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Patching iBoot -> {}", output.to_string_lossy()),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

fn pack_img4(app: &AppHandle, input: &Path, output: &Path) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "img4tool").map_err(AppError::CommandFailed)?;
    let args = vec![
        "-c".to_string(),
        output.to_string_lossy().to_string(),
        "-p".to_string(),
        input.to_string_lossy().to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Packing IMG4 -> {}", output.to_string_lossy()),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

fn repack_img3(app: &AppHandle, input: &Path, output: &Path) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "xpwntool").map_err(AppError::CommandFailed)?;
    let args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Repacking IMG3 -> {}", output.to_string_lossy()),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

fn extract_zip_entry(archive_path: &str, entry_name: &str, output_path: &Path) -> Result<(), AppError> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::CommandFailed(format!("Failed to open IPSW: {e}")))?;
    let mut entry = archive
        .by_name(entry_name)
        .map_err(|e| AppError::CommandFailed(format!("Entry '{entry_name}' not found: {e}")))?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = File::create(output_path)?;
    std::io::copy(&mut entry, &mut output)?;
    Ok(())
}

fn find_component_path(ipsw_path: &str, image: &str) -> Option<String> {
    let file = File::open(ipsw_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        let name = entry.name();
        if !name.starts_with("Firmware/dfu/") {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if lower.contains(&image.to_ascii_lowercase()) && lower.ends_with(".dfu") {
            return Some(name.to_string());
        }
    }
    None
}

fn cache_is_reusable(
    ipsw_path: &str,
    ibss_path: &Path,
    ibec_path: Option<&Path>,
) -> Result<bool, AppError> {
    if !ibss_path.exists() {
        return Ok(false);
    }
    if let Some(ibec) = ibec_path {
        if !ibec.exists() {
            return Ok(false);
        }
    }

    let ipsw_mtime = fs::metadata(ipsw_path)?.modified()?;
    let ibss_mtime = fs::metadata(ibss_path)?.modified()?;
    if ibss_mtime <= ipsw_mtime {
        return Ok(false);
    }
    if let Some(ibec) = ibec_path {
        let ibec_mtime = fs::metadata(ibec)?.modified()?;
        if ibec_mtime <= ipsw_mtime {
            return Ok(false);
        }
    }
    Ok(true)
}
