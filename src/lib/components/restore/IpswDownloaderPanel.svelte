<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    cancelIpswDownload,
    checkIpswSigning,
    downloadIpsw,
    listExistingIpsws,
    listFirmwares,
    onIpswDownloadProgress,
    type ExistingIpswEntry,
    type FirmwareListEntry,
    type IpswDownloadProgressEvent,
  } from '$lib/api/restore';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { toastStore } from '$lib/stores/toastStore.svelte';
  import ProgressBar from '$lib/components/common/ProgressBar.svelte';

  interface Props {
    deviceIdentifier: string | null;
    onUseIpsw: (path: string, sha1: string | null) => void;
  }

  let { deviceIdentifier, onUseIpsw }: Props = $props();

  let isLoading = $state(false);
  let isDownloading = $state(false);
  let errorMessage = $state<string | null>(null);
  let search = $state('');
  let selectedBuildId = $state<string | null>(null);
  let firmwares = $state<FirmwareListEntry[]>([]);
  let lastFetchedAt = $state<number | null>(null);
  let fromCache = $state(false);

  let currentDownloadId = $state<string | null>(null);
  let progress = $state<IpswDownloadProgressEvent | null>(null);
  let downloadedPath = $state<string | null>(null);
  let downloadedSha1 = $state<string | null>(null);
  let downloadedSha1Matches = $state<boolean | null>(null);
  let signingKnown = $state<boolean | null>(null);

  let existingIpsws = $state<ExistingIpswEntry[]>([]);

  let unlistenProgress: (() => void) | null = null;

  let selectedFirmware = $derived(
    selectedBuildId ? firmwares.find((f) => f.buildId === selectedBuildId) ?? null : null,
  );

  let filteredFirmwares = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return firmwares;
    return firmwares.filter((f) =>
      `${f.version} ${f.buildId}`.toLowerCase().includes(q),
    );
  });

  let canDownload = $derived(!!selectedFirmware && !isDownloading);

  $effect(() => {
    if (deviceIdentifier && firmwares.length === 0 && !isLoading) {
      void refreshFirmwares();
    }
  });

  $effect(() => {
    if (deviceIdentifier) {
      void loadExistingIpsws();
    }
  });

  async function loadExistingIpsws() {
    if (!deviceIdentifier) return;
    try {
      const result = await listExistingIpsws({ deviceIdentifier });
      existingIpsws = result.ipsws;
    } catch (error) {
      console.error('Failed to list existing IPSWs:', error);
    }
  }

  function useExistingIpsw(ipsw: ExistingIpswEntry) {
    onUseIpsw(ipsw.path, null);
    toastStore.success('IPSW selected', ipsw.fileName);
  }

  void onIpswDownloadProgress((event) => {
    if (!currentDownloadId || event.downloadId !== currentDownloadId) return;
    progress = event;
  }).then((unlisten) => {
    unlistenProgress = unlisten;
  });

  onDestroy(() => {
    unlistenProgress?.();
  });

  async function refreshFirmwares() {
    if (!deviceIdentifier) {
      errorMessage = 'Connect a device (or enter one manually) to fetch firmwares.';
      return;
    }
    isLoading = true;
    errorMessage = null;
    try {
      const result = await listFirmwares({ deviceIdentifier });
      firmwares = result.firmwares;
      fromCache = result.cached;
      lastFetchedAt = result.fetchedAtUnix;
      if (!selectedBuildId && result.firmwares.length > 0) {
        selectedBuildId = result.firmwares[0].buildId;
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : String(error);
      toastStore.error('Firmware lookup failed', errorMessage);
    } finally {
      isLoading = false;
    }
  }

  function suggestedFileName(fw: FirmwareListEntry): string {
    const model = (deviceIdentifier ?? 'device').replace(/\s+/g, '');
    return `${model}_${fw.version}_${fw.buildId}_Restore.ipsw`;
  }

  async function startDownload() {
    if (!selectedFirmware) {
      errorMessage = 'Select a firmware row before downloading.';
      return;
    }
    const downloadId = crypto.randomUUID();
    currentDownloadId = downloadId;
    progress = null;
    downloadedPath = null;
    downloadedSha1 = null;
    downloadedSha1Matches = null;
    signingKnown = null;
    isDownloading = true;
    errorMessage = null;

    logStore.append(`Downloading ${selectedFirmware.version} (${selectedFirmware.buildId})...`, 'info');
    toastStore.info('IPSW download started', `${selectedFirmware.version} (${selectedFirmware.buildId})`);

    try {
      const result = await downloadIpsw({
        url: selectedFirmware.url,
        outputDir: '',
        deviceIdentifier,
        fileName: suggestedFileName(selectedFirmware),
        expectedSha1: selectedFirmware.sha1,
        downloadId,
      });

      downloadedPath = result.path;
      downloadedSha1 = result.sha1;
      downloadedSha1Matches = result.sha1Matches;
      if (deviceIdentifier) {
        const signing = await checkIpswSigning({
          deviceIdentifier,
          buildId: selectedFirmware.buildId,
        });
        signingKnown = signing.signed;
      }

      toastStore.success('IPSW download complete', result.path);
      logStore.append(`Downloaded IPSW: ${result.path}`, 'info');
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      toastStore.error('IPSW download failed', msg);
      logStore.append(`Download failed: ${msg}`, 'stderr');
    } finally {
      isDownloading = false;
      currentDownloadId = null;
    }
  }

  async function cancelDownload() {
    if (!currentDownloadId) return;
    const result = await cancelIpswDownload({ downloadId: currentDownloadId });
    if (result.cancelled) {
      toastStore.warning('Download cancelled', 'aria2c process was terminated');
      logStore.append(`Cancelled download ${result.downloadId}`, 'info');
    }
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null || Number.isNaN(bytes)) return '—';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
  }

  function formatEta(seconds: number | null): string {
    if (seconds === null || seconds < 0) return '—';
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }

  function useDownloadedIpsw() {
    if (!downloadedPath) return;
    onUseIpsw(downloadedPath, downloadedSha1);
    toastStore.success('IPSW selected', 'Filled restore IPSW path');
  }
</script>

<div class="downloader-panel">
  <div class="panel-head">
    <h3>IPSW Downloader</h3>
    <button class="secondary" onclick={refreshFirmwares} disabled={isLoading || !deviceIdentifier}>
      {isLoading ? 'Refreshing…' : 'Refresh list'}
    </button>
  </div>

  <p class="meta">
    Device: <strong>{deviceIdentifier ?? 'Not detected'}</strong>
    {#if lastFetchedAt}
      · Last fetch: {new Date(lastFetchedAt * 1000).toLocaleString()}{fromCache ? ' (cache)' : ''}
    {/if}
  </p>

  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}

  <div class="controls">
    <input bind:value={search} placeholder="Filter by iOS or build (e.g. 8.4.1 / 12H321)" />
    <p class="workspace-note">Download destination: workspace <code>ipsw/&lt;device&gt;</code></p>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>iOS</th>
          <th>Build</th>
          <th>Size</th>
          <th>Signed</th>
        </tr>
      </thead>
      <tbody>
        {#if filteredFirmwares.length === 0}
          <tr><td colspan={4} class="empty">No firmware entries</td></tr>
        {:else}
          {#each filteredFirmwares as fw}
            <tr
              class:selected={selectedBuildId === fw.buildId}
              onclick={() => (selectedBuildId = fw.buildId)}
            >
              <td>{fw.version}</td>
              <td>{fw.buildId}</td>
              <td>{formatBytes(fw.sizeBytes)}</td>
              <td>{fw.signed === null ? '—' : fw.signed ? 'Yes' : 'No'}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  {#if existingIpsws.length > 0}
    <div class="existing-section">
      <h4>Existing IPSWs in Workspace</h4>
      <div class="existing-list">
        {#each existingIpsws as ipsw}
          <div class="existing-item">
            <div class="existing-info">
              <span class="existing-name">{ipsw.fileName}</span>
              <span class="existing-size">{formatBytes(ipsw.sizeBytes)}</span>
            </div>
            <button class="secondary" onclick={() => useExistingIpsw(ipsw)}>Use</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="actions">
    <button class="secondary" onclick={startDownload} disabled={!canDownload}>
      Download selected IPSW
    </button>
    <button class="danger" onclick={cancelDownload} disabled={!isDownloading || !currentDownloadId}>
      Cancel
    </button>
  </div>

  {#if isDownloading}
    <div class="progress">
      <ProgressBar
        value={progress?.percent ?? null}
        max={100}
        label="Download progress"
        indeterminate={progress === null || progress.percent === null}
      />
      <div class="progress-meta">
        <span>{formatBytes(progress?.downloadedBytes ?? null)} / {formatBytes(progress?.totalBytes ?? null)}</span>
        <span>{formatBytes(progress?.speedBps ?? null)}/s</span>
        <span>ETA {formatEta(progress?.etaSeconds ?? null)}</span>
      </div>
    </div>
  {/if}

  {#if downloadedPath}
    <div class="result">
      <div><strong>Saved:</strong> <code>{downloadedPath}</code></div>
      <div><strong>SHA-1:</strong> <code>{downloadedSha1}</code></div>
      <div>
        <strong>Hash check:</strong>
        {#if downloadedSha1Matches === null}
          Not checked
        {:else if downloadedSha1Matches}
          Match
        {:else}
          Mismatch
        {/if}
      </div>
      <div>
        <strong>Signing check:</strong>
        {#if signingKnown === null}
          Not checked
        {:else if signingKnown}
          Signed
        {:else}
          Not signed
        {/if}
      </div>
      <div class="actions">
        <button class="secondary" onclick={useDownloadedIpsw}>Use this IPSW for restore</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .downloader-panel {
    border-top: 1px solid var(--color-border);
    margin-top: var(--spacing-md);
    padding-top: var(--spacing-md);
  }

  .panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
  }

  .panel-head h3 {
    margin: 0;
    font-size: 0.92rem;
    color: var(--color-text-primary);
  }

  .meta {
    margin: 0 0 var(--spacing-md);
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .controls {
    display: grid;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .workspace-note {
    margin: 0;
    font-size: 0.76rem;
    color: var(--color-text-secondary);
  }

  .workspace-note code {
    color: var(--color-text-primary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.74rem;
  }

  input {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font: inherit;
    font-size: 0.85rem;
    padding: 8px 10px;
  }

  .table-wrap {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: auto;
    max-height: 260px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
  }

  th,
  td {
    padding: 8px;
    border-bottom: 1px solid var(--color-border);
    text-align: left;
    color: var(--color-text-primary);
  }

  tr {
    background: var(--color-bg-primary);
    cursor: pointer;
  }

  tr.selected {
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-bg-primary));
  }

  .empty {
    color: var(--color-text-secondary);
    text-align: center;
  }

  .actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    margin-top: var(--spacing-md);
  }

  button.secondary,
  button.danger {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 12px;
  }

  button.secondary {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
  }

  button.danger {
    background: var(--color-danger);
    color: #fff;
    border-color: var(--color-danger);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .progress {
    margin-top: var(--spacing-md);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
  }

  .progress-meta {
    margin-top: 6px;
    display: flex;
    justify-content: space-between;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    gap: var(--spacing-sm);
  }

  .result,
  .error {
    margin-top: var(--spacing-md);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 0.8rem;
    display: grid;
    gap: 6px;
  }

  .error {
    border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    color: var(--color-danger);
  }

  code {
    overflow-wrap: anywhere;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

  .existing-section {
    margin-top: var(--spacing-md);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
  }

  .existing-section h4 {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-primary);
  }

  .existing-list {
    display: grid;
    gap: var(--spacing-xs);
  }

  .existing-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 6px 8px;
    background: var(--color-bg-secondary);
    border-radius: var(--radius-sm);
  }

  .existing-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .existing-name {
    font-size: 0.8rem;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .existing-size {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }
</style>
