use crate::models::device::{DeviceInfo, DeviceMode};

pub fn parse_ideviceinfo(stdout: &str) -> DeviceInfo {
    let mut info = DeviceInfo {
        connected: true,
        mode: DeviceMode::Normal,
        ..Default::default()
    };

    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().to_string();
            match key {
                "DeviceName" => info.name = Some(value),
                "UniqueDeviceID" => info.udid = Some(value),
                "SerialNumber" => info.serial = Some(value),
                "ProductType" => info.product_type = Some(value),
                "HardwareModel" => info.model = Some(value),
                "ProductVersion" => info.ios_version = Some(value),
                "UniqueChipID" => info.ecid = Some(value),
                "BuildVersion" => info.build_version = Some(value),
                "CPUArchitecture" => info.cpu_architecture = Some(value),
                "HardwarePlatform" => info.hardware_platform = Some(value),
                "DeviceColor" => info.device_color = Some(value),
                "DeviceClass" => info.device_class = Some(value),
                "ModelNumber" => info.model_number = Some(value),
                "RegionInfo" => info.region_info = Some(value),
                "ActivationState" => info.activation_state = Some(value),
                "BasebandVersion" => info.baseband_version = Some(value),
                "FirmwareVersion" => info.firmware_version = Some(value),
                "TotalDiskCapacity" => info.total_disk_capacity = value.parse().ok(),
                "TotalDataAvailable" => info.total_data_available = value.parse().ok(),
                "BatteryCurrentCapacity" => info.battery_current_capacity = value.parse().ok(),
                "PasswordProtected" => info.password_protected = Some(value == "true"),
                "TelephonyCapability" => info.telephony_capability = Some(value == "true"),
                "InternationalMobileEquipmentIdentity" => info.imei = Some(value),
                "WiFiAddress" => info.wifi_address = Some(value),
                "BluetoothAddress" => info.bluetooth_address = Some(value),
                _ => {}
            }
        }
    }

    info
}

pub fn parse_irecovery_q(stdout: &str) -> DeviceInfo {
    let mut info = DeviceInfo {
        connected: true,
        mode: DeviceMode::Recovery,
        ..Default::default()
    };

    for line in stdout.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().to_string();
            match key {
                "ECID" => info.ecid = Some(value),
                "SRNM" => info.serial = Some(value),
                "PRODUCT" | "ProductType" => info.product_type = Some(value),
                "MODEL" => info.model = Some(value),
                "MODE" => {
                    info.mode = match value.as_str() {
                        "DFU" => DeviceMode::DFU,
                        "Recovery" => DeviceMode::Recovery,
                        _ => DeviceMode::Recovery,
                    };
                }
                "CPID" => info.cpid = Some(value),
                "CPRV" => info.cprv = Some(value),
                "BDID" => info.bdid = Some(value),
                "IBFL" => info.ibfl = Some(value),
                "NONC" => info.apnonce = Some(value),
                "SNON" => info.sepnonce = Some(value),
                "PWND" => info.pwnd = Some(value),
                "SRTG" => info.srtg = Some(value),
                _ => {}
            }
        }
    }

    let saw_mode_dfu = matches!(info.mode, DeviceMode::DFU);
    let has_non_empty_pwnd = info.pwnd.as_deref().is_some_and(|s| !s.is_empty());

    // PWND is only emitted by irecovery once a checkm8-style exploit has run,
    // so its presence promotes "DFU" to "pwnDFU".
    if saw_mode_dfu && has_non_empty_pwnd {
        info.mode = DeviceMode::PwnDFU;
    }

    // A6 devices (ipwnder) report pwned iBSS via SRTG rather than PWND.
    // Unpwned A6 DFU shows SRTG like "iBoot-1145.3.3" or "[iBoot-1940.10]";
    // after a successful ipwnder exploit, SRTG becomes "N/A". Only promote
    // when SRTG is explicitly absent/N/A (the pwn signal), NOT just any
    // non-iBoot string, to avoid false positives on unpwned devices.
    if saw_mode_dfu
        && !has_non_empty_pwnd
        && info
            .srtg
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("N/A"))
    {
        info.mode = DeviceMode::PwnDFU;
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ideviceinfo_happy_path() {
        let out = r#"DeviceName: Alice's iPhone
UniqueDeviceID: 00008020-001D2D2234567890
SerialNumber: F2LQWXYZABC1
ProductType: iPhone6,1
ProductVersion: 10.3.3
UniqueChipID: 1234567890123456
BuildVersion: 14G60
TotalDiskCapacity: 123
PasswordProtected: true
TelephonyCapability: false
WiFiAddress: 00:11:22:33:44:55
"#;

        let info = parse_ideviceinfo(out);
        assert!(info.connected);
        assert!(matches!(info.mode, DeviceMode::Normal));
        assert_eq!(info.name.as_deref(), Some("Alice's iPhone"));
        assert_eq!(info.product_type.as_deref(), Some("iPhone6,1"));
        assert_eq!(info.total_disk_capacity, Some(123));
        assert_eq!(info.password_protected, Some(true));
        assert_eq!(info.telephony_capability, Some(false));
    }

    #[test]
    fn parse_ideviceinfo_ignores_unknown_keys() {
        let out = r#"UnknownKey: something
AnotherWeirdKey: 42
"#;

        let info = parse_ideviceinfo(out);
        assert!(info.connected);
        assert!(matches!(info.mode, DeviceMode::Normal));
        assert_eq!(info.product_type, None);
    }

    #[test]
    fn parse_irecovery_q_dfu_path() {
        let out = r#"ECID: 0x1234ABCD
CPID: 0x8960
BDID: 0x0A
MODE: DFU
PRODUCT: iPhone6,1
"#;

        let info = parse_irecovery_q(out);
        assert!(info.connected);
        assert!(matches!(info.mode, DeviceMode::DFU));
        assert_eq!(info.ecid.as_deref(), Some("0x1234ABCD"));
        assert_eq!(info.cpid.as_deref(), Some("0x8960"));
        assert_eq!(info.bdid.as_deref(), Some("0x0A"));
    }

    #[test]
    fn parse_irecovery_q_recovery_and_unknown_mode_default_to_recovery() {
        let recovery = parse_irecovery_q("MODE: Recovery\n");
        assert!(matches!(recovery.mode, DeviceMode::Recovery));

        let unknown = parse_irecovery_q("MODE: WTF\n");
        assert!(matches!(unknown.mode, DeviceMode::Recovery));
    }

    #[test]
    fn parse_irecovery_q_pwndfu_path() {
        let out = r#"ECID: 0x1234ABCD
CPID: 0x8960
BDID: 0x0A
MODE: DFU
PRODUCT: iPhone6,1
PWND: gaster
SRTG: [iBoot-1940.10]
"#;

        let info = parse_irecovery_q(out);
        assert!(matches!(info.mode, DeviceMode::PwnDFU));
        assert_eq!(info.pwnd.as_deref(), Some("gaster"));
        assert_eq!(info.srtg.as_deref(), Some("[iBoot-1940.10]"));
    }

    #[test]
    fn parse_irecovery_q_dfu_with_empty_pwnd_stays_dfu() {
        let out = "MODE: DFU\nPWND:\n";
        let info = parse_irecovery_q(out);
        assert!(matches!(info.mode, DeviceMode::DFU));
    }

    #[test]
    fn parse_irecovery_q_a6_pwndfu_via_srtg() {
        // A6 devices (ipwnder) report pwned iBSS via SRTG="N/A" rather than PWND
        let out = r#"ECID: 0x00000301b418b38a
CPID: 0x8955
BDID: 0x00
MODE: DFU
PRODUCT: iPad3,4
SRTG: N/A
"#;
        let info = parse_irecovery_q(out);
        assert!(matches!(info.mode, DeviceMode::PwnDFU));
        assert_eq!(info.srtg.as_deref(), Some("N/A"));
    }

    #[test]
    fn parse_irecovery_q_dfu_with_iboot_srtg_stays_dfu() {
        // Unpwned A6 (real irecovery output: no brackets)
        let out = "MODE: DFU\nSRTG: iBoot-1145.3.3\n";
        let info = parse_irecovery_q(out);
        assert!(matches!(info.mode, DeviceMode::DFU));
    }

    #[test]
    fn parse_irecovery_q_dfu_with_bracketed_iboot_srtg_stays_dfu() {
        // Unpwned A6 (alternate bracketed form)
        let out = "MODE: DFU\nSRTG: [iBoot-1940.10]\n";
        let info = parse_irecovery_q(out);
        assert!(matches!(info.mode, DeviceMode::DFU));
    }
}
