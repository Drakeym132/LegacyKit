<script lang="ts">
  import { onMount } from 'svelte';
  import { navigationStore } from '../stores/navigationStore.svelte';
  import { deviceStore } from '../stores/deviceStore.svelte';
  import { settingsStore } from '../stores/settingsStore.svelte';
  import { listJustBootHistory, type JustBootEntry } from '../api/justBoot';
  import JustBootDialog from '../components/device/JustBootDialog.svelte';
  import DeviceImage from '../components/device/DeviceImage.svelte';
  import { getDeviceFriendlyName } from '../utils/deviceModels';

  let showJustBoot = $state(false);
  let justBootHistory = $state<JustBootEntry[]>([]);
  let isLoadingHistory = $state(false);

  type QuickAction =
    | { kind: 'nav'; label: string; subtitle: string; view: ViewName; icon: string }
    | { kind: 'just-boot'; label: string; subtitle: string; icon: string };

  const quickActions = $derived<QuickAction[]>([
    {
      kind: 'nav',
      label: 'Jailbreak',
      subtitle: 'Pwn, patch, and untether',
      view: 'jailbreak',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/><path d="m9 12 2 2 4-4"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'Restore',
      subtitle: 'Downgrade and restore',
      view: 'restore',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>`,
    },
    {
      kind: 'nav',
      label: 'SSH Ramdisk',
      subtitle: 'Custom ramdisk with SSH',
      view: 'ssh-ramdisk',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14,2 14,8 20,8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="10" y2="9"/></svg>`,
    },
    {
      kind: 'just-boot',
      label: 'Just Boot',
      subtitle: getJustBootSubtitle(),
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><polygon points="10,8 16,12 10,16" fill="currentColor" stroke="none"/></svg>`,
    },
  ]);

  function getJustBootSubtitle(): string {
    if (!deviceStore.state.connected || !deviceStore.state.ecid) {
      return 'Boot from a previous install';
    }
    
    if (isLoadingHistory) {
      return 'Loading history...';
    }
    
    const entry = justBootHistory.find(h => h.ecid === deviceStore.state.ecid);
    if (entry) {
      return `Boot iOS ${entry.iosVersion ?? '?'} (${entry.buildId})`;
    }
    
    return 'Boot from a previous install';
  }

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

  // Derived state
  let isNormalMode = $derived(deviceStore.state.mode === 'Normal');
  let modelNumber = $derived(deviceStore.state.hardware_model || '—');
  let model = $derived(getDeviceFriendlyName(deviceStore.state.product_type) || deviceStore.state.product_type || 'Unknown');
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

  async function loadJustBootHistory() {
    if (!deviceStore.state.connected) return;
    
    isLoadingHistory = true;
    try {
      justBootHistory = await listJustBootHistory();
    } catch (error) {
      console.warn('Failed to load Just Boot history:', error);
      justBootHistory = [];
    } finally {
      isLoadingHistory = false;
    }
  }

  // Load history when device connects or on mount
  onMount(() => {
    loadJustBootHistory();
  });

  $effect(() => {
    if (deviceStore.state.connected && deviceStore.state.ecid) {
      loadJustBootHistory();
    }
  });
</script>

<div class="view">
  <div class="view-header">
    <DeviceImage productType={deviceStore.state.product_type} />
    <div class="device-info">
      <h1>{deviceStore.state.name || getDeviceFriendlyName(deviceStore.state.product_type) || 'Legacy Device'}</h1>
      <p>{deviceStore.state.ios_version || 'Unknown iOS'} • {deviceStore.state.mode || 'Unknown Mode'}</p>
    </div>
  </div>

  <div class="content-grid">
    <section class="quick-actions-section">
      <header>
        <div>
          <h2>Quick Actions</h2>
          <p>Common tasks at a glance</p>
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
            {#if action.subtitle}
              <span class="action-subtitle">{action.subtitle}</span>
            {/if}
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
      {/if}

      <section class="info-section">
        <h2 class="section-title">Utilities</h2>
        <dl class="info-rows">
          <div class="info-row">
            <dt>LegacyKit</dt>
            <dd>{settingsStore.state.appVersion}</dd>
          </div>
          <div class="info-row">
            <dt>Platform</dt>
            <dd>{settingsStore.state.platform}</dd>
          </div>
        </dl>
      </section>
    </section>
  </div>
</div>

<JustBootDialog open={showJustBoot} onClose={() => (showJustBoot = false)} />

<style>
  .view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-md);
    max-width: 800px;
  }

  .view-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .device-info h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .device-info p {
    margin: 0.25rem 0 0;
    color: var(--color-text-secondary);
    font-size: 0.875rem;
  }

  .content-grid {
    display: grid;
    gap: var(--spacing-lg);
  }

  .quick-actions-section header {
    margin-bottom: var(--spacing-sm);
  }

  .quick-actions-section h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .quick-actions-section p {
    margin: 0.25rem 0 0;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
  }

  .quick-actions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--spacing-sm);
  }

  .quick-action {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-xs);
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: center;
  }

  .quick-action:hover {
    background: var(--color-bg-secondary);
    border-color: var(--color-accent);
    transform: translateY(-1px);
  }

  .quick-action:active {
    transform: translateY(0);
  }

  .action-icon {
    width: 24px;
    height: 24px;
    color: var(--color-accent);
  }

  .action-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .action-subtitle {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    margin-top: -4px;
  }

  .info-section {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
  }

  .section-title {
    margin: 0 0 var(--spacing-sm);
    font-size: 1rem;
    font-weight: 600;
  }

  .info-rows {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.8125rem;
  }

  .info-row dt {
    color: var(--color-text-secondary);
  }

  .info-row dd {
    margin: 0;
    font-weight: 500;
    color: var(--color-text-primary);
  }
</style>