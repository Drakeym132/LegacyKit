//! Firmware key fetching and caching for img3 decryption.
//!
//! Provides IV and decryption keys for iBSS/iBEC components from api.ipsw.me.
//! Keys are cached locally under `<workspace>/firmware/<device>/<build>/keys.json`.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

/// Firmware keys for a single component (e.g., iBSS, iBEC).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentKeys {
    pub iv: String,
    pub key: String,
}

/// Firmware keys response from api.ipsw.me.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareKeysResponse {
    pub device: String,
    pub buildid: String,
    #[serde(default)]
    pub keys: HashMap<String, ComponentKeys>,
}

/// In-memory cache for firmware keys to avoid repeated fetches.
static KEYS_CACHE: OnceLock<std::sync::Mutex<HashMap<String, FirmwareKeysResponse>>> =
    OnceLock::new();

fn keys_cache() -> &'static std::sync::Mutex<HashMap<String, FirmwareKeysResponse>> {
    KEYS_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Cache key combining device and build ID.
fn cache_key(device: &str, build: &str) -> String {
    format!("{device}:{build}")
}

/// Fetches firmware keys for a device/build combination.
///
/// First checks in-memory cache, then local disk cache, then fetches from api.ipsw.me,
/// then falls back to TheAppleWiki. Keys are cached to `<workspace>/firmware/<device>/<build>/keys.json`.
pub async fn fetch_firmware_keys(
    app: &AppHandle,
    device: &str,
    build: &str,
) -> Result<FirmwareKeysResponse, AppError> {
    let key = cache_key(device, build);

    // Check in-memory cache
    {
        let cache = keys_cache()
            .lock()
            .map_err(|_| AppError::CommandFailed("Failed to lock keys cache".to_string()))?;
        if let Some(cached) = cache.get(&key) {
            return Ok(cached.clone());
        }
    }

    // Check disk cache
    let disk_cache_path = keys_disk_cache_path(app, device, build);
    if let Ok(cached) = read_keys_from_disk(&disk_cache_path) {
        let mut cache = keys_cache()
            .lock()
            .map_err(|_| AppError::CommandFailed("Failed to lock keys cache".to_string()))?;
        cache.insert(key.clone(), cached.clone());
        return Ok(cached);
    }

    // Try api.ipsw.me first
    let url = format!("https://api.ipsw.me/v4/keys/device/{device}/{build}");
    crate::tools::runner::emit_log(app, "info", &format!("Fetching firmware keys from {url}"));

    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "LegacyKit/1.0")
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Firmware keys request failed: {e}")))?;

    if response.status().is_success() {
        let text = response
            .text()
            .await
            .map_err(|e| AppError::CommandFailed(format!("Failed reading keys response: {e}")))?;

        if let Ok(mut keys) = serde_json::from_str::<FirmwareKeysResponse>(&text) {
            keys.device = device.to_string();
            keys.buildid = build.to_string();

            if !keys.keys.is_empty() {
                return save_and_cache_keys(app, device, build, keys);
            }
        }
    }

    // Fallback to TheAppleWiki
    crate::tools::runner::emit_log(
        app,
        "info",
        "ipsw.me keys not available, trying TheAppleWiki...",
    );
    if let Ok(keys) = fetch_keys_from_applewiki(app, device, build).await {
        if !keys.keys.is_empty() {
            return save_and_cache_keys(app, device, build, keys);
        }
    }

    Err(AppError::CommandFailed(format!(
        "Firmware keys not found for {} build {} (tried ipsw.me and TheAppleWiki)",
        device, build
    )))
}

/// Saves keys to disk and memory cache, returning the result.
fn save_and_cache_keys(
    app: &AppHandle,
    device: &str,
    build: &str,
    mut keys: FirmwareKeysResponse,
) -> Result<FirmwareKeysResponse, AppError> {
    keys.device = device.to_string();
    keys.buildid = build.to_string();

    let disk_cache_path = keys_disk_cache_path(app, device, build);
    if let Some(parent) = disk_cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&keys) {
        let _ = fs::write(&disk_cache_path, json);
    }

    let key = cache_key(device, build);
    {
        let mut cache = keys_cache()
            .lock()
            .map_err(|_| AppError::CommandFailed("Failed to lock keys cache".to_string()))?;
        cache.insert(key, keys.clone());
    }

    crate::tools::runner::emit_log(
        app,
        "info",
        &format!(
            "Cached {} firmware keys for {} build {}",
            keys.keys.len(),
            device,
            build
        ),
    );

    Ok(keys)
}

/// Fetches firmware keys from TheAppleWiki for a device/build combination.
/// Uses the MediaWiki API to parse the keys page.
async fn fetch_keys_from_applewiki(
    app: &AppHandle,
    device: &str,
    build: &str,
) -> Result<FirmwareKeysResponse, AppError> {
    // TheAppleWiki page title format: "Keys:Brighton 10B146 (iPhone5,1)"
    let page_title = build_applewiki_page_title(device, build)?;
    // URL encode the page title (spaces and parentheses)
    let encoded_title = page_title
        .replace(' ', "%20")
        .replace('(', "%28")
        .replace(')', "%29");
    let url = format!(
        "https://theapplewiki.com/api.php?action=parse&page={}&format=json&prop=text",
        encoded_title
    );

    crate::tools::runner::emit_log(
        app,
        "info",
        &format!("Fetching keys from TheAppleWiki: {}", page_title),
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("User-Agent", "LegacyKit/1.0")
        .send()
        .await
        .map_err(|e| AppError::CommandFailed(format!("TheAppleWiki request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::CommandFailed(format!(
            "TheAppleWiki returned status {}",
            response.status()
        )));
    }

    let text = response
        .text()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed reading TheAppleWiki response: {e}")))?;

    // Parse the JSON response to extract the unescaped HTML
    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| AppError::Parse(format!("Invalid TheAppleWiki JSON: {e}")))?;
    let html = json
        .get("parse")
        .and_then(|p| p.get("text"))
        .and_then(|t| t.get("*"))
        .and_then(|s| s.as_str())
        .ok_or_else(|| {
            AppError::Parse("TheAppleWiki response missing parse.text.* field".to_string())
        })?;

    parse_applewiki_html(html, device, build)
}

/// Builds TheAppleWiki page title for a device/build combination.
/// Format: "Keys:Codename_Build_(DeviceModel)"
fn build_applewiki_page_title(device: &str, build: &str) -> Result<String, AppError> {
    // Map common build IDs to codenames (iOS 6.x - 7.x)
    let codename = match build {
        "10A403" | "10A405" | "10A525" | "10A551" => "Sundance", // iOS 6.0.x
        "10B143" | "10B146" => "Brighton",                       // iOS 6.1 - 6.1.2
        "10B329" | "10B350" => "BrightonMaps",                   // iOS 6.1.3 - 6.1.4
        "11A465" | "11A501" => "Innsbruck",                      // iOS 7.0.x
        "11B511" | "11B554a" | "11B651" => "InnsbruckTaos",      // iOS 7.1.x
        "11D167" | "11D201" | "11D257" => "Sochi",               // iOS 7.1.1 - 7.1.2
        _ => {
            return Err(AppError::Parse(format!(
                "Unknown codename for build {}",
                build
            )))
        }
    };

    // Format: Keys:Brighton 10B146 (iPhone5,1) - uses spaces, not underscores
    Ok(format!("Keys:{} {} ({})", codename, build, device))
}

/// Parses TheAppleWiki HTML response to extract firmware keys.
fn parse_applewiki_html(
    html: &str,
    device: &str,
    build: &str,
) -> Result<FirmwareKeysResponse, AppError> {
    let mut keys = FirmwareKeysResponse {
        device: device.to_string(),
        buildid: build.to_string(),
        keys: HashMap::new(),
    };

    // Parse component keys from HTML using regex-like extraction
    // The HTML contains patterns like: id="keypage-ibss-iv">VALUE</code>
    // and id="keypage-ibss-key">VALUE</code>
    for component in ["iBSS", "iBEC", "iBoot", "LLB", "Kernelcache", "RecoveryMode"] {
        let iv_pattern = format!(r#"id="keypage-{}-iv">([a-f0-9]{{32}})</code>"#, component.to_lowercase());
        let key_pattern = format!(r#"id="keypage-{}-key">([a-f0-9]{{64}})</code>"#, component.to_lowercase());

        if let (Some(iv), Some(key)) = (
            extract_pattern(html, &iv_pattern),
            extract_pattern(html, &key_pattern),
        ) {
            keys.keys.insert(
                component.to_string(),
                ComponentKeys { iv, key },
            );
        }
    }

    if keys.keys.is_empty() {
        return Err(AppError::Parse(
            "No firmware keys found in TheAppleWiki response".to_string(),
        ));
    }

    Ok(keys)
}

/// Extracts a captured group from HTML using a simple pattern match.
fn extract_pattern(html: &str, pattern: &str) -> Option<String> {
    // Simple pattern matching for id="...">value</code>
    let id_start = pattern.find("id=\"")?;
    let id_end = pattern[id_start..].find("\">")? + id_start;
    let id_attr = &pattern[id_start + 4..id_end];

    // Find the id in HTML
    let search = format!("id=\"{}\">", id_attr);
    let start = html.find(&search)?;
    let value_start = start + search.len();
    // Find </code> after value_start
    let remaining = &html[value_start..];
    let value_end = remaining.find("</code>")?;
    Some(html[value_start..value_start + value_end].to_string())
}

/// Gets keys for a specific component (e.g., "iBSS", "iBEC").
///
/// Returns `None` if keys haven't been fetched yet or component doesn't exist.
pub fn get_component_keys(device: &str, build: &str, component: &str) -> Option<ComponentKeys> {
    let key = cache_key(device, build);
    let cache = keys_cache().lock().ok()?;
    let entry = cache.get(&key)?;

    // Try exact match first, then lowercase
    entry
        .keys
        .get(component)
        .cloned()
        .or_else(|| entry.keys.get(&component.to_lowercase()).cloned())
}

/// Returns the disk cache path for firmware keys.
fn keys_disk_cache_path(app: &AppHandle, device: &str, build: &str) -> PathBuf {
    let workspace = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    workspace
        .join("firmware")
        .join(device)
        .join(build)
        .join("keys.json")
}

/// Reads keys from disk cache.
fn read_keys_from_disk(path: &PathBuf) -> Result<FirmwareKeysResponse, std::io::Error> {
    let json = fs::read_to_string(path)?;
    let keys: FirmwareKeysResponse = serde_json::from_str(&json)?;
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_combines_device_and_build() {
        assert_eq!(cache_key("iPhone5,1", "10B329"), "iPhone5,1:10B329");
    }
}
