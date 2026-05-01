<script lang="ts">
  import { runKloader } from '../../api/jailbreak';
  import { 
    listJustBootHistory, 
    recordJustBoot, 
    forgetJustBoot, 
    prepareAndJustBoot,
    type JustBootEntry,
    type JustBootEntryInput,
    type PrepareAndJustBootRequest
  } from '../../api/justBoot';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import { logStore } from '../../stores/logStore.svelte';
  import { toastStore } from '../../stores/toastStore.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open: isOpen, onClose }: Props = $props();

  let history = $state<JustBootEntry[]>([]);
  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);
  let isLoadingHistory = $state(false);

  // Form states for new build section
  let ipswPath = $state('');
  let buildId = $state('');
  let iosVersion = $state('');
  let includeIbec = $state(true);
  let bootArgs = $state('');
  let manualIbssPath = $state('');
  let manualIbecPath = $state('');
  let showAdvanced = $state(false);

  // Device state
  let deviceState = $derived(deviceStore.state);
  let isPwnDfu = $derived(deviceState.mode === 'pwnDFU');
  let deviceEcid = $derived(deviceState.ecid);
  let deviceProductType = $derived(deviceState.product_type);

  // History filtering
  let heroEntry = $derived<JustBootEntry | null>(
    deviceEcid ? history.find(entry => 
      entry.ecid === deviceEcid && 
      entry.repackedIbssPath !== null
    ) ?? null : null
  );

  let thisDeviceEntries = $derived(
    history.filter(entry => 
      entry.productType === deviceProductType && 
      entry !== heroEntry
    )
  );

  let otherDeviceEntries = $derived(
    history.filter(entry => entry.productType !== deviceProductType)
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

  function inferProcessorGen(product: string | null): number | null {
    if (!product) return null;
    if (/^iPhone(1|2),/.test(product) || /^iPod(1|2),/.test(product)) return 1;
    if (product === 'iPod3,1') return 3;
    if (/^iPhone3,/.test(product) || product === 'iPad1,1' || product === 'iPod4,1') return 4;
    if (product === 'iPhone4,1' || /^iPad2,/.test(product) || /^iPad3,[1-3]/.test(product) || product === 'iPod5,1') return 5;
    if (/^iPhone5,/.test(product) || /^iPad3,[4-6]/.test(product)) return 6;
    if (/^iPhone6,/.test(product) || /^iPad4,/.test(product)) return 7;
    if (/^iPhone7,/.test(product) || product === 'iPod7,1' || /^iPad5,/.test(product)) return 8;
    if (/^iPhone8,/.test(product) || /^iPad6,/.test(product)) return 9;
    if (/^iPhone9,/.test(product) || /^iPad7,/.test(product)) return 10;
    return null;
  }

  function updateIncludeIbecDefault() {
    const gen = inferProcessorGen(deviceProductType);
    includeIbec = gen !== null && gen >= 6;
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
        ipswPath = selected;
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      toastStore.error('Browse failed', msg);
    }
  }

  async function handleHeroBoot() {
    if (!heroEntry || !heroEntry.repackedIbssPath) return;
    if (!isPwnDfu) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Booting iOS ${heroEntry.iosVersion ?? '?'} (${heroEntry.buildId})`;
    logStore.append(`${label}...`, 'info');
    
    try {
      await runKloader({ 
        ibssPath: heroEntry.repackedIbssPath, 
        ibecPath: heroEntry.repackedIbecPath 
      });
      
      // Update last booted time
      await recordJustBoot({
        ecid: heroEntry.ecid,
        productType: heroEntry.productType,
        deviceName: heroEntry.deviceName,
        buildId: heroEntry.buildId,
        iosVersion: heroEntry.iosVersion,
        bootArgs: heroEntry.bootArgs,
        repackedIbssPath: heroEntry.repackedIbssPath,
        repackedIbecPath: heroEntry.repackedIbecPath,
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

  async function handleHistoryBoot(entry: JustBootEntry) {
    if (!entry.repackedIbssPath) return;
    if (!isPwnDfu) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    isWorking = true;
    errorMessage = null;
    const label = `Booting iOS ${entry.iosVersion ?? '?'} (${entry.buildId})`;
    logStore.append(`${label}...`, 'info');
    
    try {
      await runKloader({ 
        ibssPath: entry.repackedIbssPath, 
        ibecPath: entry.repackedIbecPath 
      });
      
      // Update last booted time
      await recordJustBoot({
        ecid: entry.ecid,
        productType: entry.productType,
        deviceName: entry.deviceName,
        buildId: entry.buildId,
        iosVersion: entry.iosVersion,
        bootArgs: entry.bootArgs,
        repackedIbssPath: entry.repackedIbssPath,
        repackedIbecPath: entry.repackedIbecPath,
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
    if (!isPwnDfu) {
      errorMessage = 'Device must be in pwnDFU mode before booting';
      return;
    }

    // Check if manual paths are provided
    if (manualIbssPath.trim() && manualIbecPath.trim()) {
      isWorking = true;
      errorMessage = null;
      const label = 'Booting with manual paths';
      logStore.append(`${label}...`, 'info');
      
      try {
        await runKloader({ 
          ibssPath: manualIbssPath.trim(), 
          ibecPath: manualIbecPath.trim() 
        });
        
        // Record the boot
        await recordJustBoot({
          ecid: deviceEcid || '',
          productType: deviceProductType || '',
          deviceName: deviceState.name,
          buildId: buildId.trim(),
          iosVersion: iosVersion.trim() || null,
          bootArgs: bootArgs.trim() || null,
          repackedIbssPath: manualIbssPath.trim(),
          repackedIbecPath: manualIbecPath.trim(),
          sourceIpswPath: ipswPath.trim()
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
        bootArgs: bootArgs.trim() || null,
        includeIbec
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
      updateIncludeIbecDefault();
    }
  });

  $effect(() => {
    updateIncludeIbecDefault();
  });
</script>

<svelte:window onkeydown={onKey} />

{#if isOpen}
  <div class="overlay" role="presentation" onclick={handleClose}>
    <div
      class="dialog"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="just-boot-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 id="just-boot-title">Just Boot</h3>
      
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

      <!-- pwnDFU Warning -->
      {#if deviceState.connected && !isPwnDfu}
        <div class="warning-banner">
          ⚠️ Device must be in pwnDFU mode before booting
        </div>
      {/if}

      <!-- Hero Card - Boot last build -->
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
                disabled={isWorking || !isPwnDfu}
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

      <!-- Boot History -->
      <div class="history-section">
        <h4>Boot history</h4>
        
        {#if isLoadingHistory}
          <div class="loading">Loading history...</div>
        {:else if history.length === 0}
          <div class="empty-state">No boot history found</div>
        {:else}
          <!-- This Device entries -->
          {#if thisDeviceEntries.length > 0}
            <div class="history-group">
              <h5>This device</h5>
              {#each thisDeviceEntries as entry}
                <div class="history-item">
                  <div class="item-info">
                    <span class="device-name">{entry.deviceName || entry.productType}</span>
                    <span class="build-info">iOS {entry.iosVersion ?? '?'} ({entry.buildId})</span>
                    <span class="last-booted">{formatRelativeTime(entry.lastBootedAt)}</span>
                  </div>
                  <div class="item-actions">
                    {#if entry.repackedIbssPath}
                      <button 
                        class="primary small"
                        onclick={() => handleHistoryBoot(entry)}
                        disabled={isWorking || !isPwnDfu}
                      >
                        Boot
                      </button>
                    {:else}
                      <button 
                        class="secondary small"
                        onclick={() => {
                          ipswPath = entry.sourceIpswPath || '';
                          buildId = entry.buildId;
                          iosVersion = entry.iosVersion || '';
                        }}
                        disabled={isWorking}
                      >
                        Prepare
                      </button>
                    {/if}
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
            </div>
          {/if}

          <!-- Other Devices entries -->
          {#if otherDeviceEntries.length > 0}
            <details class="history-group">
              <summary>
                <h5>Other devices</h5>
                <span class="count">{otherDeviceEntries.length}</span>
              </summary>
              {#each otherDeviceEntries as entry}
                <div class="history-item">
                  <div class="item-info">
                    <span class="device-name">{entry.deviceName || entry.productType}</span>
                    <span class="build-info">iOS {entry.iosVersion ?? '?'} ({entry.buildId})</span>
                    <span class="last-booted">{formatRelativeTime(entry.lastBootedAt)}</span>
                  </div>
                  <div class="item-actions">
                    {#if entry.repackedIbssPath}
                      <button 
                        class="primary small"
                        onclick={() => handleHistoryBoot(entry)}
                        disabled={isWorking || !isPwnDfu}
                        title={!isPwnDfu ? 'Connect this device to boot' : ''}
                      >
                        Boot
                      </button>
                    {:else}
                      <button 
                        class="secondary small"
                        onclick={() => {
                          ipswPath = entry.sourceIpswPath || '';
                          buildId = entry.buildId;
                          iosVersion = entry.iosVersion || '';
                        }}
                        disabled={isWorking}
                      >
                        Prepare
                      </button>
                    {/if}
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
            </details>
          {/if}
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

          <label class="checkbox">
            <input type="checkbox" bind:checked={includeIbec} disabled={isWorking} />
            <span>Include patched iBEC</span>
          </label>

          <details class="advanced-section" bind:open={showAdvanced}>
            <summary>Advanced</summary>
            
            <label class="field">
              <span>Custom boot-args</span>
              <input bind:value={bootArgs} placeholder="pio-error=0 -v" disabled={isWorking} />
            </label>

            <div class="manual-paths">
              <label class="field">
                <span>Patched iBSS path (manual)</span>
                <input bind:value={manualIbssPath} placeholder="/path/to/iBSS.repacked" disabled={isWorking} />
              </label>
              <label class="field">
                <span>Patched iBEC path (manual)</span>
                <input bind:value={manualIbecPath} placeholder="/path/to/iBEC.repacked" disabled={isWorking} />
              </label>
            </div>
          </details>

          {#if errorMessage}
            <div class="error">{errorMessage}</div>
          {/if}

          <div class="actions">
            <button 
              class="primary"
              onclick={handlePrepareAndBoot}
              disabled={isWorking || !isPwnDfu || !ipswPath.trim() || !buildId.trim()}
            >
              {isWorking ? 'Working…' : manualIbssPath.trim() && manualIbecPath.trim() ? 'Boot' : 'Prepare & Boot'}
            </button>
          </div>
        </div>
      </details>

      <div class="footer-actions">
        <button class="secondary" onclick={handleClose} disabled={isWorking}>Cancel</button>
      </div>
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
    padding: 16px;
  }
  
  .dialog {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    width: min(600px, 100%);
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
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

  h5 { 
    margin: 0; 
    font-size: 0.875rem; 
    font-weight: 600;
    color: var(--color-text-secondary);
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

  .warning-banner {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) 40%, transparent);
    color: var(--color-warning-text);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    font-weight: 500;
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

  .history-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
  }

  .history-group summary {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    padding: var(--spacing-xs) 0;
  }

  .history-group summary::-webkit-details-marker {
    display: none;
  }

  .count {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
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

  .checkbox {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.8125rem;
  }

  .checkbox input {
    margin: 0;
  }

  .advanced-section {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-xs);
  }

  .advanced-section summary {
    cursor: pointer;
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
  }

  .manual-paths {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding-left: var(--spacing-sm);
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

  .footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-md);
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--color-border);
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