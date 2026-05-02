<script lang="ts">
  import { navigationStore, type ViewName } from '../stores/navigationStore.svelte';
  import { deviceStore } from '../stores/deviceStore.svelte';
  import JustBootDialog from '../components/device/JustBootDialog.svelte';
  import DeviceImage from '../components/device/DeviceImage.svelte';
  import { getDeviceFriendlyName } from '../utils/deviceModels';

  let showJustBoot = $state(false);

  type QuickAction =
    | { kind: 'nav'; label: string; view: ViewName; icon: string; disabled?: boolean }
    | { kind: 'just-boot'; label: string; icon: string; disabled?: boolean };

  const quickActions = $derived<QuickAction[]>([
    {
      kind: 'nav',
      label: 'Restore',
      view: 'restore',
      icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'Jailbreak',
      view: 'jailbreak',
      icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'SHSH Blobs',
      view: 'shsh',
      icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'SSH Ramdisk',
      view: 'ssh-ramdisk',
      icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
    },
    {
      kind: 'just-boot',
      label: 'Just Boot',
      icon: `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><polygon points="10,8 16,12 10,16" fill="currentColor" stroke="none"/></svg>`,
    },
  ]);

  function handleAction(action: QuickAction) {
    if (action.disabled) return;
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

  // Derived state
  let isConnected = $derived(deviceStore.state.connected);
  let isNormalMode = $derived(deviceStore.state.mode === 'Normal');
  let deviceName = $derived(
    deviceStore.state.name
      || getDeviceFriendlyName(deviceStore.state.product_type)
      || deviceStore.state.product_type
      || 'Legacy Device'
  );
  let modelNumber = $derived(deviceStore.state.model || '—');
  let capacityLabel = $derived(
    deviceStore.state.total_disk_capacity
      ? formatBytes(deviceStore.state.total_disk_capacity)
      : '—'
  );
  let deviceMode = $derived(deviceStore.state.mode || '—');
  let batteryCapacity = $derived(
    deviceStore.state.battery_current_capacity !== null
      ? `${deviceStore.state.battery_current_capacity}%`
      : '—'
  );
  let iosVersionLabel = $derived.by(() => {
    const v = deviceStore.state.ios_version;
    const b = deviceStore.state.build_version;
    if (v && b) return `${v} (${b})`;
    return v || '—';
  });
  let firmwareLabel = $derived(deviceStore.state.firmware_version || '—');
  let basebandLabel = $derived(deviceStore.state.baseband_version || '—');
  let serialLabel = $derived(deviceStore.state.serial || '—');
  let ecidLabel = $derived(deviceStore.state.ecid || '—');
  let udidLabel = $derived(deviceStore.state.udid || '—');
  let wifiLabel = $derived(deviceStore.state.wifi_address || '—');
  let bluetoothLabel = $derived(deviceStore.state.bluetooth_address || '—');

</script>

<div class="view" class:view--empty={!isConnected}>
  {#if isConnected}
    <header class="hero">
      <div class="device-stage">
        <DeviceImage
          productType={deviceStore.state.product_type}
          deviceColor={deviceStore.state.device_color}
          width={150}
          height={300}
        />
      </div>
      <h1 class="device-name">{deviceName}</h1>
      <div class="status-pill" data-mode={deviceMode}>
        <span class="status-dot" aria-hidden="true"></span>
        <span>Connected · {deviceMode}</span>
      </div>
    </header>

    <nav class="quick-actions" aria-label="Quick actions">
      {#each quickActions as action}
        <button
          class="quick-action"
          onclick={() => handleAction(action)}
          aria-label={action.label}
          title={action.label}
          disabled={action.disabled}
        >
          <span class="action-icon">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html action.icon}
          </span>
          <span class="action-label">{action.label}</span>
        </button>
      {/each}
    </nav>

    <section class="info-section" aria-labelledby="about-title">
      <h2 id="about-title" class="section-title">About</h2>
      <dl class="info-rows">
        <div class="info-row"><dt>Model</dt><dd>{modelNumber}</dd></div>
        <div class="info-row"><dt>Capacity</dt><dd>{capacityLabel}</dd></div>
        <div class="info-row"><dt>State</dt><dd>{deviceMode}</dd></div>
        {#if isNormalMode}
          <div class="info-row"><dt>Battery</dt><dd>{batteryCapacity}</dd></div>
        {/if}
      </dl>
    </section>

    <section class="info-section" aria-labelledby="tech-title">
      <h2 id="tech-title" class="section-title">Tech Info</h2>
      <dl class="info-rows">
        {#if isNormalMode}
          <div class="info-row"><dt>iOS Version</dt><dd>{iosVersionLabel}</dd></div>
        {/if}
        <div class="info-row"><dt>Firmware</dt><dd class="mono">{firmwareLabel}</dd></div>
        {#if isNormalMode}
          <div class="info-row"><dt>Baseband</dt><dd class="mono">{basebandLabel}</dd></div>
          <div class="info-row"><dt>Serial</dt><dd class="mono">{serialLabel}</dd></div>
        {/if}
        <div class="info-row"><dt>ECID</dt><dd class="mono">{ecidLabel}</dd></div>
        {#if isNormalMode}
          <div class="info-row"><dt>UDID</dt><dd class="mono">{udidLabel}</dd></div>
          <div class="info-row"><dt>Wi-Fi Address</dt><dd class="mono">{wifiLabel}</dd></div>
          <div class="info-row"><dt>Bluetooth</dt><dd class="mono">{bluetoothLabel}</dd></div>
        {/if}
      </dl>
    </section>
  {:else}
    <div class="empty-state" role="status" aria-live="polite">
      <div class="empty-illustration" aria-hidden="true">
        <svg width="120" height="120" viewBox="0 0 120 120" fill="none">
          <rect x="36" y="14" width="48" height="92" rx="9"
            fill="var(--color-bg-secondary)"
            stroke="var(--color-border)" stroke-width="1.5"/>
          <rect x="40" y="22" width="40" height="70" rx="3"
            fill="var(--color-bg-primary)"
            stroke="var(--color-border)" stroke-width="1"/>
          <circle cx="60" cy="100" r="3" fill="none"
            stroke="var(--color-border)" stroke-width="1.2"/>
          <line x1="22" y1="22" x2="98" y2="98"
            stroke="var(--color-text-secondary)" stroke-width="2.5"
            stroke-linecap="round" opacity="0.55"/>
        </svg>
      </div>
      <h1 class="empty-title">No Device Connected</h1>
      <p class="empty-subtitle">
        Connect your iOS device via USB to get started. LegacyKit will detect it
        automatically once unlocked and trusted, or connected in DFU/Recovery
        Mode.
      </p>
    </div>
  {/if}
</div>

<JustBootDialog open={showJustBoot} onClose={() => (showJustBoot = false)} />

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-lg) var(--spacing-md);
    max-width: 720px;
    margin: 0 auto;
    width: 100%;
  }

  .view--empty {
    flex: 1;
    min-height: 100%;
    justify-content: center;
    align-items: center;
    padding: var(--spacing-xl) var(--spacing-md);
    gap: 0;
  }

  /* ---------- Hero ---------- */
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--spacing-sm);
    padding-top: var(--spacing-md);
  }

  .device-stage {
    height: 280px;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    margin-bottom: var(--spacing-xs);
  }

  .device-name {
    margin: 0;
    font-size: 1.75rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 5px 12px 5px 10px;
    background: var(--color-bg-secondary);
    border-radius: 999px;
    font-size: 0.8125rem;
    color: var(--color-text-primary);
    font-weight: 500;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-success);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-success) 25%, transparent);
  }

  .status-pill[data-mode="Recovery"] .status-dot,
  .status-pill[data-mode="DFU"] .status-dot,
  .status-pill[data-mode="kDFU"] .status-dot,
  .status-pill[data-mode="pwnDFU"] .status-dot {
    background: var(--color-warning);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-warning) 25%, transparent);
  }

  /* ---------- Quick Actions ---------- */
  .quick-actions {
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
    padding: 0 var(--spacing-sm);
  }

  .quick-action {
    flex: 0 1 96px;
    min-width: 88px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 12px 8px 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease, transform 0.15s ease;
    text-align: center;
    color: var(--color-text-primary);
  }

  .quick-action:hover:not(:disabled) {
    background: var(--color-bg-secondary);
    border-color: color-mix(in srgb, var(--color-accent) 40%, var(--color-border));
  }

  .quick-action:active:not(:disabled) {
    transform: translateY(1px);
  }

  .quick-action:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .action-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .action-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    line-height: 1.2;
  }

  /* ---------- Info Sections (Configurator-style list) ---------- */
  .info-section {
    padding: 0 var(--spacing-sm);
  }

  .section-title {
    margin: 0 0 var(--spacing-xs);
    font-size: 1rem;
    font-weight: 600;
    padding-bottom: var(--spacing-xs);
    border-bottom: 1px solid var(--color-border);
  }

  .info-rows {
    margin: 0;
    display: flex;
    flex-direction: column;
  }

  .info-row {
    display: grid;
    grid-template-columns: minmax(140px, 0.6fr) 1fr;
    align-items: baseline;
    gap: var(--spacing-md);
    padding: 6px 0;
    font-size: 0.8125rem;
    border-bottom: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .info-row dt {
    color: var(--color-text-secondary);
    text-align: right;
    font-weight: 400;
  }

  .info-row dd {
    margin: 0;
    color: var(--color-text-primary);
    font-weight: 500;
    word-break: break-all;
  }

  .info-row dd.mono {
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.78125rem;
  }

  /* ---------- Empty State ---------- */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--spacing-md);
    max-width: 440px;
    width: 100%;
  }

  .empty-illustration {
    opacity: 0.85;
    margin-bottom: var(--spacing-xs);
  }

  .empty-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .empty-subtitle {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.9375rem;
    max-width: 440px;
    line-height: 1.55;
    text-wrap: balance;
  }
</style>
