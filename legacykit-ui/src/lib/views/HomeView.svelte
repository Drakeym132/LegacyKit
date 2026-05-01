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
  <!-- Hero card: device identity + compact action toolbar -->
  <section class="hero-card">
    {#if isConnected}
      <div class="hero-device">
        <div class="device-illustration">
          <svg viewBox="0 0 120 200" fill="none" xmlns="http://www.w3.org/2000/svg" class="device-svg">
            <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.35" />
            <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.06" />
            <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.25" />
            <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.2" />
            <circle cx="60" cy="14" r="2" fill="currentColor" opacity="0.15" />
          </svg>
        </div>
        <div class="hero-text">
          <h2 class="device-name">{deviceName}</h2>
          <span class="device-product">{productType}</span>
          <DeviceStatus mode={deviceMode} />
        </div>
      </div>

      <div class="action-toolbar" role="toolbar" aria-label="Quick actions">
        {#each quickActions as action}
          <button
            class="action-tile"
            onclick={() => navigationStore.navigate(action.view)}
            title={action.description}
          >
            <span class="action-icon" aria-hidden="true">{action.icon}</span>
            <span class="action-label">{action.label}</span>
          </button>
        {/each}
      </div>
    {:else}
      <div class="hero-device">
        <div class="device-illustration disconnected">
          <svg viewBox="0 0 120 200" fill="none" xmlns="http://www.w3.org/2000/svg" class="device-svg">
            <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.15" stroke-dasharray="6 4" />
            <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.03" />
            <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.1" />
            <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.08" />
          </svg>
        </div>
        <div class="hero-text">
          <h2 class="device-name muted">No Device Connected</h2>
          <span class="device-product muted">Connect a USB device to get started</span>
        </div>
      </div>
    {/if}
  </section>

  <!-- Detail grid: each section is its own card -->
  {#if isConnected}
    <div class="detail-grid">
      {#if isNormalMode}
        {#if hasIdentity}
          <section class="detail-card">
            <h3 class="card-title">Identity</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>Device Class</dt><dd>{deviceClass}</dd></div>
              <div class="detail-row"><dt>Model</dt><dd>{model}</dd></div>
              <div class="detail-row"><dt>Model Number</dt><dd>{modelNumber}</dd></div>
              <div class="detail-row"><dt>Region</dt><dd>{regionInfo}</dd></div>
              <div class="detail-row"><dt>Color</dt><dd>{deviceColor}</dd></div>
              <div class="detail-row"><dt>Serial</dt><dd class="mono">{serial}</dd></div>
              <div class="detail-row"><dt>UDID</dt><dd class="mono truncate" title={udid}>{udid}</dd></div>
              <div class="detail-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
            </dl>
          </section>
        {/if}

        {#if hasSoftware}
          <section class="detail-card">
            <h3 class="card-title">Software</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>iOS</dt><dd>{iosVersion}</dd></div>
              <div class="detail-row"><dt>Build</dt><dd>{buildVersion}</dd></div>
              <div class="detail-row"><dt>Firmware</dt><dd class="mono truncate" title={firmwareVersion}>{firmwareVersion}</dd></div>
              <div class="detail-row"><dt>Baseband</dt><dd>{basebandVersion}</dd></div>
              <div class="detail-row"><dt>Activation</dt><dd>{activationState}</dd></div>
            </dl>
          </section>
        {/if}

        {#if hasHardware}
          <section class="detail-card">
            <h3 class="card-title">Hardware</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>CPU Architecture</dt><dd>{cpuArchitecture}</dd></div>
              <div class="detail-row"><dt>Platform</dt><dd>{hardwarePlatform}</dd></div>
              <div class="detail-row"><dt>Battery</dt><dd>{batteryCapacity}</dd></div>
              <div class="detail-row"><dt>Storage</dt><dd>{totalDisk}</dd></div>
              <div class="detail-row"><dt>Available</dt><dd>{totalAvailable}</dd></div>
              <div class="detail-row"><dt>Passcode</dt><dd>{passwordProtected}</dd></div>
            </dl>
          </section>
        {/if}

        {#if hasNetwork}
          <section class="detail-card">
            <h3 class="card-title">Network</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>Cellular</dt><dd>{telephonyCapability}</dd></div>
              <div class="detail-row"><dt>IMEI</dt><dd class="mono">{imei}</dd></div>
              <div class="detail-row"><dt>WiFi MAC</dt><dd class="mono">{wifiAddress}</dd></div>
              <div class="detail-row"><dt>Bluetooth MAC</dt><dd class="mono">{bluetoothAddress}</dd></div>
            </dl>
          </section>
        {/if}
      {:else}
        {#if hasRecoveryInfo}
          <section class="detail-card">
            <h3 class="card-title">Recovery Info</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
              <div class="detail-row"><dt>Serial</dt><dd class="mono">{serial}</dd></div>
              <div class="detail-row"><dt>Model</dt><dd>{model}</dd></div>
              <div class="detail-row"><dt>Product</dt><dd>{productType}</dd></div>
              <div class="detail-row"><dt>CPID</dt><dd class="mono">{cpid}</dd></div>
              <div class="detail-row"><dt>CPRV</dt><dd class="mono">{cprv}</dd></div>
              <div class="detail-row"><dt>Board ID</dt><dd class="mono">{bdid}</dd></div>
              <div class="detail-row"><dt>iBoot Flags</dt><dd class="mono">{ibfl}</dd></div>
            </dl>
          </section>
        {/if}

        {#if hasNonces}
          <section class="detail-card">
            <h3 class="card-title">Nonces</h3>
            <dl class="detail-list">
              <div class="detail-row"><dt>APNonce</dt><dd class="mono truncate" title={apnonce}>{apnonce}</dd></div>
              <div class="detail-row"><dt>SEPNonce</dt><dd class="mono truncate" title={sepnonce}>{sepnonce}</dd></div>
            </dl>
          </section>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .home-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    height: 100%;
    min-height: 0;
    padding: var(--spacing-lg);
    overflow-y: auto;
  }

  /* ── Hero card ── */
  .hero-card {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-lg);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md) var(--spacing-lg);
    flex-wrap: wrap;
  }

  .hero-device {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    min-width: 0;
    flex: 1 1 auto;
  }

  .device-illustration {
    width: 56px;
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

  .hero-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .device-name {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.2;
    word-break: break-word;
  }

  .device-name.muted {
    color: var(--color-text-secondary);
    font-weight: 600;
  }

  .device-product {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  /* ── Action toolbar (compact) ── */
  .action-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    flex-shrink: 0;
  }

  .action-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    width: 64px;
    padding: 8px 6px;
    background: var(--color-bg-secondary);
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.15s ease;
    font-family: inherit;
    color: var(--color-text-primary);
  }

  .action-tile:hover {
    background: var(--color-bg-elevated);
    border-color: var(--color-accent);
    transform: translateY(-1px);
  }

  .action-tile:active {
    transform: translateY(0);
  }

  .action-icon {
    font-size: 1.125rem;
    line-height: 1;
  }

  .action-label {
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    white-space: nowrap;
  }

  .action-tile:hover .action-label {
    color: var(--color-text-primary);
  }

  /* ── Detail grid ── */
  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--spacing-md);
    align-items: start;
  }

  .detail-card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    min-width: 0;
  }

  .card-title {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm) 0;
  }

  .detail-list {
    margin: 0;
    display: flex;
    flex-direction: column;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--spacing-sm);
    padding: 7px 0;
    border-bottom: 1px solid var(--color-border);
    min-width: 0;
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-row dt {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    flex-shrink: 0;
  }

  .detail-row dd {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-primary);
    font-weight: 500;
    text-align: right;
    min-width: 0;
  }

  .detail-row dd.mono {
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.6875rem;
    letter-spacing: -0.01em;
  }

  .detail-row dd.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Responsive ── */
  @media (max-width: 640px) {
    .hero-card {
      flex-direction: column;
      align-items: flex-start;
    }

    .action-toolbar {
      width: 100%;
      justify-content: flex-start;
    }
  }
</style>
