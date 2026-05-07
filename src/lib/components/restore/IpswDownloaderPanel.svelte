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

  let unlistenProgress: (() => void) | null = null;

  type SortCol = 'ios' | 'build' | 'size' | 'signed';
  type SortDir = 'asc' | 'desc';
  type FirmwareFilter = 'all' | 'downloaded';

  let firmwareFilter = $state<FirmwareFilter>('all');
  let existingIpsws = $state<ExistingIpswEntry[]>([]);

  function downloadedEntryFor(fw: FirmwareListEntry): ExistingIpswEntry | null {
    return existingIpsws.find((e) => e.fileName.includes(fw.buildId)) ?? null;
  }

  let sortCol = $state<SortCol>('ios');
  let sortDir = $state<SortDir>('desc');

  function parseVersion(v: string): number[] {
    return v.split('.').map((p) => parseInt(p, 10) || 0);
  }

  function compareVersions(a: string, b: string): number {
    const av = parseVersion(a);
    const bv = parseVersion(b);
    for (let i = 0; i < Math.max(av.length, bv.length); i++) {
      const diff = (av[i] ?? 0) - (bv[i] ?? 0);
      if (diff !== 0) return diff;
    }
    return 0;
  }

  function toggleSort(col: SortCol) {
    if (sortCol === col) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortCol = col;
      sortDir = col === 'ios' ? 'desc' : 'asc';
    }
  }

  let selectedFirmware = $derived(
    selectedBuildId ? firmwares.find((f) => f.buildId === selectedBuildId) ?? null : null,
  );

  let filteredFirmwares = $derived.by(() => {
    const q = search.trim().toLowerCase();
    let list = q
      ? firmwares.filter((f) => `${f.version} ${f.buildId}`.toLowerCase().includes(q))
      : [...firmwares];

    if (firmwareFilter === 'downloaded') {
      list = list.filter((f) => existingIpsws.some((e) => e.fileName.includes(f.buildId)));
    }

    list.sort((a, b) => {
      let cmp = 0;
      if (sortCol === 'ios') cmp = compareVersions(a.version, b.version);
      else if (sortCol === 'build') cmp = a.buildId.localeCompare(b.buildId);
      else if (sortCol === 'size') cmp = (a.sizeBytes ?? 0) - (b.sizeBytes ?? 0);
      else if (sortCol === 'signed') cmp = (a.signed === b.signed ? 0 : a.signed ? 1 : -1);
      return sortDir === 'asc' ? cmp : -cmp;
    });
    return list;
  });

  let selectedDownloadedEntry = $derived(
    selectedFirmware ? downloadedEntryFor(selectedFirmware) : null
  );

  let canDownload = $derived(!!selectedFirmware && !isDownloading && !selectedDownloadedEntry);

  let isVerifying = $state(false);
  let lastProgressTime = $state<number | null>(null);
  let verifyCheckInterval: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    if (deviceIdentifier && firmwares.length === 0 && !isLoading) {
      void refreshFirmwares();
    }
  });

  void onIpswDownloadProgress((event) => {
    if (!currentDownloadId || event.downloadId !== currentDownloadId) return;
    progress = event;
    lastProgressTime = Date.now();
    isVerifying = false;
  }).then((unlisten) => {
    unlistenProgress = unlisten;
  });

  onDestroy(() => {
    unlistenProgress?.();
    if (verifyCheckInterval !== null) clearInterval(verifyCheckInterval);
  });

  async function refreshFirmwares() {
    if (!deviceIdentifier) {
      errorMessage = 'Connect a device (or enter one manually) to fetch firmwares.';
      return;
    }
    isLoading = true;
    errorMessage = null;
    try {
      const [result, existing] = await Promise.all([
        listFirmwares({ deviceIdentifier }),
        listExistingIpsws({ deviceIdentifier }).catch(() => ({ ipsws: [] })),
      ]);
      firmwares = result.firmwares;
      fromCache = result.cached;
      lastFetchedAt = result.fetchedAtUnix;
      existingIpsws = existing.ipsws;
      if (!selectedBuildId && result.firmwares.length > 0) {
        const newest = [...result.firmwares].sort((a, b) => -compareVersions(a.version, b.version));
        selectedBuildId = newest[0].buildId;
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
    lastProgressTime = null;
    isVerifying = false;
    downloadedPath = null;
    downloadedSha1 = null;
    downloadedSha1Matches = null;
    signingKnown = null;
    isDownloading = true;
    errorMessage = null;

    verifyCheckInterval = setInterval(() => {
      if (lastProgressTime !== null && Date.now() - lastProgressTime > 2000) {
        isVerifying = true;
      }
    }, 500);

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
      if (deviceIdentifier) {
        listExistingIpsws({ deviceIdentifier }).then((r) => { existingIpsws = r.ipsws; }).catch(() => {});
      }
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      toastStore.error('IPSW download failed', msg);
      logStore.append(`Download failed: ${msg}`, 'stderr');
    } finally {
      isDownloading = false;
      currentDownloadId = null;
      isVerifying = false;
      if (verifyCheckInterval !== null) {
        clearInterval(verifyCheckInterval);
        verifyCheckInterval = null;
      }
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
  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}

  <div class="controls">
    <input bind:value={search} placeholder="Filter by iOS or build (e.g. 8.4.1 / 12H321)" />
    <div class="filter-row">
      <div class="filter-tabs">
        <button
          class:active={firmwareFilter === 'all'}
          onclick={() => (firmwareFilter = 'all')}
        >All</button>
        <button
          class:active={firmwareFilter === 'downloaded'}
          onclick={() => (firmwareFilter = 'downloaded')}
        >Downloaded{existingIpsws.length > 0 ? ` (${existingIpsws.filter(e => firmwares.some(f => e.fileName.includes(f.buildId))).length})` : ''}</button>
      </div>
      <button class="secondary" onclick={refreshFirmwares} disabled={isLoading || !deviceIdentifier}>
        {isLoading ? 'Refreshing…' : 'Refresh list'}
      </button>
    </div>
  </div>

  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th onclick={() => toggleSort('ios')} class:sorted={sortCol === 'ios'}>
            iOS{#if sortCol === 'ios'}<span class="sort-arrow">{sortDir === 'asc' ? ' ↑' : ' ↓'}</span>{/if}
          </th>
          <th onclick={() => toggleSort('build')} class:sorted={sortCol === 'build'}>
            Build{#if sortCol === 'build'}<span class="sort-arrow">{sortDir === 'asc' ? ' ↑' : ' ↓'}</span>{/if}
          </th>
          <th onclick={() => toggleSort('size')} class:sorted={sortCol === 'size'}>
            Size{#if sortCol === 'size'}<span class="sort-arrow">{sortDir === 'asc' ? ' ↑' : ' ↓'}</span>{/if}
          </th>
          <th onclick={() => toggleSort('signed')} class:sorted={sortCol === 'signed'}>
            Signed{#if sortCol === 'signed'}<span class="sort-arrow">{sortDir === 'asc' ? ' ↑' : ' ↓'}</span>{/if}
          </th>
        </tr>
      </thead>
      <tbody>
        {#if filteredFirmwares.length === 0}
          <tr><td colspan={4} class="empty">No firmware entries</td></tr>
        {:else}
          {#each filteredFirmwares as fw}
            {@const isOnDisk = downloadedEntryFor(fw) !== null}
            <tr
              class:selected={selectedBuildId === fw.buildId}
              class:on-disk={isOnDisk}
              onclick={() => (selectedBuildId = fw.buildId)}
            >
              <td>
                {fw.version}
                {#if isOnDisk}<span class="on-disk-badge">✓</span>{/if}
              </td>
              <td>{fw.buildId}</td>
              <td>{formatBytes(fw.sizeBytes)}</td>
              <td>{fw.signed === null ? '—' : fw.signed ? 'Yes' : 'No'}</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>

  <div class="actions">
    {#if selectedDownloadedEntry}
      <button class="use-downloaded" onclick={() => { if (selectedDownloadedEntry) onUseIpsw(selectedDownloadedEntry.path, null); }}>
        Use selected IPSW
      </button>
    {:else}
      <button class="secondary" onclick={startDownload} disabled={!canDownload}>
        Download selected IPSW
      </button>
    {/if}
    <button class="danger" onclick={cancelDownload} disabled={!isDownloading || !currentDownloadId}>
      Cancel
    </button>
  </div>

  {#if isDownloading}
    <div class="progress">
      {#if isVerifying}
        <ProgressBar
          value={null}
          max={100}
          label="Verifying download…"
          indeterminate={true}
        />
        <div class="progress-meta">
          <span class="verifying-label">Checking integrity — please wait…</span>
        </div>
      {:else}
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
      {/if}
    </div>
  {/if}

  {#if downloadedPath}
    <div class="result">
      <div class="result-header">
        <span class="result-icon">✓</span>
        <div class="result-title-group">
          <span class="result-title">Download complete</span>
          <span class="result-path">{downloadedPath}</span>
        </div>
      </div>

      <div class="result-grid">
        <span class="result-label">SHA-1</span>
        <code class="result-hash">{downloadedSha1 ?? '—'}</code>

        <span class="result-label">Integrity</span>
        <span
          class="result-value"
          class:ok={downloadedSha1Matches === true}
          class:fail={downloadedSha1Matches === false}
        >
          {#if downloadedSha1Matches === null}—{:else if downloadedSha1Matches}✓ Verified{:else}✗ Mismatch{/if}
        </span>

        <span class="result-label">Signing</span>
        <span
          class="result-value"
          class:ok={signingKnown === true}
        >
          {#if signingKnown === null}—{:else if signingKnown}✓ Signed{:else}Not signed{/if}
        </span>
      </div>

      <div class="result-actions">
        <button class="use-ipsw" onclick={useDownloadedIpsw}>Use this IPSW for restore</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .downloader-panel {
    padding: var(--spacing-md) var(--spacing-md) var(--spacing-sm);
  }

  .controls {
    display: grid;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }

  .filter-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
  }

  .filter-tabs {
    display: flex;
    gap: 2px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 2px;
    width: fit-content;
  }

  .filter-tabs button {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: calc(var(--radius-sm) - 2px);
    padding: 4px 12px;
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: color 0.15s;
  }

  .filter-tabs button.active {
    background: var(--color-bg-primary);
    border-color: var(--color-border);
    color: var(--color-text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  }



  .workspace-note code {
    color: var(--color-text-primary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.74rem;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font: inherit;
    font-size: 0.85rem;
    padding: 9px 12px;
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

  th {
    background: var(--color-bg-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-secondary);
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    text-align: left;
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    transition: color 0.15s;
  }

  th:hover {
    color: var(--color-text-primary);
  }

  th.sorted {
    color: var(--color-accent);
  }

  .sort-arrow {
    font-size: 0.7rem;
    opacity: 0.8;
  }

  td {
    padding: 10px 12px;
    border-bottom: 1px solid var(--color-border);
    text-align: left;
    color: var(--color-text-primary);
  }

  tr:last-child td {
    border-bottom: none;
  }

  tr {
    background: var(--color-bg-primary);
    cursor: pointer;
  }

  tr:not(.selected):hover td {
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-bg-primary));
  }

  tr.selected td {
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-bg-primary));
  }

  .on-disk-badge {
    display: inline-block;
    margin-left: 6px;
    font-size: 0.68rem;
    font-weight: 700;
    color: var(--color-success);
    vertical-align: middle;
  }

  button.use-downloaded {
    background: var(--color-success);
    color: #fff;
    border: 1px solid var(--color-success);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 16px;
    cursor: pointer;
  }

  button.use-downloaded:hover {
    opacity: 0.88;
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
    padding: 8px 16px;
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
    margin-bottom: var(--spacing-sm);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-md);
    background: var(--color-bg-primary);
  }

  .progress-meta {
    margin-top: 12px;
    display: flex;
    justify-content: space-between;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    gap: var(--spacing-sm);
  }

  .verifying-label {
    font-style: italic;
    color: var(--color-text-secondary);
  }

  .result {
    margin-top: var(--spacing-md);
    margin-bottom: var(--spacing-sm);
    border: 1px solid color-mix(in srgb, var(--color-success) 30%, var(--color-border));
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-success) 6%, var(--color-bg-primary));
    overflow: hidden;
  }

  .result-header {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    border-bottom: 1px solid color-mix(in srgb, var(--color-success) 20%, var(--color-border));
  }

  .result-icon {
    font-size: 1rem;
    font-weight: 700;
    color: var(--color-success);
    flex-shrink: 0;
    margin-top: 1px;
  }

  .result-title-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .result-title {
    font-size: 0.875rem;
    font-weight: 700;
    color: var(--color-text-primary);
  }

  .result-path {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    row-gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    align-items: center;
  }

  .result-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding-right: var(--spacing-md);
    white-space: nowrap;
  }

  .result-hash {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .result-value {
    font-size: 0.82rem;
    color: var(--color-text-secondary);
  }

  .result-value.ok {
    color: var(--color-success);
    font-weight: 600;
  }

  .result-value.fail {
    color: var(--color-danger);
    font-weight: 600;
  }

  .result-actions {
    padding: var(--spacing-xs) var(--spacing-md) var(--spacing-md);
    display: flex;
    justify-content: flex-end;
  }

  button.use-ipsw {
    background: var(--color-accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 16px;
    cursor: pointer;
  }

  button.use-ipsw:hover {
    opacity: 0.88;
  }

  .error {
    margin-top: var(--spacing-md);
    border: 1px solid color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    background: var(--color-bg-primary);
    color: var(--color-danger);
    font-size: 0.8rem;
    display: grid;
    gap: 6px;
  }

  code {
    overflow-wrap: anywhere;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  }

</style>
