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

/// Maps product_type (e.g., "iPhone5,1") to hardware model code (e.g., "n41").
/// This is used to resolve IPSW component paths for DeviceTree and Kernelcache.
/// Source: restore.sh:1448-1525
pub fn product_type_to_hw_model(product_type: &str) -> Option<&'static str> {
    // The hardware model is the internal codename without the "ap" suffix
    match product_type {
        // iPad
        "iPad1,1" => Some("k48"),
        "iPad2,1" => Some("k93"),
        "iPad2,2" => Some("k94"),
        "iPad2,3" => Some("k95"),
        "iPad2,4" => Some("k93a"),
        "iPad2,5" => Some("p105"),
        "iPad2,6" => Some("p106"),
        "iPad2,7" => Some("p107"),
        "iPad3,1" => Some("j1"),
        "iPad3,2" => Some("j2"),
        "iPad3,3" => Some("j2a"),
        "iPad3,4" => Some("p101"),
        "iPad3,5" => Some("p102"),
        "iPad3,6" => Some("p103"),
        "iPad4,1" => Some("j71"),
        "iPad4,2" => Some("j72"),
        "iPad4,3" => Some("j73"),
        "iPad4,4" => Some("j85"),
        "iPad4,5" => Some("j86"),
        "iPad4,6" => Some("j87"),
        "iPad4,7" => Some("j85m"),
        "iPad4,8" => Some("j86m"),
        "iPad4,9" => Some("j87m"),
        "iPad5,1" => Some("j96"),
        "iPad5,2" => Some("j97"),
        "iPad5,3" => Some("j81"),
        "iPad5,4" => Some("j82"),
        "iPad6,3" => Some("j127"),
        "iPad6,4" => Some("j128"),
        "iPad6,7" => Some("j98a"),
        "iPad6,8" => Some("j99a"),
        "iPad7,1" => Some("j120"),
        "iPad7,2" => Some("j121"),
        "iPad7,3" => Some("j207"),
        "iPad7,4" => Some("j208"),
        "iPad7,5" => Some("j71b"),
        "iPad7,6" => Some("j72b"),
        "iPad7,11" => Some("j171"),
        "iPad7,12" => Some("j172"),
        // iPhone
        "iPhone1,1" => Some("m68"),
        "iPhone1,2" => Some("n82"),
        "iPhone2,1" => Some("n88"),
        "iPhone3,1" => Some("n90"),
        "iPhone3,2" => Some("n90b"),
        "iPhone3,3" => Some("n92"),
        "iPhone4,1" => Some("n94"),
        "iPhone5,1" => Some("n41"),
        "iPhone5,2" => Some("n42"),
        "iPhone5,3" => Some("n48"),
        "iPhone5,4" => Some("n49"),
        "iPhone6,1" => Some("n51"),
        "iPhone6,2" => Some("n53"),
        "iPhone7,1" => Some("n56"),
        "iPhone7,2" => Some("n61"),
        "iPhone9,1" => Some("d10"),
        "iPhone9,2" => Some("d11"),
        "iPhone9,3" => Some("d101"),
        "iPhone9,4" => Some("d111"),
        "iPhone10,1" => Some("d20"),
        "iPhone10,2" => Some("d21"),
        "iPhone10,3" => Some("d22"),
        "iPhone10,4" => Some("d201"),
        "iPhone10,5" => Some("d211"),
        "iPhone10,6" => Some("d221"),
        // iPod
        "iPod1,1" => Some("n45"),
        "iPod2,1" => Some("n72"),
        "iPod3,1" => Some("n18"),
        "iPod4,1" => Some("n81"),
        "iPod5,1" => Some("n78"),
        "iPod7,1" => Some("n102"),
        "iPod9,1" => Some("n112"),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct PreparedBootchain {
    pub repacked_ibss_path: String,
    pub repacked_ibec_path: Option<String>,
    /// Path to the decrypted DeviceTree file (for just boot)
    pub decrypted_devicetree_path: Option<String>,
    /// Path to the decrypted Kernelcache file (for just boot)
    pub decrypted_kernelcache_path: Option<String>,
}

/// Determines if iBEC should be included based on build ID and product type.
/// iOS 7.x/8.x non-iPad devices skip iBEC patching (restore.sh:7000-7001).
fn should_include_ibec(build_id: &str, product_type: &str) -> bool {
    // iOS 7.x/8.x non-iPad devices skip iBEC
    if (build_id.starts_with('7') || build_id.starts_with('8')) && !product_type.starts_with("iPad") {
        return false;
    }
    true
}

/// Determines the kernelcache filename suffix based on build ID and hardware model.
/// iOS 7-11 builds use `{hw_model}ap` suffix, older builds use just `{hw_model}`.
fn kernelcache_suffix(build_id: &str, hw_model: &str) -> String {
    // iOS 7-11 builds use the "ap" suffix
    if build_id.starts_with('7')
        || build_id.starts_with('8')
        || build_id.starts_with('9')
        || build_id.starts_with("10")
        || build_id.starts_with("11")
    {
        format!("{}ap", hw_model)
    } else {
        hw_model.to_string()
    }
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
    // Auto-determine iBEC inclusion based on build ID and product type
    let include_ibec = should_include_ibec(build_id, product_type);

    let cached_ibss = cache_dir.join("iBSS.repacked");
    let cached_ibec = cache_dir.join("iBEC.repacked");
    let cached_devicetree = cache_dir.join("DeviceTree.dec");
    let cached_kernelcache = cache_dir.join("Kernelcache.dec");

    if cache_is_reusable(
        ipsw_path,
        &cached_ibss,
        include_ibec.then_some(&cached_ibec),
        &cached_devicetree,
        &cached_kernelcache,
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
                decrypted_devicetree_path: Some(cached_devicetree.to_string_lossy().to_string()),
                decrypted_kernelcache_path: Some(cached_kernelcache.to_string_lossy().to_string()),
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

    // Get hardware model for DeviceTree/Kernelcache path resolution
    let hw_model = product_type_to_hw_model(product_type);

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

    // Process iBSS (with --debug flag for just boot)
    let (ibss_patched, ibss_template) = if use_img4 {
        // A7+ devices use img4: just patch directly
        let patched = work_dir.join("iBSS.patched");
        patch_iboot32(app, &ibss_extracted, &patched, boot_args, true)?;
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
        // iBSS gets --debug flag for just boot (restore.sh:6997)
        patch_iboot32(app, &ibss_stripped, &ibss_patched, boot_args, true)?;

        (ibss_patched, Some(ibss_decrypted))
    } else {
        // Already decrypted (rare case)
        let patched = work_dir.join("iBSS.patched");
        patch_iboot32(app, &ibss_extracted, &patched, boot_args, true)?;
        (patched, None)
    };

    // Process iBEC if needed (NO --debug flag for just boot iBEC)
    let (ibec_patched, ibec_template) = if let Some(extracted) = ibec_extracted.as_ref() {
        if use_img4 {
            let patched = work_dir.join("iBEC.patched");
            // iBEC does NOT get --debug flag for just boot (restore.sh:7004-7010)
            patch_iboot32(app, extracted, &patched, boot_args, false)?;
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
            // iBEC does NOT get --debug flag for just boot (restore.sh:7004-7010)
            patch_iboot32(app, &ibec_stripped, &ibec_patched, boot_args, false)?;

            (Some(ibec_patched), Some(ibec_decrypted))
        } else {
            let patched = work_dir.join("iBEC.patched");
            patch_iboot32(app, extracted, &patched, boot_args, false)?;
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

    // Extract and decrypt DeviceTree
    let devicetree_path = if let Some(hw) = hw_model {
        extract_and_decrypt_devicetree(
            app,
            ipsw_path,
            &work_dir,
            &cached_devicetree,
            hw,
            build_id,
            product_type,
            use_img4,
        )
        .await?
    } else {
        crate::tools::runner::emit_log(
            app,
            "warn",
            &format!(
                "No hardware model mapping for {product_type}, skipping DeviceTree extraction"
            ),
        );
        None
    };

    // Extract and decrypt Kernelcache
    let kernelcache_path = if let Some(hw) = hw_model {
        extract_and_decrypt_kernelcache(
            app,
            ipsw_path,
            &work_dir,
            &cached_kernelcache,
            hw,
            build_id,
            product_type,
            use_img4,
        )
        .await?
    } else {
        crate::tools::runner::emit_log(
            app,
            "warn",
            &format!(
                "No hardware model mapping for {product_type}, skipping Kernelcache extraction"
            ),
        );
        None
    };

    let _ = fs::remove_dir_all(&work_dir);

    Ok((
        PreparedBootchain {
            repacked_ibss_path: cached_ibss.to_string_lossy().to_string(),
            repacked_ibec_path: include_ibec.then(|| cached_ibec.to_string_lossy().to_string()),
            decrypted_devicetree_path: devicetree_path,
            decrypted_kernelcache_path: kernelcache_path,
        },
        false,
    ))
}

/// Extracts and decrypts DeviceTree from IPSW.
async fn extract_and_decrypt_devicetree(
    app: &AppHandle,
    ipsw_path: &str,
    work_dir: &Path,
    output_path: &Path,
    hw_model: &str,
    build_id: &str,
    product_type: &str,
    use_img4: bool,
) -> Result<Option<String>, AppError> {
    crate::tools::runner::emit_log(app, "info", "Extracting DeviceTree...");

    // Find DeviceTree in IPSW
    let dt_component = find_devicetree_path(ipsw_path, hw_model, build_id);

    let Some(dt_entry) = dt_component else {
        crate::tools::runner::emit_log(
            app,
            "warn",
            "Could not locate DeviceTree in IPSW, skipping",
        );
        return Ok(None);
    };

    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Found DeviceTree at: {}", dt_entry),
    );

    let dt_extracted = work_dir.join("DeviceTree.extracted");
    extract_zip_entry(ipsw_path, &dt_entry, &dt_extracted)?;

    // Check if we need to decrypt
    if use_img4 {
        // A7+ uses img4 format - for now just copy the extracted file
        // TODO: img4 decryption for A7+ devices
        fs::copy(&dt_extracted, output_path)?;
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!("DeviceTree copied to {}", output_path.to_string_lossy()),
        );
    } else if is_img3_container(&dt_extracted) {
        // Try to get DeviceTree keys
        let keys = get_component_keys(product_type, build_id, "DeviceTree");

        if let Some(keys) = keys {
            decrypt_component_only(
                app,
                &dt_extracted,
                output_path,
                Some(&keys.iv),
                Some(&keys.key),
            )?;
        } else {
            // No keys available - treat as plaintext
            crate::tools::runner::emit_log(
                app,
                "info",
                "DeviceTree has no firmware keys — treating as plaintext",
            );
            fs::copy(&dt_extracted, output_path)?;
        }
    } else {
        // Not img3, just copy
        fs::copy(&dt_extracted, output_path)?;
        crate::tools::runner::emit_log(
            app,
            "info",
            "DeviceTree is not img3 format, copied as-is",
        );
    }

    Ok(Some(output_path.to_string_lossy().to_string()))
}

/// Extracts and decrypts Kernelcache from IPSW.
async fn extract_and_decrypt_kernelcache(
    app: &AppHandle,
    ipsw_path: &str,
    work_dir: &Path,
    output_path: &Path,
    hw_model: &str,
    build_id: &str,
    product_type: &str,
    use_img4: bool,
) -> Result<Option<String>, AppError> {
    crate::tools::runner::emit_log(app, "info", "Extracting Kernelcache...");

    // Find Kernelcache in IPSW
    let kc_component = find_kernelcache_path(ipsw_path, hw_model, build_id);

    let Some(kc_entry) = kc_component else {
        crate::tools::runner::emit_log(
            app,
            "warn",
            "Could not locate Kernelcache in IPSW, skipping",
        );
        return Ok(None);
    };

    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Found Kernelcache at: {}", kc_entry),
    );

    let kc_extracted = work_dir.join("Kernelcache.extracted");
    extract_zip_entry(ipsw_path, &kc_entry, &kc_extracted)?;

    // Check if we need to decrypt
    if use_img4 {
        // A7+ uses img4 format - for now just copy the extracted file
        // TODO: img4 decryption for A7+ devices
        fs::copy(&kc_extracted, output_path)?;
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!("Kernelcache copied to {}", output_path.to_string_lossy()),
        );
    } else if is_img3_container(&kc_extracted) {
        // Try to get Kernelcache keys
        let keys = get_component_keys(product_type, build_id, "Kernelcache");

        if let Some(keys) = keys {
            decrypt_component_only(
                app,
                &kc_extracted,
                output_path,
                Some(&keys.iv),
                Some(&keys.key),
            )?;
        } else {
            // No keys available - treat as plaintext
            crate::tools::runner::emit_log(
                app,
                "info",
                "Kernelcache has no firmware keys — treating as plaintext",
            );
            fs::copy(&kc_extracted, output_path)?;
        }
    } else {
        // Not img3, just copy
        fs::copy(&kc_extracted, output_path)?;
        crate::tools::runner::emit_log(
            app,
            "info",
            "Kernelcache is not img3 format, copied as-is",
        );
    }

    Ok(Some(output_path.to_string_lossy().to_string()))
}

/// Finds the DeviceTree path in an IPSW.
/// For builds < 14E: searches Firmware/all_flash/all_flash.{hw_model}ap.production/DeviceTree.{hw_model}ap.img3
/// For builds 14E+: searches Firmware/all_flash/DeviceTree.{hw_model}ap.img3
/// Fallback: scans Firmware/all_flash/ for any DeviceTree*.img3 entry
fn find_devicetree_path(ipsw_path: &str, hw_model: &str, build_id: &str) -> Option<String> {
    let file = File::open(ipsw_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    // Determine if this is a 14E+ build (iOS 14+)
    let is_ios14_plus = build_id.starts_with("14E")
        || build_id.starts_with("14F")
        || build_id.starts_with("14G")
        || build_id.starts_with("14H")
        || build_id.starts_with("14I")
        || build_id.starts_with("14J")
        || build_id.starts_with("14K")
        || build_id.starts_with("14L")
        || build_id.starts_with("14M")
        || build_id.starts_with("14N")
        || build_id.starts_with("14O")
        || build_id.starts_with("14P")
        || build_id.starts_with("14Q")
        || build_id.starts_with("14R")
        || build_id.starts_with("14S")
        || build_id.starts_with("14T")
        || build_id.starts_with("14U")
        || build_id.starts_with("14V")
        || build_id.starts_with("14W")
        || build_id.starts_with("14X")
        || build_id.starts_with("14Y")
        || build_id.starts_with("14Z");

    // Try specific paths first
    let specific_paths = if is_ios14_plus {
        vec![
            format!("Firmware/all_flash/DeviceTree.{}ap.img3", hw_model),
            format!("Firmware/all_flash/devicetree.{}ap.img3", hw_model),
        ]
    } else {
        vec![
            format!(
                "Firmware/all_flash/all_flash.{}ap.production/DeviceTree.{}ap.img3",
                hw_model, hw_model
            ),
            format!(
                "Firmware/all_flash/all_flash.{}ap.production/devicetree.{}ap.img3",
                hw_model, hw_model
            ),
        ]
    };

    for path in &specific_paths {
        if archive.by_name(path).is_ok() {
            return Some(path.clone());
        }
    }

    // Fallback: scan for any DeviceTree*.img3 in all_flash
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        let name = entry.name();
        if name.contains("all_flash") && name.to_lowercase().contains("devicetree") {
            if name.ends_with(".img3") || name.ends_with(".img4") || name.ends_with(".im4p") {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Finds the Kernelcache path in an IPSW.
/// iOS 7-11 builds: kernelcache.release.{hw_model}ap
/// Older builds: kernelcache.release.{hw_model}
/// Fallback: scan root entries for kernelcache.release.* matching hw_model prefix
fn find_kernelcache_path(ipsw_path: &str, hw_model: &str, build_id: &str) -> Option<String> {
    let file = File::open(ipsw_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let suffix = kernelcache_suffix(build_id, hw_model);

    // Try specific paths
    let specific_paths = vec![
        format!("kernelcache.release.{}", suffix),
        format!("Kernelcache.release.{}", suffix),
    ];

    for path in &specific_paths {
        if archive.by_name(path).is_ok() {
            return Some(path.clone());
        }
    }

    // Fallback: scan for kernelcache.release.* entries
    for index in 0..archive.len() {
        let entry = archive.by_index(index).ok()?;
        let name = entry.name();
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("kernelcache.release.") {
            // Check if it matches our hw_model (with or without "ap" suffix)
            if lower.contains(&hw_model.to_lowercase()) {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Decrypts a component without patching or repacking.
/// If IV+key are provided: xpwntool <input> <output> -iv <iv> -k <key> -decrypt
/// If no keys (plaintext component): cp <input> <output>
fn decrypt_component_only(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    iv_opt: Option<&str>,
    key_opt: Option<&str>,
) -> Result<(), AppError> {
    if let (Some(iv), Some(key)) = (iv_opt, key_opt) {
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
            &format!("Decrypting -> {}", output.to_string_lossy()),
        );
        crate::tools::runner::run_streaming(app, binary, &args)
    } else {
        crate::tools::runner::emit_log(
            app,
            "info",
            &format!(
                "No keys provided, copying as plaintext -> {}",
                output.to_string_lossy()
            ),
        );
        fs::copy(input, output)?;
        Ok(())
    }
}

/// Sends patched iBSS/iBEC to a device in pwnDFU mode using irecovery.
/// This is the correct flow for 32-bit devices when booting from pwnDFU.
///
/// The flow is (restore.sh:7037-7074):
/// 1. For A6 devices: gaster reset (to clear the pwned state)
/// 2. irecovery -f <ibss> (send patched iBSS)
/// 3. Sleep 500ms
/// 4. irecovery -f <ibec> (send patched iBEC, if provided)
/// 5. Sleep 1500ms (wait for USB re-enumeration into recovery PID 0x1281)
/// 6. irecovery -f <devicetree> (send DeviceTree)
/// 7. irecovery -c "devicetree" (load DeviceTree)
/// 8. irecovery -f <kernelcache> (send Kernelcache)
/// 9. irecovery -c "bootx" (boot the kernel)
pub fn send_bootchain_pwndfu(
    app: &AppHandle,
    ibss_path: &str,
    ibec_path: Option<&str>,
    devicetree_path: Option<&str>,
    kernelcache_path: Option<&str>,
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
    }

    // Wait for USB re-enumeration into Recovery mode (PID 0x1281)
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Send DeviceTree and Kernelcache for just boot
    // This is the correct sequence that was missing - the old code used
    // setenv/saveenv/fsboot which doesn't work for tethered boot
    let dt_path = devicetree_path.ok_or_else(|| {
        AppError::CommandFailed(
            "DeviceTree path is required for just boot. Please re-prepare the bootchain.".to_string(),
        )
    })?;
    let kc_path = kernelcache_path.ok_or_else(|| {
        AppError::CommandFailed(
            "Kernelcache path is required for just boot. Please re-prepare the bootchain.".to_string(),
        )
    })?;

    // Send DeviceTree
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Sending DeviceTree via irecovery: {}", dt_path),
    );
    crate::tools::runner::run_streaming(
        app,
        irecovery.clone(),
        &["-f".to_string(), dt_path.to_string()],
    )?;

    // Issue devicetree command
    crate::tools::runner::emit_log(app, "info", "Loading DeviceTree...");
    crate::tools::runner::run_streaming(
        app,
        irecovery.clone(),
        &["-c".to_string(), "devicetree".to_string()],
    )?;

    // Send Kernelcache
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Sending Kernelcache via irecovery: {}", kc_path),
    );
    crate::tools::runner::run_streaming(
        app,
        irecovery.clone(),
        &["-f".to_string(), kc_path.to_string()],
    )?;

    // Issue bootx command
    crate::tools::runner::emit_log(app, "info", "Booting kernel...");
    crate::tools::runner::run_streaming(
        app,
        irecovery.clone(),
        &["-c".to_string(), "bootx".to_string()],
    )?;

    crate::tools::runner::emit_log(app, "info", "Bootchain sent successfully - device should boot now");
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
/// debug: true adds --debug flag (used for iBSS, not for iBEC in just boot)
fn patch_iboot32(
    app: &AppHandle,
    input: &Path,
    output: &Path,
    boot_args: &str,
    debug: bool,
) -> Result<(), AppError> {
    let binary = resolve_binary_path(app, "iBoot32Patcher").map_err(AppError::CommandFailed)?;
    
    let mut args = vec![
        input.to_string_lossy().to_string(),
        output.to_string_lossy().to_string(),
        "--rsa".to_string(),
    ];
    
    // Add --debug flag if requested (iBSS gets it, iBEC does not for just boot)
    if debug {
        args.push("--debug".to_string());
    }
    
    args.push("-b".to_string());
    args.push(boot_args.to_string());
    
    crate::tools::runner::emit_log(
        app,
        "info",
        &format!(
            "Patching iBoot {} -> {}",
            if debug { "(with --debug)" } else { "" },
            output.to_string_lossy()
        ),
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
    devicetree_path: &Path,
    kernelcache_path: &Path,
) -> Result<bool, AppError> {
    if !ibss_path.exists() {
        return Ok(false);
    }
    if !devicetree_path.exists() {
        return Ok(false);
    }
    if !kernelcache_path.exists() {
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
    let dt_mtime = fs::metadata(devicetree_path)?.modified()?;
    if dt_mtime <= ipsw_mtime {
        return Ok(false);
    }
    let kc_mtime = fs::metadata(kernelcache_path)?.modified()?;
    if kc_mtime <= ipsw_mtime {
        return Ok(false);
    }
    Ok(true)
}
