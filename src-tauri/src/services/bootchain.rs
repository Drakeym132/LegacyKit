use crate::error::AppError;
use crate::platform::resolve_binary_path;
use crate::services::firmware_keys::{fetch_firmware_keys, get_component_keys};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Checks if a file has img3 container magic bytes.
/// img3 magic is "3gmI" (little-endian) = bytes 0x33 0x67 0x6d 0x49.
fn is_img3_container(path: &Path) -> bool {
    let mut buf = [0u8; 4];
    if let Ok(mut f) = File::open(path) {
        if f.read(&mut buf).is_ok() {
            // img3 magic is "3gmI" in little-endian
            return buf == [0x33, 0x67, 0x6d, 0x49];
        }
    }
    false
}

#[derive(Debug, Clone)]
pub struct PreparedBootchain {
    pub repacked_ibss_path: String,
    pub repacked_ibec_path: Option<String>,
}

pub async fn prepare_cached_bootchain(
    app: &AppHandle,
    ipsw_path: &str,
    cache_dir: &Path,
    boot_args: &str,
    use_img4: bool,
    product_type: &str,
    build_id: &str,
) -> Result<(PreparedBootchain, bool), AppError> {
    let cached_ibss = cache_dir.join("iBSS.repacked");
    let cached_ibec = cache_dir.join("iBEC.repacked");

    if cache_is_reusable(
        ipsw_path,
        &cached_ibss,
        include_ibec.then_some(&cached_ibec),
    )? {
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
        Some(find_component_path(ipsw_path, "iBEC").ok_or_else(|| {
            AppError::Parse("Could not locate iBEC component in IPSW".to_string())
        })?)
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

    // Fetch firmware keys if we're dealing with img3 containers (A4-A6 devices)
    let needs_keys = !use_img4
        && (is_img3_container(&ibss_extracted)
            || ibec_extracted
                .as_ref()
                .is_some_and(|p| is_img3_container(p)));

    if needs_keys {
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!(
                "Detected img3 containers, fetching firmware keys for {product_type} {build_id}"
            ),
        );
        fetch_firmware_keys(app, product_type, build_id).await?;
    }

    // Process iBSS
    let (ibss_patched, ibss_template) = if use_img4 {
        // A7+ devices use img4: just patch directly
        let patched = work_dir.join("iBSS.patched");
        patch_iboot32(app, &ibss_extracted, &patched, boot_args)?;
        (patched, None)
    } else if is_img3_container(&ibss_extracted) {
        // A4-A6 devices with img3: decrypt -> strip -> patch -> repack with template
        let keys = get_component_keys(product_type, build_id, "iBSS").ok_or_else(|| {
            AppError::CommandFailed(format!(
                "No firmware keys found for iBSS ({product_type} {build_id})"
            ))
        })?;

        let ibss_decrypted = work_dir.join("iBSS.decrypted");
        decrypt_img3(app, &ibss_extracted, &ibss_decrypted, &keys.iv, &keys.key)?;

        let ibss_stripped = work_dir.join("iBSS.stripped");
        strip_img3_wrapper(app, &ibss_decrypted, &ibss_stripped)?;

        let ibss_patched = work_dir.join("iBSS.patched");
        patch_iboot32(app, &ibss_stripped, &ibss_patched, boot_args)?;

        (ibss_patched, Some(ibss_decrypted))
    } else {
        // Already decrypted (rare case)
        let patched = work_dir.join("iBSS.patched");
        patch_iboot32(app, &ibss_extracted, &patched, boot_args)?;
        (patched, None)
    };

    // Process iBEC if needed
    let (ibec_patched, ibec_template) = if let Some(extracted) = ibec_extracted.as_ref() {
        if use_img4 {
            let patched = work_dir.join("iBEC.patched");
            patch_iboot32(app, extracted, &patched, boot_args)?;
            (Some(patched), None)
        } else if is_img3_container(extracted) {
            let keys = get_component_keys(product_type, build_id, "iBEC").ok_or_else(|| {
                AppError::CommandFailed(format!(
                    "No firmware keys found for iBEC ({product_type} {build_id})"
                ))
            })?;

            let ibec_decrypted = work_dir.join("iBEC.decrypted");
            decrypt_img3(app, extracted, &ibec_decrypted, &keys.iv, &keys.key)?;

            let ibec_stripped = work_dir.join("iBEC.stripped");
            strip_img3_wrapper(app, &ibec_decrypted, &ibec_stripped)?;

            let ibec_patched = work_dir.join("iBEC.patched");
            patch_iboot32(app, &ibec_stripped, &ibec_patched, boot_args)?;

            (Some(ibec_patched), Some(ibec_decrypted))
        } else {
            let patched = work_dir.join("iBEC.patched");
            patch_iboot32(app, extracted, &patched, boot_args)?;
            (Some(patched), None)
        }
    } else {
        (None, None)
    };

    // Repack components
    if use_img4 {
        pack_img4(app, &ibss_patched, &cached_ibss)?;
        if let Some(ibec) = ibec_patched.as_ref() {
            pack_img4(app, ibec, &cached_ibec)?;
        }
    } else {
        // For img3, repack with template for proper signing
        if let Some(template) = ibss_template.as_ref() {
            repack_img3_with_template(app, &ibss_patched, &cached_ibss, template)?;
        } else {
            repack_img3(app, &ibss_patched, &cached_ibss)?;
        }
        if let (Some(ibec), Some(template)) = (ibec_patched.as_ref(), ibec_template.as_ref()) {
            repack_img3_with_template(app, ibec, &cached_ibec, template)?;
        } else if let Some(ibec) = ibec_patched.as_ref() {
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

/// Sends patched iBSS/iBEC to a device in pwnDFU mode using irecovery.
/// This is the correct flow for A6 and later devices when booting from pwnDFU.
///
/// The flow is:
/// 1. For A6 devices: gaster reset (to clear the pwned state)
/// 2. irecovery -f <ibss> (send patched iBSS)
/// 3. irecovery -f <ibec> (send patched iBEC, if provided)
///
/// Note: kloader is NOT used here - it's an ARM binary that runs on the iOS device,
/// not a host-side tool. kloader is only used for kDFU mode (entering DFU from a
/// jailbroken device via SSH).
pub fn send_bootchain_pwndfu(
    app: &AppHandle,
    ibss_path: &str,
    ibec_path: Option<&str>,
    processor_gen: Option<u8>,
) -> Result<(), AppError> {
    let irecovery = resolve_binary_path(app, "irecovery").map_err(AppError::CommandFailed)?;

    // For A6 devices, we need to reset gaster first
    if processor_gen == Some(6) {
        let gaster = resolve_binary_path(app, "gaster").map_err(AppError::CommandFailed)?;
        crate::tools::runner::emit_log(app, "info", "Resetting gaster for A6 device...");
        crate::tools::runner::run_streaming(app, gaster, &["reset".to_string()])?;
        // Small delay to let the device settle
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // Send iBSS
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Sending patched iBSS via irecovery: {}", ibss_path),
    );
    crate::tools::runner::run_streaming(
        app,
        irecovery.clone(),
        &["-f".to_string(), ibss_path.to_string()],
    )?;

    // Small delay between iBSS and iBEC
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Send iBEC if provided
    if let Some(ibec) = ibec_path {
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!("Sending patched iBEC via irecovery: {}", ibec),
        );
        crate::tools::runner::run_streaming(
            app,
            irecovery.clone(),
            &["-f".to_string(), ibec.to_string()],
        )?;

        // After iBEC is loaded it sits at its own recovery prompt waiting for input.
        // Without an explicit boot command iBEC just stays there and the device
        // appears to be "stuck in recovery" from the host's perspective.
        //
        // Sequence (matches the bash legacy `just boot` flow):
        //   1. setenv auto-boot true   – ensure iBoot will continue past recovery on next reset
        //   2. saveenv                 – persist the env var
        //   3. fsboot                  – tell iBEC to boot the OS from NAND
        //
        // Wait a moment so iBEC's USB recovery interface has time to come up before
        // we start issuing -c commands; otherwise the first command can race the
        // re-enumeration and silently drop.
        std::thread::sleep(std::time::Duration::from_millis(1500));

        let post_ibec_cmds: &[&str] = &["setenv auto-boot true", "saveenv", "fsboot"];
        for cmd in post_ibec_cmds {
            crate::tools::runner::emit_log(
                app,
                "info",
                &format!("irecovery -c \"{}\"", cmd),
            );
            crate::tools::runner::run_streaming(
                app,
                irecovery.clone(),
                &["-c".to_string(), (*cmd).to_string()],
            )?;
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    crate::tools::runner::emit_log(app, "info", "Bootchain sent successfully");
    Ok(())
}

/// Resource path resolver for kloader variants.
/// Returns the path to the kloader binary in the app's resources directory.
///
/// IMPORTANT: kloader is an ARM binary that runs ON the iOS device, not on the host.
/// It is used for kDFU mode (entering DFU from a jailbroken device via SSH).
/// The caller is responsible for sending this binary to the device via SSH.
pub fn get_kloader_resource_path(app: &AppHandle, variant: &str) -> Result<std::path::PathBuf, AppError> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| AppError::CommandFailed(format!("Failed to get resource dir: {}", e)))?;
    
    let kloader_name = match variant {
        "kloader5" => "kloader5",
        "kloader_axi0mX" => "kloader_axi0mX",
        _ => "kloader",
    };
    
    let path = resource_path.join("kloader").join(kloader_name);
    if path.exists() {
        Ok(path)
    } else {
        Err(AppError::CommandFailed(format!(
            "kloader variant '{}' not found at {}",
            kloader_name,
            path.display()
        )))
    }
}

/// Decrypts an img3 file using xpwntool with IV and key.
fn decrypt_img3(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    iv: &str,
    key: &str,
) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "xpwntool").map_err(AppError::CommandFailed)?;
    let args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
        "-iv".to_string(),
        iv.to_string(),
        "-k".to_string(),
        key.to_string(),
        "-decrypt".to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Decrypting img3 -> {}", output.to_string_lossy()),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

/// Strips img3 wrapper to get raw iBoot binary.
fn strip_img3_wrapper(app: &AppHandle, input: &Path, output: &Path) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "xpwntool").map_err(AppError::CommandFailed)?;
    let args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Stripping img3 wrapper -> {}", output.to_string_lossy()),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

/// Patches iBoot32 binary with custom boot arguments.
fn patch_iboot32(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    boot_args: &str,
) -> Result<(), AppError> {
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

    // Run the patcher - some iBoot32Patcher builds return exit code 1 even on success,
    // so we check for output file existence rather than relying solely on exit code.
    let result = crate::tools::runner::run_streaming(app, binary, &args);

    // If the command failed but the output file exists with non-zero size, consider it success.
    // This handles quirky iBoot32Patcher builds that write valid output but return non-zero.
    if result.is_err() && output.exists() {
        if let Ok(metadata) = std::fs::metadata(output) {
            if metadata.len() > 0 {
                crate::tools::runner::emit_log(
                    app,
                    "info",
                    "iBoot32Patcher returned non-zero but output file exists - treating as success",
                );
                return Ok(());
            }
        }
    }

    result
}

/// Packs a binary into IMG4 format.
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

/// Repacks a binary into img3 format (without template).
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

/// Repacks a binary into img3 format using a template for proper structure.
fn repack_img3_with_template(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    template: &Path,
) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "xpwntool").map_err(AppError::CommandFailed)?;
    let args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
        "-t".to_string(),
        template.to_string_lossy().to_string(),
    ];
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!(
            "Repacking IMG3 with template -> {}",
            output.to_string_lossy()
        ),
    );
    crate::tools::runner::run_streaming(app, binary, &args)
}

fn extract_zip_entry(
    archive_path: &str,
    entry_name: &str,
    output_path: &Path,
) -> Result<(), AppError> {
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
