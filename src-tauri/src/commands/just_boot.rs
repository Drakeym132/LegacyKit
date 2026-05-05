use crate::error::AppError;
use crate::models::just_boot::{JustBootEntry, JustBootEntryInput, PrepareAndJustBootRequest};
use crate::services::{bootchain, just_boot_store};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[tauri::command]
pub async fn list_just_boot_history(app: AppHandle) -> Result<Vec<JustBootEntry>, AppError> {
    let mut entries = just_boot_store::list(&app)?;
    entries.sort_by_key(|entry| std::cmp::Reverse(parse_rfc3339(&entry.last_booted_at)));
    Ok(entries)
}

#[tauri::command]
pub async fn record_just_boot(
    app: AppHandle,
    entry: JustBootEntryInput,
) -> Result<JustBootEntry, AppError> {
    let now = Utc::now().to_rfc3339();
    let value = JustBootEntry {
        id: Uuid::new_v4().to_string(),
        ecid: entry.ecid,
        product_type: entry.product_type,
        device_name: entry.device_name,
        build_id: entry.build_id,
        ios_version: entry.ios_version,
        boot_args: entry.boot_args,
        repacked_ibss_path: entry.repacked_ibss_path,
        repacked_ibec_path: entry.repacked_ibec_path,
        decrypted_devicetree_path: entry.decrypted_devicetree_path,
        decrypted_kernelcache_path: entry.decrypted_kernelcache_path,
        source_ipsw_path: entry.source_ipsw_path,
        created_at: now.clone(),
        last_booted_at: now,
    };
    just_boot_store::upsert(&app, value)
}

#[tauri::command]
pub async fn forget_just_boot(app: AppHandle, id: String) -> Result<(), AppError> {
    just_boot_store::remove(&app, id.trim())
}

#[tauri::command]
pub async fn prepare_and_just_boot(
    app: AppHandle,
    request: PrepareAndJustBootRequest,
) -> Result<JustBootEntry, AppError> {
    let ecid = normalize_ecid(&request.ecid);
    if ecid.is_empty() {
        return Err(AppError::Parse("ECID is required".to_string()));
    }

    let product_type = request.product_type.trim().to_string();
    if product_type.is_empty() {
        return Err(AppError::Parse("Product type is required".to_string()));
    }

    let build_id = request.build_id.trim().to_string();
    if build_id.is_empty() {
        return Err(AppError::Parse("Build ID is required".to_string()));
    }

    let ipsw_path = request.ipsw_path.trim().to_string();
    if ipsw_path.is_empty() {
        return Err(AppError::Parse("IPSW path is required".to_string()));
    }
    if !Path::new(&ipsw_path).exists() {
        return Err(AppError::Parse(format!("IPSW does not exist: {ipsw_path}")));
    }

    let boot_args = request
        .boot_args
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("pio-error=0 -v")
        .to_string();

    // Auto-detect processor generation from product_type
    let proc_gen = infer_processor_generation(&product_type);
    // img4 starts at A7; A4–A6 devices use img3 containers
    let use_img4 = proc_gen.map(|value| value >= 7).unwrap_or(true);

    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!(
            "Processor generation: {:?}, using {} format",
            proc_gen,
            if use_img4 { "img4" } else { "img3" }
        ),
    );

    let cache_root = app
        .path()
        .app_cache_dir()
        .map_err(|err| AppError::CommandFailed(format!("Failed to get app cache dir: {err}")))?;
    let cache_dir = cache_root
        .join("just_boot")
        .join(&product_type)
        .join(format!("ramdisk_{build_id}"));
    fs::create_dir_all(&cache_dir)?;

    crate::tools::runner::emit_log(
        &app,
        "info",
        &format!(
            "Preparing Just Boot bootchain for {product_type} {build_id} (cache: {})",
            cache_dir.to_string_lossy()
        ),
    );

    // Prepare the bootchain (iBSS, iBEC, DeviceTree, Kernelcache)
    // Note: include_ibec is auto-determined from build_id and product_type
    let (prepared, _) = bootchain::prepare_cached_bootchain(
        &app,
        &ipsw_path,
        &cache_dir,
        &boot_args,
        use_img4,
        &product_type,
        &build_id,
    )
    .await?;

    let now = Utc::now().to_rfc3339();
    let draft = JustBootEntry {
        id: Uuid::new_v4().to_string(),
        ecid,
        product_type,
        device_name: request.device_name,
        build_id,
        ios_version: request.ios_version,
        boot_args: Some(boot_args),
        repacked_ibss_path: Some(prepared.repacked_ibss_path.clone()),
        repacked_ibec_path: prepared.repacked_ibec_path.clone(),
        decrypted_devicetree_path: prepared.decrypted_devicetree_path.clone(),
        decrypted_kernelcache_path: prepared.decrypted_kernelcache_path.clone(),
        source_ipsw_path: Some(ipsw_path),
        created_at: now.clone(),
        last_booted_at: now,
    };

    // Send the bootchain via irecovery (correct for pwnDFU mode)
    // The new sequence: iBSS -> iBEC -> DeviceTree -> devicetree -> Kernelcache -> bootx
    let boot_result = bootchain::send_bootchain_pwndfu(
        &app,
        draft
            .repacked_ibss_path
            .as_deref()
            .ok_or_else(|| AppError::Parse("Missing repacked iBSS path".to_string()))?,
        draft.repacked_ibec_path.as_deref(),
        draft.decrypted_devicetree_path.as_deref(),
        draft.decrypted_kernelcache_path.as_deref(),
        proc_gen,
    );

    let stored = just_boot_store::upsert(&app, draft)?;
    match boot_result {
        Ok(_) => Ok(stored),
        Err(err) => Err(err),
    }
}

fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn normalize_ecid(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_ascii_lowercase()
}

fn infer_processor_generation(product_type: &str) -> Option<u8> {
    if matches_any(product_type, &["iPhone1,1", "iPhone1,2", "iPod1,1"]) {
        return Some(1);
    }
    if matches_any(product_type, &["iPhone2,1", "iPod2,1"]) {
        return Some(2);
    }
    if product_type == "iPod3,1" {
        return Some(3);
    }
    if product_family_in(product_type, "iPhone3", 1..=3)
        || product_type == "iPad1,1"
        || product_type == "iPod4,1"
    {
        return Some(4);
    }
    if product_type == "iPhone4,1"
        || product_family_in(product_type, "iPad2", 1..=7)
        || product_family_in(product_type, "iPad3", 1..=3)
        || product_type == "iPod5,1"
    {
        return Some(5);
    }
    if product_family_in(product_type, "iPhone5", 1..=4)
        || product_family_in(product_type, "iPad3", 4..=6)
    {
        return Some(6);
    }
    if product_family_in(product_type, "iPhone6", 1..=2)
        || product_family_in(product_type, "iPad4", 1..=9)
    {
        return Some(7);
    }
    if product_family_in(product_type, "iPhone7", 1..=2)
        || product_type == "iPod7,1"
        || product_family_in(product_type, "iPad5", 1..=4)
    {
        return Some(8);
    }
    if product_family_in(product_type, "iPhone8", 1..=4)
        || product_family_in(product_type, "iPad6", 3..=12)
    {
        return Some(9);
    }
    if product_family_in(product_type, "iPhone9", 1..=4)
        || product_family_in(product_type, "iPad7", 1..=4)
    {
        return Some(10);
    }
    None
}

fn matches_any(value: &str, candidates: &[&str]) -> bool {
    candidates.contains(&value)
}

fn product_family_in(
    product_type: &str,
    family: &str,
    range: std::ops::RangeInclusive<u8>,
) -> bool {
    let Some(suffix) = product_type.strip_prefix(family) else {
        return false;
    };
    let Some(number) = suffix.strip_prefix(',') else {
        return false;
    };

    number
        .parse::<u8>()
        .map(|value| range.contains(&value))
        .unwrap_or(false)
}
