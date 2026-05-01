<script lang="ts">
  import { deviceStore } from '../stores/deviceStore.svelte';
  import DeviceStatus from '../components/device/DeviceStatus.svelte';

  function formatBytes(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  function formatBytesPrecise(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} GB`;
  }

  let isConnected = $derived(deviceStore.state.connected);
  let deviceName = $derived(deviceStore.state.name ?? 'Unknown Device');
  let productType = $derived(deviceStore.state.product_type ?? '—');
  let deviceMode = $derived(deviceStore.state.mode);
  let isNormalMode = $derived(deviceMode === 'Normal');

  // Identity
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
  let batteryCapacity = $derived(
    deviceStore.state.battery_current_capacity != null ? `${deviceStore.state.battery_current_capacity}%` : '—'
  );
  let totalDiskRaw = $derived(deviceStore.state.total_disk_capacity);
  let totalAvailableRaw = $derived(deviceStore.state.total_data_available);
  let capacityLabel = $derived(totalDiskRaw != null ? formatBytes(totalDiskRaw) : '—');
  let passwordProtected = $derived(
    deviceStore.state.password_protected != null
      ? (deviceStore.state.password_protected ? 'Yes' : 'No')
      : '—'
  );

  // Network
  let telephonyCapability = $derived(deviceStore.state.telephony_capability ?? false);
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

  // Usage bar
  let hasUsage = $derived(totalDiskRaw != null && totalAvailableRaw != null && totalDiskRaw > 0);
  let usedPercent = $derived(
    hasUsage ? Math.max(0, Math.min(100, ((totalDiskRaw! - totalAvailableRaw!) / totalDiskRaw!) * 100)) : 0
  );
  let availableLabel = $derived(totalAvailableRaw != null ? `${formatBytesPrecise(totalAvailableRaw)} Available` : '');

  // Recovery/DFU section visibility
  let hasNonces = $derived(deviceStore.state.apnonce != null || deviceStore.state.sepnonce != null);
</script>

<div class="home-view">
  {#if !isConnected}
    <div class="empty-state">
      <svg width="92" height="184" viewBox="0 0 120 200" fill="none" aria-hidden="true">
        <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.18" stroke-dasharray="6 4" />
        <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.04" />
        <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.12" />
        <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.1" />
      </svg>
      <h2 class="empty-title">No Device Connected</h2>
      <p class="empty-subtitle">Connect a USB device to get started</p>
    </div>
  {:else}
    <div class="info-pane">
      <header class="info-header">
        <div class="device-image">
          <svg width="80" height="166" viewBox="0 0 96 200" fill="none">
            <rect x="1" y="1" width="94" height="198" rx="15" fill="#1c2b4a" />
            <rect x="3" y="3" width="90" height="194" rx="13" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="1" />
            <rect x="7" y="30" width="82" height="140" rx="5" fill="#dbeafe" />
            <rect x="7" y="30" width="82" height="140" rx="5" fill="url(#sg)" />
            <defs>
              <linearGradient id="sg" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stop-color="white" stop-opacity="0.18" />
                <stop offset="100%" stop-color="white" stop-opacity="0" />
              </linearGradient>
            </defs>
            <circle cx="48" cy="185" r="8" fill="none" stroke="rgba(255,255,255,0.15)" stroke-width="1.5" />
            <circle cx="48" cy="185" r="4" fill="rgba(255,255,255,0.06)" />
            <circle cx="48" cy="18" r="3.5" fill="rgba(255,255,255,0.12)" />
            {#each [37, 41, 45, 49, 53, 57, 61] as x}
              <rect {x} y="16" width="2" height="4" rx="1" fill="rgba(255,255,255,0.1)" />
            {/each}
            <rect x="15" y="46" width="66" height="10" rx="3" fill="rgba(79,142,247,0.3)" />
            <rect x="15" y="62" width="46" height="5" rx="2" fill="rgba(79,142,247,0.15)" />
            <rect x="15" y="73" width="56" height="5" rx="2" fill="rgba(79,142,247,0.1)" />
            <rect x="15" y="84" width="38" height="5" rx="2" fill="rgba(79,142,247,0.08)" />
            {#each [15, 37, 59] as x}
              <rect {x} y="100" width="18" height="18" rx="4" fill="rgba(79,142,247,0.12)" />
            {/each}
          </svg>
        </div>

        <div class="header-text">
          <h1 class="device-name">{deviceName}</h1>
          <div class="device-product">{productType}</div>
          <div class="device-status-wrap">
            <DeviceStatus mode={deviceMode} />
          </div>
        </div>
      </header>

      {#if isNormalMode}
        <section class="info-section">
          <h2 class="section-title">About</h2>
          <dl class="info-rows">
            <div class="info-row"><dt>Model</dt><dd>{modelNumber !== '—' ? modelNumber : model}</dd></div>
            <div class="info-row"><dt>Capacity</dt><dd>{capacityLabel}</dd></div>
            <div class="info-row"><dt>State</dt><dd>{deviceMode}</dd></div>
            <div class="info-row"><dt>Battery</dt><dd>{batteryCapacity}</dd></div>
          </dl>
        </section>

        <section class="info-section">
          <h2 class="section-title">Tech Info</h2>
          <dl class="info-rows">
            <div class="info-row"><dt>iOS Version</dt><dd>{iosVersion} {buildVersion !== '—' ? `(${buildVersion})` : ''}</dd></div>
            <div class="info-row"><dt>Firmware</dt><dd class="mono truncate" title={firmwareVersion}>{firmwareVersion}</dd></div>
            <div class="info-row"><dt>Baseband</dt><dd>{basebandVersion}</dd></div>
            <div class="info-row"><dt>Serial Number</dt><dd class="mono">{serial}</dd></div>
            <div class="info-row"><dt>UDID</dt><dd class="mono truncate" title={udid}>{udid}</dd></div>
            <div class="info-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
            <div class="info-row"><dt>Wi-Fi Address</dt><dd class="mono">{wifiAddress}</dd></div>
            <div class="info-row"><dt>Bluetooth Address</dt><dd class="mono">{bluetoothAddress}</dd></div>
            {#if telephonyCapability}
              <div class="info-row"><dt>IMEI</dt><dd class="mono">{imei}</dd></div>
            {/if}
          </dl>
        </section>

        <section class="info-section">
          <h2 class="section-title">Hardware</h2>
          <dl class="info-rows">
            <div class="info-row"><dt>CPU Architecture</dt><dd>{cpuArchitecture}</dd></div>
            <div class="info-row"><dt>Platform</dt><dd>{hardwarePlatform}</dd></div>
            <div class="info-row"><dt>Region</dt><dd>{regionInfo}</dd></div>
            <div class="info-row"><dt>Color</dt><dd>{deviceColor}</dd></div>
            <div class="info-row"><dt>Activation</dt><dd>{activationState}</dd></div>
            <div class="info-row"><dt>Passcode</dt><dd>{passwordProtected}</dd></div>
          </dl>
        </section>

        {#if hasUsage}
          <section class="info-section">
            <h2 class="section-title">Usage</h2>
            <div class="usage-bar" role="img" aria-label={`${usedPercent.toFixed(0)}% used`}>
              <div class="usage-bar-fill" style:width={`${usedPercent}%`}></div>
            </div>
            <div class="usage-caption">{availableLabel}</div>
          </section>
        {/if}
      {:else}
        <section class="info-section">
          <h2 class="section-title">About</h2>
          <dl class="info-rows">
            <div class="info-row"><dt>Mode</dt><dd>{deviceMode}</dd></div>
            <div class="info-row"><dt>Product</dt><dd>{productType}</dd></div>
            <div class="info-row"><dt>Model</dt><dd>{model}</dd></div>
          </dl>
        </section>

        <section class="info-section">
          <h2 class="section-title">Tech Info</h2>
          <dl class="info-rows">
            <div class="info-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
            <div class="info-row"><dt>Serial</dt><dd class="mono">{serial}</dd></div>
            <div class="info-row"><dt>CPID</dt><dd class="mono">{cpid}</dd></div>
            <div class="info-row"><dt>CPRV</dt><dd class="mono">{cprv}</dd></div>
            <div class="info-row"><dt>Board ID</dt><dd class="mono">{bdid}</dd></div>
            <div class="info-row"><dt>iBoot Flags</dt><dd class="mono">{ibfl}</dd></div>
          </dl>
        </section>

        {#if hasNonces}
          <section class="info-section">
            <h2 class="section-title">Nonces</h2>
            <dl class="info-rows">
              <div class="info-row"><dt>APNonce</dt><dd class="mono truncate" title={apnonce}>{apnonce}</dd></div>
              <div class="info-row"><dt>SEPNonce</dt><dd class="mono truncate" title={sepnonce}>{sepnonce}</dd></div>
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
    height: 100%;
    min-height: 0;
    padding: var(--spacing-xl) var(--spacing-xl) var(--spacing-lg);
    overflow-y: auto;
  }

  .info-pane {
    width: 100%;
    max-width: 880px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xl);
  }

  /* ── Header ── */
  .info-header {
    display: flex;
    align-items: center;
    gap: 32px;
    padding-bottom: var(--spacing-md);
  }

  .device-image {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 96px;
    height: 166px;
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .device-name {
    font-size: 1.625rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.15;
    word-break: break-word;
    letter-spacing: -0.01em;
  }

  .device-product {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .device-status-wrap {
    margin-top: 4px;
  }

  /* ── Info sections ── */
  .info-section {
    display: flex;
    flex-direction: column;
  }

  .section-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
    letter-spacing: -0.01em;
    margin: 0 0 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--color-border);
  }

  /* ── Info rows: aligned label/value pairs ── */
  .info-rows {
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .info-row {
    display: grid;
    grid-template-columns: 36% 1fr;
    column-gap: var(--spacing-md);
    align-items: baseline;
    padding: 5px 0;
    min-width: 0;
  }

  .info-row dt {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    font-weight: 400;
    text-align: right;
  }

  .info-row dd {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--color-text-primary);
    font-weight: 500;
    text-align: left;
    min-width: 0;
  }

  .info-row dd.mono {
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.75rem;
    font-weight: 500;
    letter-spacing: -0.01em;
  }

  .info-row dd.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Usage bar ── */
  .usage-bar {
    width: 100%;
    height: 10px;
    border-radius: 5px;
    background: var(--color-bg-secondary);
    overflow: hidden;
    margin-top: 4px;
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-border) 60%, transparent);
  }

  .usage-bar-fill {
    height: 100%;
    background: linear-gradient(
      90deg,
      #ff9500 0%,
      #ffcc00 25%,
      #34c759 50%,
      #5ac8fa 75%,
      #af52de 100%
    );
    border-radius: 5px;
    transition: width 0.3s ease;
  }

  .usage-caption {
    margin-top: 8px;
    text-align: center;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  /* ── Empty state ── */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-md);
    color: var(--color-text-secondary);
    padding: var(--spacing-xl);
  }

  .empty-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .empty-subtitle {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* ── Responsive ── */
  @media (max-width: 700px) {
    .home-view {
      padding: var(--spacing-md);
    }

    .info-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-md);
    }

    .info-row {
      grid-template-columns: 40% 1fr;
    }
  }
</style>
