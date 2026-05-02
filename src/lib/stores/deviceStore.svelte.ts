export type DeviceMode = 'Normal' | 'Recovery' | 'DFU' | 'kDFU' | 'pwnDFU';

export interface DeviceInfo {
    connected: boolean;
    name: string | null;
    udid: string | null;
    ecid: string | null;
    serial: string | null;
    model: string | null;
    product_type: string | null;
    ios_version: string | null;
    mode: DeviceMode;

    // Normal mode fields
    build_version: string | null;
    cpu_architecture: string | null;
    hardware_platform: string | null;
    device_color: string | null;
    device_class: string | null;
    model_number: string | null;
    region_info: string | null;
    activation_state: string | null;
    baseband_version: string | null;
    firmware_version: string | null;
    total_disk_capacity: number | null;
    total_data_available: number | null;
    battery_current_capacity: number | null;
    password_protected: boolean | null;
    telephony_capability: boolean | null;
    imei: string | null;
    wifi_address: string | null;
    bluetooth_address: string | null;

    // Recovery/DFU mode fields
    cpid: string | null;
    cprv: string | null;
    bdid: string | null;
    ibfl: string | null;
    apnonce: string | null;
    sepnonce: string | null;
    pwnd: string | null;
    srtg: string | null;
}

const EMPTY_DEVICE_INFO: DeviceInfo = {
    connected: false,
    name: null,
    udid: null,
    ecid: null,
    serial: null,
    model: null,
    product_type: null,
    ios_version: null,
    mode: 'Normal',

    // Normal mode fields
    build_version: null,
    cpu_architecture: null,
    hardware_platform: null,
    device_color: null,
    device_class: null,
    model_number: null,
    region_info: null,
    activation_state: null,
    baseband_version: null,
    firmware_version: null,
    total_disk_capacity: null,
    total_data_available: null,
    battery_current_capacity: null,
    password_protected: null,
    telephony_capability: null,
    imei: null,
    wifi_address: null,
    bluetooth_address: null,

    // Recovery/DFU mode fields
    cpid: null,
    cprv: null,
    bdid: null,
    ibfl: null,
    apnonce: null,
    sepnonce: null,
    pwnd: null,
    srtg: null
};

class DeviceStore {
    state = $state<DeviceInfo>({ ...EMPTY_DEVICE_INFO });

    updateFromBackend(info: DeviceInfo) {
        this.state = { ...info };
    }

    setDevice(info: Partial<DeviceInfo>) {
        this.state = { ...this.state, ...info, connected: true };
    }

    optimisticallySetMode(mode: DeviceMode, pwnd: string | null = null) {
        this.state = { ...this.state, mode, pwnd: pwnd ?? this.state.pwnd };
    }

    clearDevice() {
        this.state = { ...EMPTY_DEVICE_INFO };
    }
}

export const deviceStore = new DeviceStore();
