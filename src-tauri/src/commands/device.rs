use crate::error::AppError;
use crate::models::device::DeviceInfo;
use crate::platform::resolve_binary_path;
use crate::services::device_parser::{parse_ideviceinfo, parse_irecovery_q};
use std::process::Command;

#[tauri::command]
pub async fn detect_device(app: tauri::AppHandle) -> Result<DeviceInfo, AppError> {
    // Try ideviceinfo first (normal mode)
    if let Ok(ideviceinfo_path) = resolve_binary_path(&app, "ideviceinfo") {
        if let Ok(output) = Command::new(&ideviceinfo_path).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Ok(parse_ideviceinfo(&stdout));
            }
        }
    }
    
    // Fallback: try irecovery for Recovery/DFU mode
    if let Ok(irecovery_path) = resolve_binary_path(&app, "irecovery") {
        if let Ok(output) = Command::new(&irecovery_path).arg("-q").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Ok(parse_irecovery_q(&stdout));
            }
        }
    }
    
    // No device found
    Ok(DeviceInfo::default())
}
