<script lang="ts">
  import { deviceStore } from '../stores/deviceStore.svelte';
  import { navigationStore } from '../stores/navigationStore.svelte';
  import type { ViewName } from '../stores/navigationStore.svelte';
  import DeviceStatus from '../components/device/DeviceStatus.svelte';

  function formatBytes(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  let isConnected = $derived(deviceStore.state.connected);
  let deviceName = $derived(deviceStore.state.name ?? 'Unknown Device');
  let productType = $derived(deviceStore.state.product_type ?? '—');
  let deviceMode = $derived(deviceStore.state.mode);

  // Identity
  let deviceClass = $derived(deviceStore.state.device_class ?? '—');
  let model = $derived(deviceStore.state.model ?? '—');
  let modelNumber = $derived(deviceStore.state.model_number ?? '—');
  let regionInfo = $derived(deviceStore.state.region_info ?? '—');
  let deviceColor = $derived(deviceStore.state.device_color ?? '—');
  let serial = $derived(deviceStore.state.serial ?? '—');
  let udid = $derived(deviceStore.state.udid ?? '—');
  let ecid = $derived(deviceStore.state.ecid ?? '—');

  // Software
  let iosVersion = $derived(deviceStore.state.ios_version ?? '—');
  let buildVersion = $derived(deviceStore.state.build_version ?? '—');
  let firmwareVersion = $derived(deviceStore.state.firmware_version ?? '—');
  let basebandVersion = $derived(deviceStore.state.baseband_version ?? '—');
  let activationState = $derived(deviceStore.state.activation_state ?? '—');

  // Hardware
  let cpuArchitecture = $derived(deviceStore.state.cpu_architecture ?? '—');
  let hardwarePlatform = $derived(deviceStore.state.hardware_platform ?? '—');
  let batteryCapacity = $derived(deviceStore.state.battery_current_capacity != null ? `${deviceStore.state.battery_current_capacity}%` : '—');
  let totalDisk = $derived(deviceStore.state.total_disk_capacity != null ? formatBytes(deviceStore.state.total_disk_capacity) : '—');
  let totalAvailable = $derived(deviceStore.state.total_data_available != null ? formatBytes(deviceStore.state.total_data_available) : '—');
  let passwordProtected = $derived(deviceStore.state.password_protected != null ? (deviceStore.state.password_protected ? 'Yes' : 'No') : '—');

  // Network
  let telephonyCapability = $derived(deviceStore.state.telephony_capability != null ? (deviceStore.state.telephony_capability ? 'Yes' : 'No') : '—');
  let imei = $derived(deviceStore.state.imei ?? '—');
  let wifiAddress = $derived(deviceStore.state.wifi_address ?? '—');
  let bluetoothAddress = $derived(deviceStore.state.bluetooth_address ?? '—');

  // Recovery/DFU fields
  let cpid = $derived(deviceStore.state.cpid ?? '—');
  let cprv = $derived(deviceStore.state.cprv ?? '—');
  let bdid = $derived(deviceStore.state.bdid ?? '—');
  let ibfl = $derived(deviceStore.state.ibfl ?? '—');
  let apnonce = $derived(deviceStore.state.apnonce ?? '—');
  let sepnonce = $derived(deviceStore.state.sepnonce ?? '—');

  // Section visibility checks (Normal mode)
  let hasIdentity = $derived(
    deviceStore.state.device_class != null || deviceStore.state.model != null ||
    deviceStore.state.model_number != null || deviceStore.state.region_info != null ||
    deviceStore.state.device_color != null || deviceStore.state.serial != null ||
    deviceStore.state.udid != null || deviceStore.state.ecid != null
  );
  let hasSoftware = $derived(
    deviceStore.state.ios_version != null || deviceStore.state.build_version != null ||
    deviceStore.state.firmware_version != null || deviceStore.state.baseband_version != null ||
    deviceStore.state.activation_state != null
  );
  let hasHardware = $derived(
    deviceStore.state.cpu_architecture != null || deviceStore.state.hardware_platform != null ||
    deviceStore.state.battery_current_capacity != null || deviceStore.state.total_disk_capacity != null ||
    deviceStore.state.total_data_available != null || deviceStore.state.password_protected != null
  );
  let hasNetwork = $derived(
    deviceStore.state.telephony_capability != null || deviceStore.state.imei != null ||
    deviceStore.state.wifi_address != null || deviceStore.state.bluetooth_address != null
  );

  // Section visibility checks (Recovery/DFU mode)
  let hasRecoveryInfo = $derived(
    deviceStore.state.ecid != null || deviceStore.state.serial != null ||
    deviceStore.state.model != null || deviceStore.state.product_type != null ||
    deviceStore.state.cpid != null || deviceStore.state.cprv != null ||
    deviceStore.state.bdid != null || deviceStore.state.ibfl != null
  );
  let hasNonces = $derived(
    deviceStore.state.apnonce != null || deviceStore.state.sepnonce != null
  );

  let isNormalMode = $derived(deviceMode === 'Normal');

  const quickActions: { label: string; icon: string; view: ViewName; description: string }[] = [
    { label: 'Restore', icon: '⬇️', view: 'restore', description: 'Restore & downgrade iOS' },
    { label: 'Jailbreak', icon: '🔓', view: 'jailbreak', description: 'Jailbreak legacy devices' },
    { label: 'SHSH Blobs', icon: '💾', view: 'shsh', description: 'Save & manage blobs' },
    { label: 'SSH Ramdisk', icon: '🖥️', view: 'ssh-ramdisk', description: 'Boot SSH ramdisk' },
    { label: 'Apps', icon: '📱', view: 'apps', description: 'Manage device apps' },
    { label: 'Utilities', icon: '🔧', view: 'utilities', description: 'Device utilities' },
  ];
</script>

<div class="home-view">
  <div class="unified-pane">
    <!-- Left: Device Details -->
    <div class="device-side">
      {#if isConnected}
        <div class="device-illustration">
          <svg viewBox="0 0 120 200" fill="none" xmlns="http://www.w3.org/2000/svg" class="device-svg">
            <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.35" />
            <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.06" />
            <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.25" />
            <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.2" />
            <circle cx="60" cy="14" r="2" fill="currentColor" opacity="0.15" />
          </svg>
        </div>

        <div class="device-headline">
          <h2 class="device-name">{deviceName}</h2>
          <span class="device-product">{productType}</span>
          <DeviceStatus mode={deviceMode} />
        </div>

        <div class="device-details">
          {#if isNormalMode}
            {#if hasIdentity}
              <div class="detail-section">
                <div class="section-label">Identity</div>
                <div class="detail-row">
                  <span class="detail-label">Device Class</span>
                  <span class="detail-value">{deviceClass}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Model</span>
                  <span class="detail-value">{model}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Model Number</span>
                  <span class="detail-value">{modelNumber}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Region</span>
                  <span class="detail-value">{regionInfo}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Color</span>
                  <span class="detail-value">{deviceColor}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Serial</span>
                  <span class="detail-value mono">{serial}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">UDID</span>
                  <span class="detail-value mono">{udid}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">ECID</span>
                  <span class="detail-value mono">{ecid}</span>
                </div>
              </div>
            {/if}

            {#if hasSoftware}
              <div class="detail-section">
                <div class="section-label">Software</div>
                <div class="detail-row">
                  <span class="detail-label">iOS</span>
                  <span class="detail-value">{iosVersion}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Build</span>
                  <span class="detail-value">{buildVersion}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Firmware</span>
                  <span class="detail-value">{firmwareVersion}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Baseband</span>
                  <span class="detail-value">{basebandVersion}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Activation</span>
                  <span class="detail-value">{activationState}</span>
                </div>
              </div>
            {/if}

            {#if hasHardware}
              <div class="detail-section">
                <div class="section-label">Hardware</div>
                <div class="detail-row">
                  <span class="detail-label">CPU Architecture</span>
                  <span class="detail-value">{cpuArchitecture}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Platform</span>
                  <span class="detail-value">{hardwarePlatform}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Battery</span>
                  <span class="detail-value">{batteryCapacity}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Storage</span>
                  <span class="detail-value">{totalDisk}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Available</span>
                  <span class="detail-value">{totalAvailable}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Passcode</span>
                  <span class="detail-value">{passwordProtected}</span>
                </div>
              </div>
            {/if}

            {#if hasNetwork}
              <div class="detail-section">
                <div class="section-label">Network</div>
                <div class="detail-row">
                  <span class="detail-label">Cellular</span>
                  <span class="detail-value">{telephonyCapability}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">IMEI</span>
                  <span class="detail-value mono">{imei}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">WiFi MAC</span>
                  <span class="detail-value mono">{wifiAddress}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Bluetooth MAC</span>
                  <span class="detail-value mono">{bluetoothAddress}</span>
                </div>
              </div>
            {/if}
          {:else}
            {#if hasRecoveryInfo}
              <div class="detail-section">
                <div class="section-label">Recovery Info</div>
                <div class="detail-row">
                  <span class="detail-label">ECID</span>
                  <span class="detail-value mono">{ecid}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Serial</span>
                  <span class="detail-value mono">{serial}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Model</span>
                  <span class="detail-value">{model}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Product</span>
                  <span class="detail-value">{productType}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">CPID</span>
                  <span class="detail-value mono">{cpid}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">CPRV</span>
                  <span class="detail-value mono">{cprv}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">Board ID</span>
                  <span class="detail-value mono">{bdid}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">iBoot Flags</span>
                  <span class="detail-value mono">{ibfl}</span>
                </div>
              </div>
            {/if}

            {#if hasNonces}
              <div class="detail-section">
                <div class="section-label">Nonces</div>
                <div class="detail-row">
                  <span class="detail-label">APNonce</span>
                  <span class="detail-value mono">{apnonce}</span>
                </div>
                <div class="detail-row">
                  <span class="detail-label">SEPNonce</span>
                  <span class="detail-value mono">{sepnonce}</span>
                </div>
              </div>
            {/if}
          {/if}
        </div>
      {:else}
        <div class="device-illustration disconnected">
          <svg viewBox="0 0 120 200" fill="none" xmlns="http://www.w3.org/2000/svg" class="device-svg">
            <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.15" stroke-dasharray="6 4" />
            <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.03" />
            <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.1" />
            <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.08" />
          </svg>
        </div>
        <div class="device-headline">
          <h2 class="device-name muted">No Device Connected</h2>
          <span class="device-product muted">Connect a USB device to get started</span>
        </div>
      {/if}
    </div>

    <!-- Vertical Divider -->
    <div class="vertical-divider"></div>

    <!-- Right: Quick Actions -->
    <div class="actions-side">
      <h3 class="section-title">Quick Actions</h3>
      <div class="actions-grid">
        {#each quickActions as action}
          <button
            class="action-card"
            onclick={() => navigationStore.navigate(action.view)}
          >
            <span class="action-icon">{action.icon}</span>
            <div class="action-text">
              <span class="action-label">{action.label}</span>
              <span class="action-description">{action.description}</span>
            </div>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .home-view {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    height: 100%;
    min-height: 0;
    padding: var(--spacing-lg);
    overflow-y: auto;
  }

  /* ── Unified Pane ── */
  .unified-pane {
    width: 100%;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: row;
    gap: var(--spacing-lg);
    min-height: 0;
  }

  /* ── Left: Device Details ── */
  .device-side {
    flex: 0 0 260px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-md);
    min-width: 0;
  }

  .device-illustration {
    width: 80px;
    color: var(--color-text-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .device-illustration.disconnected {
    opacity: 0.45;
  }

  .device-svg {
    width: 100%;
    height: auto;
  }

  .device-headline {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    text-align: center;
    width: 100%;
  }

  .device-name {
    font-size: 1.125rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.3;
    word-break: break-word;
  }

  .device-name.muted {
    color: var(--color-text-secondary);
    font-size: 1rem;
  }

  .device-product {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .device-product.muted {
    font-size: 0.75rem;
    font-weight: 400;
  }

  .device-details {
    width: 100%;
    display: flex;
    flex-direction: column;
    border-top: 1px solid var(--color-border);
    padding-top: var(--spacing-sm);
    overflow-y: auto;
    max-height: calc(100vh - 260px);
  }

  .detail-section {
    margin-top: var(--spacing-sm);
  }

  .detail-section:first-child {
    margin-top: 0;
  }

  .section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-secondary);
    margin-bottom: 4px;
    opacity: 0.7;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 5px 0;
    gap: var(--spacing-sm);
    border-bottom: 1px solid var(--color-border);
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-label {
    font-size: 0.688rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    flex-shrink: 0;
  }

  .detail-value {
    font-size: 0.688rem;
    color: var(--color-text-primary);
    font-weight: 500;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .detail-value.mono {
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.625rem;
    letter-spacing: -0.01em;
  }

  /* ── Vertical Divider ── */
  .vertical-divider {
    width: 1px;
    background: var(--color-border);
    flex-shrink: 0;
    align-self: stretch;
  }

  /* ── Right: Quick Actions ── */
  .actions-side {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .section-title {
    font-size: 0.938rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .actions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--spacing-sm);
    width: 100%;
  }

  .action-card {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: left;
    font-family: inherit;
    min-width: 0;
  }

  .action-card:hover {
    border-color: var(--color-accent);
    transform: translateY(-1px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  }

  .action-icon {
    font-size: 1.125rem;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-secondary);
    border-radius: var(--radius-sm);
  }

  .action-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .action-label {
    font-size: 0.813rem;
    font-weight: 600;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .action-description {
    font-size: 0.625rem;
    color: var(--color-text-secondary);
    line-height: 1.4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* ── Responsive: stack at narrow widths ── */
  @media (max-width: 600px) {
    .unified-pane {
      flex-direction: column;
    }

    .device-side {
      flex: none;
      width: 100%;
    }

    .vertical-divider {
      width: 100%;
      height: 1px;
      align-self: auto;
