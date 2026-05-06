<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    cancelIpswDownload,
    downloadIpsw,
    listExistingIpsws,
    listFirmwares,
    onIpswDownloadProgress,
    type ExistingIpswEntry,
    type FirmwareListEntry,
    type IpswDownloadProgressEvent,
    type RestoreOption,
  } from '$lib/api/restore';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { toastStore } from '$lib/stores/toastStore.svelte';
  import ProgressBar from '$lib/components/common/ProgressBar.svelte';

  interface Props {
    option: RestoreOption | null;
    productType: string | null;
    onSelect: (path: string, sha1: string | null) => void;
  }

  let { option, productType, onSelect }: Props = $props();

  // 'targeted' → OTA downgrade or latest (known/resolvable single version)
  // 'browse'   → all other paths (full catalogue + workspace list)
  let ipswMode = $derived<'targeted' | 'browse'>(
    option?.kind === 'otaDowngrade' || option?.kind === 'latest' ? 'targeted' : 'browse',
  );

  // ── data ──────────────────────────────────────────────────────────────────
  let existingIpsws = $state<ExistingIpswEntry[]>([]);
  let firmwares = $state<FirmwareListEntry[]>([]);
  let isLoadingExisting = $state(false);
  let isLoadingFirmwares = $state(false);
  let firmwareSearch = $state('');

  // ── targeted-mode state ───────────────────────────────────────────────────
  let resolvedTargetVersion = $state<string | null>(null);
  let autoMatch = $state<ExistingIpswEntry | null>(null);
  let autoMatchNotified = $state(false);

  // ── browse-mode selection ─────────────────────────────────────────────────
  let selectedBuildId = $state<string | null>(null);

  // ── download state ────────────────────────────────────────────────────────
  let isDownloading = $state(false);
  let currentDownloadId = $state<string | null>(null);
  let downloadProgress = $state<IpswDownloadProgressEvent | null>(null);
  let lastUnavailableLogKey: string | null = null;
  let lastPropsKey: string | null = null;
  let lastAutoMatchMissKey: string | null = null;
  let forceBrowseInTargeted = $state(false);
  let selectedExistingPath = $state<string | null>(null);

  let unlistenProgress: (() => void) | null = null;

  // ── derived ───────────────────────────────────────────────────────────────
  let targetFirmware = $derived.by(() => {
    const targetVersion = normalizeVersion(resolvedTargetVersion);
    if (!targetVersion || firmwares.length === 0) return null;
    return firmwares.find((f) => normalizeVersion(f.version) === targetVersion) ?? null;
  });

  let matchingTargetIpsws = $derived.by(() => {
    const targetVersion = normalizeVersion(resolvedTargetVersion);
    if (!targetVersion || existingIpsws.length === 0) return [];
    return existingIpsws.filter((ipsw) => ipsw.fileName.includes(`_${targetVersion}_`));
  });

  let filteredFirmwares = $derived.by(() => {
    const q = firmwareSearch.trim().toLowerCase();
    if (!q) return firmwares;
    return firmwares.filter((f) =>
      `${f.version} ${f.buildId}`.toLowerCase().includes(q),
    );
  });

  let selectedFirmware = $derived(
    selectedBuildId ? (firmwares.find((f) => f.buildId === selectedBuildId) ?? null) : null,
  );

  let canDownloadSelected = $derived(!!selectedFirmware && !isDownloading);
  let showBrowseMode = $derived(
    ipswMode === 'browse' ||
      forceBrowseInTargeted ||
      (ipswMode === 'targeted' && !!resolvedTargetVersion && !isLoadingFirmwares && !targetFirmware),
  );

  function logDiag(event: string, payload: Record<string, unknown> = {}) {
    logStore.append(
      `[restore:ipsw-pane] ${event} ${JSON.stringify(payload)}`,
      'info',
    );
  }

  // ── load on prop change ───────────────────────────────────────────────────
  $effect(() => {
    const pt = productType;
    const opt = option;
    const propsKey = `${opt?.kind ?? 'null'}|${opt?.targetVersion ?? 'null'}|${pt ?? 'null'}`;

    if (lastPropsKey === propsKey) return;
    lastPropsKey = propsKey;

    logDiag('props-change', {
      optionKind: opt?.kind ?? null,
      optionTargetVersion: opt?.targetVersion ?? null,
      productType: pt ?? null,
    });

    resolvedTargetVersion = opt?.targetVersion ?? null;
    autoMatch = null;
    autoMatchNotified = false;
    selectedBuildId = null;
    downloadProgress = null;
    forceBrowseInTargeted = false;
    selectedExistingPath = null;
    lastUnavailableLogKey = null;
    lastAutoMatchMissKey = null;

    if (!pt) {
      existingIpsws = [];
      firmwares = [];
      logDiag('props-change-no-product-type', {
        existingIpswsCleared: true,
        firmwaresCleared: true,
      });
      return;
    }

    void loadExistingIpsws(pt);
    void loadFirmwares(pt);
  });

  // ── resolve 'latest' version from signed firmware list ───────────────────
  $effect(() => {
    if (option?.kind !== 'latest' || resolvedTargetVersion || firmwares.length === 0) return;
    const signed = firmwares.find((f) => f.signed === true) ?? firmwares[0];
    if (signed) {
      resolvedTargetVersion = signed.version;
      logDiag('resolved-latest-target-version', {
        resolvedTargetVersion,
        buildId: signed.buildId,
        signed: signed.signed,
      });
    }
  });

  // ── auto-match workspace IPSW for targeted mode ───────────────────────────
  $effect(() => {
    const tv = resolvedTargetVersion;
    if (ipswMode !== 'targeted' || !tv || isLoadingExisting || autoMatchNotified) return;
    const match = matchingTargetIpsws[0] ?? null;
    if (match) {
      autoMatch = match;
      autoMatchNotified = true;
      selectedExistingPath = match.path;
      onSelect(match.path, null);
      logStore.append(`Auto-selected workspace IPSW: ${match.fileName}`, 'info');
      logDiag('targeted-auto-match-found', {
        targetVersion: tv,
        fileName: match.fileName,
        path: match.path,
      });
    } else {
      const missKey = `${productType ?? 'unknown'}:${normalizeVersion(tv) ?? tv}:${existingIpsws.length}`;
      if (lastAutoMatchMissKey !== missKey) {
        lastAutoMatchMissKey = missKey;
        logDiag('targeted-auto-match-miss', {
          targetVersion: tv,
          existingIpswCount: existingIpsws.length,
          autoMatchNotified,
        });
      }
    }
  });

  $effect(() => {
    if (ipswMode !== 'targeted' || !resolvedTargetVersion || isLoadingFirmwares || targetFirmware) return;
    const key = `${option?.kind ?? 'unknown'}:${productType ?? 'unknown'}:${resolvedTargetVersion}`;
    if (lastUnavailableLogKey === key) return;
    lastUnavailableLogKey = key;
    logDiag('target-version-not-in-catalogue', {
      optionKind: option?.kind ?? null,
      productType: productType ?? null,
      resolvedTargetVersion,
      firmwareCount: firmwares.length,
      existingIpswCount: existingIpsws.length,
      autoMatchPresent: !!autoMatch,
      autoMatchNotified,
    });
  });

  void onIpswDownloadProgress((event) => {
    if (!currentDownloadId || event.downloadId !== currentDownloadId) return;
    downloadProgress = event;
  }).then((unlisten) => {
    unlistenProgress = unlisten;
  });

  onDestroy(() => {
    unlistenProgress?.();
  });

  // ── functions ─────────────────────────────────────────────────────────────
  async function loadExistingIpsws(device: string) {
    isLoadingExisting = true;
    try {
      const result = await listExistingIpsws({ deviceIdentifier: device });
      existingIpsws = result.ipsws;
      logDiag('load-existing-ipsws-success', {
        device,
        count: result.ipsws.length,
      });
    } catch {
      existingIpsws = [];
      logDiag('load-existing-ipsws-failed', { device });
    } finally {
      isLoadingExisting = false;
    }
  }

  async function loadFirmwares(device: string) {
    isLoadingFirmwares = true;
    try {
      const result = await listFirmwares({ deviceIdentifier: device });
      firmwares = result.firmwares;
      logDiag('load-firmwares-success', {
        device,
        count: result.firmwares.length,
        cached: result.cached,
      });
    } catch {
      firmwares = [];
      logDiag('load-firmwares-failed', { device });
    } finally {
      isLoadingFirmwares = false;
    }
  }

  async function startDownload(fw: FirmwareListEntry) {
    const downloadId = crypto.randomUUID();
    currentDownloadId = downloadId;
    downloadProgress = null;
    isDownloading = true;

    logStore.append(`Downloading iOS ${fw.version} (${fw.buildId})…`, 'info');
    toastStore.info('IPSW download started', `${fw.version} (${fw.buildId})`);
    logDiag('download-start', {
      version: fw.version,
      buildId: fw.buildId,
      expectedSha1: fw.sha1,
      downloadId,
    });

    try {
      const result = await downloadIpsw({
        url: fw.url,
        outputDir: '',
        deviceIdentifier: productType,
        fileName: null,
        expectedSha1: fw.sha1,
        downloadId,
      });
      onSelect(result.path, result.sha1);
      selectedExistingPath = result.path;
      toastStore.success('IPSW download complete', result.path);
      logStore.append(`Downloaded IPSW: ${result.path}`, 'info');
      logDiag('download-success', {
        path: result.path,
        sha1: result.sha1,
        sha1Matches: result.sha1Matches,
        downloadId: result.downloadId,
      });
      if (productType) void loadExistingIpsws(productType);
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      toastStore.error('IPSW download failed', msg);
      logStore.append(`Download failed: ${msg}`, 'stderr');
      logDiag('download-failed', {
        version: fw.version,
        buildId: fw.buildId,
        error: msg,
        downloadId,
      });
    } finally {
      isDownloading = false;
      currentDownloadId = null;
    }
  }

  async function cancelDownload() {
    if (!currentDownloadId) return;
    await cancelIpswDownload({ downloadId: currentDownloadId });
    toastStore.warning('Download cancelled', '');
  }

  function dismissAutoMatch() {
    logDiag('auto-match-dismissed', {
      hadAutoMatch: !!autoMatch,
      targetVersion: resolvedTargetVersion,
      autoMatchNotified,
      existingIpswCount: existingIpsws.length,
    });
    autoMatch = null;
    forceBrowseInTargeted = true;
  }

  function useExistingIpsw(ipsw: ExistingIpswEntry) {
    logDiag('existing-ipsw-use-clicked', {
      fileName: ipsw.fileName,
      path: ipsw.path,
      sizeBytes: ipsw.sizeBytes,
      fromTargetedFallback: ipswMode === 'targeted',
    });
    selectedExistingPath = ipsw.path;
    onSelect(ipsw.path, null);
    if (ipswMode === 'targeted') {
      forceBrowseInTargeted = false;
      autoMatch = ipsw;
      autoMatchNotified = true;
    }
    toastStore.success('IPSW selected', ipsw.fileName);
  }

  function normalizeVersion(value: string | null): string | null {
    if (!value) return null;
    return value.replace(/^iOS\s+/i, '').trim();
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null) return '—';
    const units = ['B', 'KB', 'MB', 'GB'];
    let v = bytes, u = 0;
    while (v >= 1024 && u < units.length - 1) { v /= 1024; u++; }
    return `${v.toFixed(u === 0 ? 0 : 1)} ${units[u]}`;
  }

  function formatEta(secs: number | null): string {
    if (secs === null) return '—';
    const h = Math.floor(secs / 3600), m = Math.floor((secs % 3600) / 60), s = secs % 60;
    if (h > 0) return `${h}h ${m}m`;
    if (m > 0) return `${m}m ${s}s`;
    return `${s}s`;
  }
</script>

{#if !showBrowseMode && ipswMode === 'targeted'}
  <!-- ── Targeted: auto-select or single download prompt ────────────────── -->
  {#if autoMatch}
    <div class="auto-match">
      <div class="auto-match-body">
        <span class="check-icon" aria-hidden="true">✓</span>
        <div class="auto-match-info">
          <strong>Found in workspace</strong>
          <span class="filename">{autoMatch.fileName}</span>
          <span class="filesize">{formatBytes(autoMatch.sizeBytes)}</span>
        </div>
      </div>
      <button class="ghost" onclick={dismissAutoMatch}>Use different</button>
    </div>
  {:else if isLoadingExisting || (isLoadingFirmwares && !resolvedTargetVersion)}
    <div class="scanning-note" aria-live="polite">
      Scanning workspace{resolvedTargetVersion ? ` for iOS ${normalizeVersion(resolvedTargetVersion)}` : ''}…
    </div>
  {:else if targetFirmware}
    <div class="targeted-prompt">
      <p class="prompt-note">
        iOS {normalizeVersion(resolvedTargetVersion)} not found in workspace — download required.
      </p>
      <div class="firmware-card">
        <div class="fw-meta">
          <strong>iOS {targetFirmware.version}</strong>
          <span class="build">{targetFirmware.buildId}</span>
          <span class="size">{formatBytes(targetFirmware.sizeBytes)}</span>
          {#if targetFirmware.signed !== null}
            <span class="badge" class:signed={targetFirmware.signed} class:unsigned={!targetFirmware.signed}>
              {targetFirmware.signed ? 'Signed' : 'Unsigned'}
            </span>
          {/if}
        </div>
        <button
          class="download-btn"
          onclick={() => startDownload(targetFirmware!)}
          disabled={isDownloading}
        >
          {isDownloading ? 'Downloading…' : 'Download'}
        </button>
      </div>

      {#if isDownloading}
        <div class="progress-card">
          <ProgressBar
            value={downloadProgress?.percent ?? null}
            max={100}
            label="Download progress"
            indeterminate={downloadProgress === null || downloadProgress.percent === null}
          />
          <div class="progress-meta">
            <span>{formatBytes(downloadProgress?.downloadedBytes ?? null)} / {formatBytes(downloadProgress?.totalBytes ?? null)}</span>
            <span>{formatBytes(downloadProgress?.speedBps ?? null)}/s</span>
            <span>ETA {formatEta(downloadProgress?.etaSeconds ?? null)}</span>
            <button class="cancel-inline" onclick={cancelDownload}>Cancel</button>
          </div>
        </div>
      {/if}
    </div>
  {:else if resolvedTargetVersion && !isLoadingFirmwares}
    <p class="not-available-note">
      iOS {normalizeVersion(resolvedTargetVersion)} is not listed in the firmware catalogue for this device.
    </p>
  {/if}

{:else}
  <!-- ── Browse: full workspace + catalogue ─────────────────────────────── -->
  {#if ipswMode === 'targeted'}
    <p class="scanning-note">
      {#if resolvedTargetVersion && !targetFirmware}
        Exact target iOS {normalizeVersion(resolvedTargetVersion)} is unavailable in the catalogue. Choose an existing IPSW or download another available version.
      {:else}
        Choose a different IPSW from workspace or firmware catalogue.
      {/if}
    </p>
  {/if}

  {#if existingIpsws.length > 0}
    <div class="browse-group">
      <h4 class="group-label">In Workspace</h4>
      <div class="existing-list">
        {#each existingIpsws as ipsw}
          <div class="existing-item" class:selected-item={selectedExistingPath === ipsw.path}>
            <div class="existing-meta">
              <span class="filename">{ipsw.fileName}</span>
              <span class="filesize">{formatBytes(ipsw.sizeBytes)}</span>
              {#if selectedExistingPath === ipsw.path}
                <span class="selected-pill">Selected for restore</span>
              {/if}
            </div>
            <button
              class="use-btn"
              class:active={selectedExistingPath === ipsw.path}
              aria-pressed={selectedExistingPath === ipsw.path}
              onclick={() => useExistingIpsw(ipsw)}
            >
              {selectedExistingPath === ipsw.path ? 'Selected' : 'Use'}
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <div class="browse-group">
    <h4 class="group-label">Download from Catalogue</h4>
    <input
      class="search-input"
      bind:value={firmwareSearch}
      placeholder="Filter by iOS or build (e.g. 8.4.1 / 12H321)"
    />
    <p class="dest-note">Destination: workspace <code>ipsw/&lt;device&gt;</code></p>

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
          {#if isLoadingFirmwares}
            <tr><td colspan={4} class="empty-cell">Loading firmware list…</td></tr>
          {:else if filteredFirmwares.length === 0}
            <tr><td colspan={4} class="empty-cell">No firmware entries</td></tr>
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

    <div class="catalogue-actions">
      <button
        onclick={() => selectedFirmware && startDownload(selectedFirmware)}
        disabled={!canDownloadSelected}
      >
        Download selected IPSW
      </button>
      <button class="danger" onclick={cancelDownload} disabled={!isDownloading}>
        Cancel
      </button>
    </div>

    {#if isDownloading}
      <div class="progress-card">
        <ProgressBar
          value={downloadProgress?.percent ?? null}
          max={100}
          label="Download progress"
          indeterminate={downloadProgress === null || downloadProgress.percent === null}
        />
        <div class="progress-meta">
          <span>{formatBytes(downloadProgress?.downloadedBytes ?? null)} / {formatBytes(downloadProgress?.totalBytes ?? null)}</span>
          <span>{formatBytes(downloadProgress?.speedBps ?? null)}/s</span>
          <span>ETA {formatEta(downloadProgress?.etaSeconds ?? null)}</span>
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── auto-match banner ─────────────────────────────────────────────────── */
  .auto-match {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: 12px 14px;
    border: 1px solid color-mix(in srgb, var(--color-accent) 40%, var(--color-border));
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-bg-primary));
    margin-bottom: var(--spacing-md);
  }

  .auto-match-body {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .check-icon {
    flex-shrink: 0;
    font-size: 1.1rem;
    color: var(--color-accent);
  }

  .auto-match-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .auto-match-info strong {
    font-size: 0.85rem;
    color: var(--color-text-primary);
  }

  /* ── scanning / not-available notes ────────────────────────────────────── */
  .scanning-note,
  .not-available-note {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    padding: 8px 0;
    margin-bottom: var(--spacing-sm);
  }

  /* ── targeted download prompt ───────────────────────────────────────────── */
  .targeted-prompt {
    margin-bottom: var(--spacing-md);
  }

  .prompt-note {
    font-size: 0.8rem;
    color: var(--color-text-secondary);
    margin: 0 0 var(--spacing-sm);
  }

  .firmware-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    padding: 12px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-secondary);
  }

  .fw-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    font-size: 0.85rem;
    color: var(--color-text-primary);
  }

  .fw-meta .build {
    color: var(--color-text-secondary);
    font-size: 0.8rem;
  }

  .fw-meta .size {
    color: var(--color-text-secondary);
    font-size: 0.8rem;
  }

  /* ── badge ──────────────────────────────────────────────────────────────── */
  .badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 2px 7px;
    border-radius: 999px;
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary);
  }

  .badge.signed {
    border-color: color-mix(in srgb, var(--color-accent) 50%, var(--color-border));
    color: var(--color-accent);
  }

  .badge.unsigned {
    border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border));
    color: var(--color-danger);
  }

  /* ── download button (targeted card) ────────────────────────────────────── */
  .download-btn {
    flex-shrink: 0;
    background: var(--color-accent);
    color: #fff;
    border: none;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 16px;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .download-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  /* ── progress card ──────────────────────────────────────────────────────── */
  .progress-card {
    margin-top: var(--spacing-sm);
    padding: var(--spacing-sm);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-sm);
    margin-top: 6px;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .cancel-inline {
    background: none;
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-sm);
    color: var(--color-danger);
    font-size: 0.75rem;
    padding: 3px 8px;
    cursor: pointer;
  }

  /* ── browse groups ──────────────────────────────────────────────────────── */
  .browse-group {
    margin-bottom: var(--spacing-md);
  }

  .group-label {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
  }

  /* ── existing list (browse) ─────────────────────────────────────────────── */
  .existing-list {
    display: grid;
    gap: 4px;
    margin-bottom: var(--spacing-sm);
  }

  .existing-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 7px 10px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .existing-item.selected-item {
    border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-bg-secondary));
  }

  .existing-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  /* ── shared filename/size ────────────────────────────────────────────────── */
  .filename {
    font-size: 0.82rem;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .filesize {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  .selected-pill {
    width: fit-content;
    border: 1px solid color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
    border-radius: 999px;
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-bg-primary));
    color: var(--color-accent);
    font-size: 0.68rem;
    font-weight: 600;
    line-height: 1;
    padding: 3px 8px;
  }

  /* ── use / ghost buttons ─────────────────────────────────────────────────── */
  .use-btn,
  .ghost {
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-weight: 600;
    padding: 5px 12px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .use-btn {
    border: 1px solid var(--color-accent);
    background: transparent;
    color: var(--color-accent);
  }

  .use-btn:hover {
    background: var(--color-accent);
    color: #fff;
  }

  .use-btn.active {
    background: var(--color-accent);
    color: #fff;
  }

  .ghost {
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-secondary);
  }

  .ghost:hover {
    color: var(--color-text-primary);
    border-color: var(--color-text-secondary);
  }

  /* ── catalogue ───────────────────────────────────────────────────────────── */
  .search-input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font: inherit;
    font-size: 0.85rem;
    padding: 8px 10px;
    margin-bottom: var(--spacing-sm);
  }

  .dest-note {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.76rem;
    color: var(--color-text-secondary);
  }

  .dest-note code {
    color: var(--color-text-primary);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.74rem;
  }

  .table-wrap {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: auto;
    max-height: 240px;
    margin-bottom: var(--spacing-sm);
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
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-bg-primary));
  }

  .empty-cell {
    color: var(--color-text-secondary);
    text-align: center;
  }

  /* ── catalogue actions ───────────────────────────────────────────────────── */
  .catalogue-actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    margin-bottom: var(--spacing-sm);
  }

  .catalogue-actions button {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 12px;
    cursor: pointer;
  }

  .catalogue-actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .catalogue-actions button.danger {
    background: var(--color-danger);
    color: #fff;
    border-color: var(--color-danger);
  }
</style>
