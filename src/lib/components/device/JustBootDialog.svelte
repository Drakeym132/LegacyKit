<script lang="ts">
  import { sendBootchain } from '../../api/jailbreak';
  import {
    listJustBootHistory,
    recordJustBoot,
    forgetJustBoot,
    prepareAndJustBoot,
    type JustBootEntry,
    type PrepareAndJustBootRequest
  } from '../../api/justBoot';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import { logStore } from '../../stores/logStore.svelte';
  import { toastStore } from '../../stores/toastStore.svelte';
  import { settingsStore } from '../../stores/settingsStore.svelte';
  import { inferProcessorGen } from '../../utils/processorGen';
  import { enterPwndfu } from '../../api/jailbreak';
  import PwnDfuHelper from './PwnDfuHelper.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  function portal(node: HTMLElement) {
    const original = node.parentNode;
    const next = node.nextSibling;
    document.body.appendChild(node);
    return {
      destroy() {
        if (original && node.parentNode === document.body) {
          original.insertBefore(node, next);
        }
      }
    };
  }

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open: isOpen, onClose }: Props = $props();

  let history = $state<JustBootEntry[]>([]);
  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);
  let isLoadingHistory = $state(false);

  // Form state for new build section
  let ipswPath = $state('');
  let buildId = $state('');
  let iosVersion = $state('');
  let bootArgs = $state('');

  // Device state
  let deviceState = $derived(deviceStore.state);
  let deviceEcid = $derived(deviceState.ecid);
  let deviceProductType = $derived(deviceState.product_type);
  let procGen = $derived(inferProcessorGen(deviceProductType));
  // A6 devices can boot via kDFU as well, so accept either pwnDFU or A6+kDFU.
  let isBootableMode = $derived(
    deviceState.mode === 'pwnDFU' || (procGen === 6 && deviceState.mode === 'kDFU')
  );

  // Hero entry: connected device's ECID, all four bootchain paths present.
  let heroEntry = $derived<JustBootEntry | null>(
    deviceEcid ? history.find(entry =>
      entry.ecid === deviceEcid &&
      entry.repackedIbssPath !== null &&
      entry.decryptedDevicetreePath !== null &&
      entry.decryptedKernelcachePath !== null
    ) ?? null : null
  );

  // Flat list of all other entries, newest-first.
  let allOtherEntries = $derived(
    history
      .filter(entry => entry !== heroEntry)
      .sort((a, b) => new Date(b.lastBootedAt).getTime() - new Date(a.lastBootedAt).getTime())
  );

  function formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  async function loadHistory() {
    if (!isOpen) return;

    isLoadingHistory = true;
    errorMessage = null;
    try {
      history = await listJustBootHistory();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = `Failed to load history: ${msg}`;
      toastStore.error('Load failed', errorMessage);
    } finally {
      isLoadingHistory = false;
    }
  }

  async function browseIpsw() {
    try {
      const selected = await openDialog({
        filters: [{ name: 'IPSW', extensions: ['ipsw'] }]
      });
      if (selected) {
        ipswPath = selected as string;
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      toastStore.error('Browse failed', msg);
    }
  }

  async function handleHeroBoot() {
    if (!heroEntry || !heroEntry.repackedIbssPath) return;
    if (!isBootableMode) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Booting iOS ${heroEntry.iosVersion ?? '?'} (${heroEntry.buildId})`;
    logStore.append(`${label}...`, 'info');

    try {
      await sendBootchain({
        ibssPath: heroEntry.repackedIbssPath,
        ibecPath: heroEntry.repackedIbecPath,
        deviceTreePath: heroEntry.decryptedDevicetreePath,
        kernelcachePath: heroEntry.decryptedKernelcachePath,
        processorGeneration: procGen
      });

      await recordJustBoot({
        ecid: heroEntry.ecid,
        productType: heroEntry.productType,
        deviceName: heroEntry.deviceName,
        buildId: heroEntry.buildId,
        iosVersion: heroEntry.iosVersion,
        bootArgs: heroEntry.bootArgs,
        repackedIbssPath: heroEntry.repackedIbssPath,
        repackedIbecPath: heroEntry.repackedIbecPath,
        decryptedDevicetreePath: heroEntry.decryptedDevicetreePath,
        decryptedKernelcachePath: heroEntry.decryptedKernelcachePath,
        sourceIpswPath: heroEntry.sourceIpswPath
      });

      logStore.append(`${label} ok`, 'info');
      toastStore.success('Boot successful', 'Device should now be booting');
      onClose();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      logStore.append(`${label} failed: ${msg}`, 'stderr');
      toastStore.error('Boot failed', msg);
    } finally {
      isWorking = false;
    }
  }

  /**
   * Re-prepares a history entry using its saved sourceIpswPath, then boots.
   * Called when a history entry is missing DT/KC paths (old cache entries).
   */
  async function handleReprepAndBoot(entry: JustBootEntry) {
    if (!entry.sourceIpswPath) {
      errorMessage = 'No source IPSW path available for re-preparation. Select an IPSW manually.';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Re-preparing and booting iOS ${entry.iosVersion ?? '?'} (${entry.buildId})`;
    logStore.append(`${label}...`, 'info');

    try {
      await prepareAndJustBoot({
        ecid: entry.ecid,
        productType: entry.productType,
        deviceName: entry.deviceName,
        buildId: entry.buildId,
        iosVersion: entry.iosVersion,
        ipswPath: entry.sourceIpswPath,
        bootArgs: entry.bootArgs
      });

      await loadHistory();
      logStore.append(`${label} ok`, 'info');
      toastStore.success('Boot successful', 'Device should now be booting');
      onClose();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      logStore.append(`${label} failed: ${msg}`, 'stderr');
      toastStore.error('Boot failed', msg);
    } finally {
      isWorking = false;
    }
  }

  async function handleHistoryBoot(entry: JustBootEntry) {
    if (!isBootableMode) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    // Guard: if any required bootchain path is missing, re-prep transparently.
    if (!entry.repackedIbssPath || !entry.decryptedDevicetreePath || !entry.decryptedKernelcachePath) {
      await handleReprepAndBoot(entry);
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Booting iOS ${entry.iosVersion ?? '?'} (${entry.buildId})`;
    logStore.append(`${label}...`, 'info');

    try {
      await sendBootchain({
        ibssPath: entry.repackedIbssPath,
        ibecPath: entry.repackedIbecPath,
        deviceTreePath: entry.decryptedDevicetreePath,
        kernelcachePath: entry.decryptedKernelcachePath,
        processorGeneration: procGen
      });

      await recordJustBoot({
        ecid: entry.ecid,
        productType: entry.productType,
        deviceName: entry.deviceName,
        buildId: entry.buildId,
        iosVersion: entry.iosVersion,
        bootArgs: entry.bootArgs,
        repackedIbssPath: entry.repackedIbssPath,
        repackedIbecPath: entry.repackedIbecPath,
        decryptedDevicetreePath: entry.decryptedDevicetreePath,
        decryptedKernelcachePath: entry.decryptedKernelcachePath,
        sourceIpswPath: entry.sourceIpswPath
      });

      logStore.append(`${label} ok`, 'info');
      toastStore.success('Boot successful', 'Device should now be booting');
      onClose();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      logStore.append(`${label} failed: ${msg}`, 'stderr');
      toastStore.error('Boot failed', msg);
    } finally {
      isWorking = false;
    }
  }

  async function handlePrepareAndBoot() {
    if (!ipswPath.trim() || !buildId.trim()) {
      errorMessage = 'IPSW path and Build ID are required.';
      return;
    }
    if (!isBootableMode) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Preparing and booting ${buildId}`;
    logStore.append(`${label}...`, 'info');

    try {
      const request: PrepareAndJustBootRequest = {
        ecid: deviceEcid || '',
        productType: deviceProductType || '',
        deviceName: deviceState.name,
        buildId: buildId.trim(),
        iosVersion: iosVersion.trim() || null,
        ipswPath: ipswPath.trim(),
        bootArgs: bootArgs.trim() || null
      };

      await prepareAndJustBoot(request);

      logStore.append(`${label} ok`, 'info');
      toastStore.success('Boot successful', 'Device should now be booting');
      onClose();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      logStore.append(`${label} failed: ${msg}`, 'stderr');
      toastStore.error('Boot failed', msg);
    } finally {
      isWorking = false;
    }
  }

  async function handleForget(entry: JustBootEntry) {
    try {
      await forgetJustBoot(entry.id);
      toastStore.success('Entry forgotten', 'Boot history entry removed');
      await loadHistory();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      toastStore.error('Forget failed', msg);
    }
  }

  function handleClose() {
    if (isWorking) return;
    errorMessage = null;
    onClose();
  }

  function onKey(e: KeyboardEvent) {
    if (!isOpen) return;
    if (e.key === 'Escape') handleClose();
  }

  $effect(() => {
    if (isOpen) {
      loadHistory();
      void maybeAutoEnterPwndfu();
      const prev = document.body.style.overflow;
      document.body.style.overflow = 'hidden';
      return () => {
        document.body.style.overflow = prev;
      };
    }
  });

  let autoPwnAttempted = false;
  async function maybeAutoEnterPwndfu() {
    if (!settingsStore.autoEnterPwnDfu || autoPwnAttempted) return;
    if (!deviceState.connected || deviceState.mode !== 'DFU') return;
    if (!deviceProductType) return;
    autoPwnAttempted = true;
    isWorking = true;
    try {
      const result = await enterPwndfu({ productType: deviceProductType });
      deviceStore.optimisticallySetMode(result.mode, result.pwnd);
      toastStore.success('pwnDFU entered', `via ${result.tool}`);
      // Boost polling so the next detect_device runs immediately with fresh
      // pwnDFU state instead of potentially stale DFU from before the exploit.
      settingsStore.boostPolling();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      isWorking = false;
    }
  }
  $effect(() => {
    if (!isOpen) {
      autoPwnAttempted = false;
    }
  });
</script>

<svelte:window onkeydown={onKey} />

{#if isOpen}
  <div
    class="overlay"
    role="presentation"
    onclick={handleClose}
    use:portal
    transition:fade={{ duration: 150 }}
  >
    <div
      class="dialog"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="just-boot-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      transition:scale={{ duration: 180, start: 0.96, easing: cubicOut }}
    >
      <header class="dialog-header">
        <h3 id="just-boot-title">Just Boot</h3>
      </header>

      <div class="dialog-body">

      <!-- Device Summary -->
      {#if deviceState.connected}
        <div class="device-summary">
          <span class="device-name">{deviceState.name || deviceState.product_type || 'Unknown Device'}</span>
          <span class="device-info">{deviceState.product_type} • {deviceState.mode}</span>
          <span class="device-ecid">ECID: {deviceState.ecid?.slice(-8) || 'N/A'}</span>
        </div>
      {:else}
        <div class="device-summary">
          <span class="device-name">No Device Connected</span>
        </div>
      {/if}

      <!-- pwnDFU helper / warning -->
      {#if !isBootableMode}
        <PwnDfuHelper />
      {/if}

      <!-- Hero Card - Boot last cached build for connected device -->
      {#if heroEntry}
        <div class="hero-card">
          <div class="hero-header">
            <h4>Boot last build</h4>
            <span class="last-booted">Last booted {formatRelativeTime(heroEntry.lastBootedAt)}</span>
          </div>
          <div class="hero-content">
            <div class="build-info">
              <span class="ios-version">iOS {heroEntry.iosVersion ?? '?'} ({heroEntry.buildId})</span>
            </div>
            <div class="hero-actions">
              <button
                class="primary large"
                onclick={handleHeroBoot}
                disabled={isWorking || !isBootableMode}
              >
                {isWorking ? 'Booting…' : 'Boot'}
              </button>
              <button
                class="secondary small"
                onclick={() => handleForget(heroEntry)}
                disabled={isWorking}
              >
                Forget
              </button>
            </div>
          </div>
        </div>
      {/if}

      <!-- Boot History - flat list, sorted newest-first -->
      <div class="history-section">
        <h4>Boot history</h4>

        {#if isLoadingHistory}
          <div class="loading">Loading history...</div>
        {:else if allOtherEntries.length === 0 && !heroEntry}
          <div class="empty-state">No boot history found</div>
        {:else}
          {#each allOtherEntries as entry}
            <div class="history-item">
              <div class="item-info">
                <span class="device-name">{entry.deviceName || entry.productType}</span>
                <span class="build-info">iOS {entry.iosVersion ?? '?'} ({entry.buildId})</span>
                <span class="last-booted">{formatRelativeTime(entry.lastBootedAt)}</span>
              </div>
              <div class="item-actions">
                <button
                  class="primary small"
                  onclick={() => handleHistoryBoot(entry)}
                  disabled={isWorking || !isBootableMode}
                >
                  Boot
                </button>
                <button
                  class="secondary small"
                  onclick={() => handleForget(entry)}
                  disabled={isWorking}
                >
                  Forget
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Boot a different build -->
      <details class="new-build-section" open={!heroEntry}>
        <summary>
          <h4>Boot a different build</h4>
        </summary>

        <div class="form-section">
          <label class="field">
            <span>IPSW path</span>
            <div class="input-group">
              <input bind:value={ipswPath} placeholder="/path/to/firmware.ipsw" disabled={isWorking} />
              <button class="secondary" onclick={browseIpsw} disabled={isWorking}>Browse…</button>
            </div>
          </label>

          <label class="field">
            <span>Build ID</span>
            <input bind:value={buildId} placeholder="e.g. 13G36" disabled={isWorking} />
          </label>

          <label class="field">
            <span>iOS version (optional)</span>
            <input bind:value={iosVersion} placeholder="e.g. 9.3.5" disabled={isWorking} />
          </label>

          <label class="field">
            <span>Custom boot-args</span>
            <input bind:value={bootArgs} placeholder="pio-error=0 -v" disabled={isWorking} />
          </label>
          <p class="advanced-note">
            Repacked bootchain files are resolved and cached automatically under the configured workspace.
          </p>

          {#if errorMessage}
            <div class="error">{errorMessage}</div>
          {/if}

          <div class="actions">
            <button
              class="primary"
              onclick={handlePrepareAndBoot}
              disabled={isWorking || !isBootableMode || !ipswPath.trim() || !buildId.trim()}
            >
              {isWorking ? 'Working…' : 'Prepare & Boot'}
            </button>
          </div>
        </div>
      </details>

      </div>

      <footer class="dialog-footer">
        <button class="secondary" onclick={handleClose} disabled={isWorking}>Cancel</button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: grid;
    place-items: center;
    z-index: 999;
    padding: clamp(8px, 2vh, 16px);
  }

  .dialog {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    width: min(600px, 100%);
    max-height: min(calc(100vh - 32px), 720px);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
  }

  .dialog-header {
    padding: clamp(12px, 2.5vh, 24px) clamp(12px, 2.5vh, 24px) 0;
  }

  .dialog-body {
    padding: clamp(8px, 1.5vh, 16px) clamp(12px, 2.5vh, 24px);
    overflow-y: auto;
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: clamp(8px, 1.5vh, 16px);
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) clamp(12px, 2.5vh, 24px);
    border-top: 1px solid var(--color-border);
  }

  @media (prefers-reduced-motion: reduce) {
    .overlay,
    .dialog {
      transition: none !important;
      animation: none !important;
    }
  }

  h3 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  h4 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .device-summary {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--spacing-sm);
    background: var(--color-bg-secondary);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
  }

  .device-name {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .device-info {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .device-ecid {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
  }

  .hero-card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .hero-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .last-booted {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .hero-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
  }

  .build-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .ios-version {
    font-size: 1.125rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .hero-actions {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
  }

  .history-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .history-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-sm);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .device-name {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .build-info {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .last-booted {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .item-actions {
    display: flex;
    gap: var(--spacing-xs);
  }

  .new-build-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .new-build-section summary {
    cursor: pointer;
    padding: var(--spacing-xs) 0;
  }

  .new-build-section summary::-webkit-details-marker {
    display: none;
  }

  .form-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding-left: var(--spacing-sm);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.8125rem;
  }

  .field span {
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .input-group {
    display: flex;
    gap: var(--spacing-xs);
  }

  .input-group input {
    flex: 1;
  }

  .field input {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    padding: 6px 10px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.8125rem;
  }

  .field input:disabled {
    opacity: 0.6;
  }

  .advanced-note {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    line-height: 1.45;
  }

  .error {
    color: var(--color-danger);
    font-size: 0.8125rem;
    background: color-mix(in srgb, var(--color-danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-xs);
  }

  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
    border: 1px solid transparent;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }

  .primary {
    background: var(--color-accent);
    border: 1px solid var(--color-accent);
    color: white;
  }

  .small {
    padding: 4px 8px;
    font-size: 0.75rem;
  }

  .large {
    padding: 10px 18px;
    font-size: 0.875rem;
  }

  .loading {
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    padding: var(--spacing-md);
  }

  .empty-state {
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    padding: var(--spacing-md);
  }
</style>
