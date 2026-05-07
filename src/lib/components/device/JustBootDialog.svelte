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
  import IpswSourcePicker, { type SelectedIpsw } from './IpswSourcePicker.svelte';
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

  let selectedIpsw = $state<SelectedIpsw | null>(null);
  let manualIosVersion = $state<string | null>(null);
  let bootArgs = $state('');

  let deviceState = $derived(deviceStore.state);
  let deviceEcid = $derived(deviceState.ecid);
  let deviceProductType = $derived(deviceState.product_type);
  let procGen = $derived(inferProcessorGen(deviceProductType));
  let isBootableMode = $derived(
    deviceState.mode === 'pwnDFU' || (procGen === 6 && deviceState.mode === 'kDFU')
  );

  let recentEntries = $derived(
    history
      .slice()
      .sort((a, b) => new Date(b.lastBootedAt).getTime() - new Date(a.lastBootedAt).getTime())
  );
  let lastBootedId = $derived(
    deviceEcid
      ? recentEntries.find((entry) => entry.ecid === deviceEcid)?.id ?? null
      : null
  );

  let effectiveIosVersion = $derived(
    selectedIpsw?.iosVersion ?? (manualIosVersion && manualIosVersion.trim() ? manualIosVersion.trim() : null)
  );
  let canBootDifferent = $derived(
    !!selectedIpsw && (selectedIpsw.metadataResolved || !!effectiveIosVersion)
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

  async function handleHistoryBoot(entry: JustBootEntry) {
    if (!isBootableMode) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

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

  async function handleReprepAndBoot(entry: JustBootEntry) {
    if (!entry.sourceIpswPath) {
      errorMessage = 'No source IPSW available to re-prepare. Pick one below to boot this build.';
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

  async function handleBootSelected() {
    if (!selectedIpsw) return;
    if (!isBootableMode) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }
    if (!selectedIpsw.metadataResolved && !effectiveIosVersion) {
      errorMessage = 'Enter an iOS version for this IPSW.';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const versionLabel = effectiveIosVersion ?? '?';
    const label = `Preparing and booting iOS ${versionLabel} (${selectedIpsw.buildId})`;
    logStore.append(`${label}...`, 'info');

    try {
      const request: PrepareAndJustBootRequest = {
        ecid: deviceEcid || '',
        productType: deviceProductType || '',
        deviceName: deviceState.name,
        buildId: selectedIpsw.buildId,
        iosVersion: effectiveIosVersion,
        ipswPath: selectedIpsw.path,
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

  $effect(() => {
    if (!isOpen) {
      selectedIpsw = null;
      manualIosVersion = null;
      bootArgs = '';
      autoPwnAttempted = false;
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
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      isWorking = false;
    }
  }

  function handleIpswSelected(ipsw: SelectedIpsw) {
    selectedIpsw = ipsw;
    manualIosVersion = null;
    errorMessage = null;
  }

  function handleIpswCleared() {
    selectedIpsw = null;
    manualIosVersion = null;
  }

  function handleManualVersion(version: string) {
    manualIosVersion = version;
  }
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
        <button class="close" type="button" aria-label="Close" onclick={handleClose} disabled={isWorking}>
          <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
            <path d="M3 3 L13 13 M13 3 L3 13" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </button>
      </header>

      <div class="dialog-body">
        <!-- pwnDFU step strip -->
        <section class="section">
          <h4 class="section-title">Get into pwnDFU</h4>
          <PwnDfuHelper />
        </section>

        <!-- Select or Download Build (includes Recent tab) -->
        <section class="section">
          <h4 class="section-title">Select or Download Build</h4>
          <IpswSourcePicker
            deviceIdentifier={deviceProductType}
            selected={selectedIpsw}
            isWorking={isWorking}
            onSelect={handleIpswSelected}
            onClear={handleIpswCleared}
            onManualVersion={handleManualVersion}
            recentEntries={recentEntries}
            lastBootedId={lastBootedId}
            isLoadingHistory={isLoadingHistory}
            isBootableMode={isBootableMode}
            onBootRecent={(entry) => { const full = history.find(h => h.id === entry.id); if (full) void handleHistoryBoot(full); }}
            onForgetRecent={(entry) => { const full = history.find(h => h.id === entry.id); if (full) void handleForget(full); }}
          />

          <details class="advanced">
            <summary>Advanced</summary>
            <label class="field">
              <span>Custom boot-args</span>
              <input
                type="text"
                bind:value={bootArgs}
                placeholder="pio-error=0 -v"
                disabled={isWorking}
              />
            </label>
          </details>
        </section>

        {#if errorMessage}
          <div class="error" role="alert">{errorMessage}</div>
        {/if}
      </div>

      <footer class="dialog-footer">
        <button class="btn-secondary" onclick={handleClose} disabled={isWorking}>Cancel</button>
        <button
          class="btn-primary lg"
          onclick={handleBootSelected}
          disabled={isWorking || !isBootableMode || !canBootDifferent}
        >
          {#if isWorking}
            Working…
          {:else if selectedIpsw && selectedIpsw.metadataResolved}
            Boot iOS {selectedIpsw.iosVersion}
          {:else if selectedIpsw}
            Boot
          {:else}
            Boot
          {/if}
        </button>
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
    width: min(620px, 100%);
    max-height: min(calc(100vh - 32px), 760px);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    overflow: hidden;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: clamp(14px, 2.5vh, 22px) clamp(16px, 2.5vh, 24px) clamp(8px, 1.5vh, 12px);
  }

  h3 {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .close {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-sm);
    line-height: 0;
  }

  .close:hover:not(:disabled) {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .close:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .dialog-body {
    padding: 0 clamp(16px, 2.5vh, 24px) clamp(8px, 1.5vh, 16px);
    overflow-y: auto;
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: clamp(12px, 2vh, 18px);
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) clamp(16px, 2.5vh, 24px);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-primary);
  }

  @media (prefers-reduced-motion: reduce) {
    .overlay,
    .dialog {
      transition: none !important;
      animation: none !important;
    }
  }

  /* Sections */
  .dialog-body > :first-child {
    margin-top: clamp(8px, 1.5vh, 14px);
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .section-head {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .section-title {
    margin: 0;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .section-count {
    font-size: 0.7rem;
    color: var(--color-text-tertiary, var(--color-text-secondary));
    font-variant-numeric: tabular-nums;
  }

  /* History */
  .placeholder-row {
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    padding: 10px 12px;
    background: var(--color-bg-secondary);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-sm);
    text-align: center;
  }

  .history-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .history-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 8px 12px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .history-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .history-version {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .history-build {
    font-weight: 500;
    color: var(--color-text-secondary);
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.78rem;
  }

  .history-sub {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .last-badge {
    color: var(--color-accent);
    font-weight: 600;
  }

  .history-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  /* Buttons */
  .btn-primary {
    border: 1px solid var(--color-accent);
    background: var(--color-accent);
    color: white;
    border-radius: var(--radius-sm);
    padding: 6px 14px;
    font-size: 0.8125rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-primary.lg {
    padding: 9px 18px;
    font-size: 0.875rem;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border-radius: var(--radius-sm);
    padding: 9px 16px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-ghost {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    width: 26px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.78rem;
    cursor: pointer;
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--color-bg-primary);
    color: var(--color-danger);
    border-color: color-mix(in srgb, var(--color-danger) 30%, var(--color-border));
  }

  .btn-ghost:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* Advanced */
  .advanced {
    margin-top: 4px;
  }

  .advanced summary {
    cursor: pointer;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    font-weight: 600;
    padding: 4px 0;
    list-style: none;
  }

  .advanced summary::-webkit-details-marker {
    display: none;
  }

  .advanced summary::before {
    content: '▸';
    display: inline-block;
    margin-right: 6px;
    font-size: 1.5em;
    transition: transform 0.15s;
  }

  .advanced[open] summary::before {
    transform: rotate(90deg);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.8125rem;
    margin-top: var(--spacing-sm);
  }

  .field span {
    color: var(--color-text-secondary);
    font-weight: 500;
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

  .error {
    color: var(--color-danger);
    font-size: 0.8125rem;
    background: color-mix(in srgb, var(--color-danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }
</style>
