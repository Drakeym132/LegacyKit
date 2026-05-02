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
                _ => {}
            }
        }
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
}
