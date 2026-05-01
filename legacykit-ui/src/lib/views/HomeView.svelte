<script lang="ts">
  import { deviceStore } from '../stores/deviceStore.svelte';
  import { navigationStore } from '../stores/navigationStore.svelte';
  import type { ViewName } from '../stores/navigationStore.svelte';
  import DeviceStatus from '../components/device/DeviceStatus.svelte';
  import DeviceImage from '../components/device/DeviceImage.svelte';
  import JustBootDialog from '../components/device/JustBootDialog.svelte';
  import { getDeviceFriendlyName } from '../utils/deviceModels';

  let showJustBoot = $state(false);

  type QuickAction =
    | { kind: 'nav'; label: string; view: ViewName; icon: string }
    | { kind: 'just-boot'; label: string; icon: string };

  const quickActions: QuickAction[] = [
    {
      kind: 'nav',
      label: 'Restore',
      view: 'restore',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'Jailbreak',
      view: 'jailbreak',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/><circle cx="12" cy="16" r="1" fill="currentColor"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'SHSH Blobs',
      view: 'shsh',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17,21 17,13 7,13 7,21"/><polyline points="7,3 7,8 15,8"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'SSH Ramdisk',
      view: 'ssh-ramdisk',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
    },
    {
      kind: 'just-boot',
      label: 'Just Boot',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><polygon points="10,8 16,12 10,16" fill="currentColor" stroke="none"/></svg>`,
    },
  ];

  function handleAction(action: QuickAction) {
    if (action.kind === 'nav') {
      navigationStore.navigate(action.view);
    } else {
      showJustBoot = true;
    }
  }

  function formatBytes(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  function formatBytesPrecise(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} GB`;
  }

  let isConnected = $derived(deviceStore.state.connected);
  let deviceName = $derived(
    deviceStore.state.name
      ?? getDeviceFriendlyName(deviceStore.state.product_type)
      ?? deviceStore.state.product_type
      ?? 'Unknown Device'
  );
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
          <DeviceImage
            productType={deviceStore.state.product_type}
            deviceColor={deviceStore.state.device_color}
          />
        </div>

        <div class="header-text">
          <h1 class="device-name">{deviceName}</h1>
          <div class="device-status-wrap">
            <DeviceStatus mode={deviceMode} connected={isConnected} />
          </div>
        </div>
      </header>

      <nav class="quick-actions" aria-label="Quick actions">
        {#each quickActions as action}
          <button
            class="quick-action"
            onclick={() => handleAction(action)}
            aria-label={action.label}
            title={action.label}
          >
            <span class="action-icon">
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html action.icon}
            </span>
            <span class="action-label">{action.label}</span>
          </button>
        {/each}
      </nav>

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

<JustBootDialog open={showJustBoot} onClose={() => (showJustBoot = false)} />

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
    justify-content: center;
    gap: 32px;
    padding-bottom: var(--spacing-md);
  }

  .device-image {
    position: relative;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 96px;
    height: 168px;
    margin-bottom: 24px;
    overflow: visible;
  }

  .header-text {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .device-name {
    font-size: 1.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.15;
    word-break: break-word;
    letter-spacing: -0.01em;
  }

  .device-status-wrap {
    margin-top: 6px;
  }

  .device-status-wrap :global(.status-indicator) {
    padding: 4px 11px;
    border-radius: 12px;
    gap: 7px;
  }

  .device-status-wrap :global(.dot) {
    width: 7px;
    height: 7px;
  }

  .device-status-wrap :global(.mode-text) {
    font-size: 13px;
  }

  /* ── Quick actions ── */
  .quick-actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    padding: 4px 0 var(--spacing-sm);
  }

  .quick-action {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    min-width: 76px;
    height: 56px;
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    cursor: pointer;
    font-family: inherit;
    color: var(--color-text-primary);
    transition: background-color 0.1s ease, border-color 0.1s ease, opacity 0.1s ease;
  }

  .quick-action:hover {
    background: color-mix(in srgb, var(--color-text-primary) 5%, transparent);
  }

  .quick-action:active {
    background: var(--color-bg-secondary);
  }

  .action-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .action-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-secondary);
    letter-spacing: 0.01em;
    line-height: 1;
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
