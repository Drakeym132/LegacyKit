<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { listExistingIpsws, type ExistingIpswEntry } from '$lib/api/restore';
  import { extractIpswMetadata } from '$lib/api/firmware';
  import { toastStore } from '$lib/stores/toastStore.svelte';
  import IpswDownloaderPanel from '../restore/IpswDownloaderPanel.svelte';

  export interface SelectedIpsw {
    path: string;
    fileName: string;
    sizeBytes: number | null;
    buildId: string;
    iosVersion: string | null;
    sha1: string | null;
    supportedProductTypes: string[];
    metadataResolved: boolean;
  }

  export interface RecentBootEntry {
    id: string;
    iosVersion: string | null;
    buildId: string;
    deviceName: string | null;
    productType: string;
    lastBootedAt: string;
  }

  interface Props {
    deviceIdentifier: string | null;
    selected: SelectedIpsw | null;
    isWorking: boolean;
    onSelect: (ipsw: SelectedIpsw) => void;
    onClear: () => void;
    onManualVersion: (version: string) => void;
    recentEntries?: RecentBootEntry[];
    lastBootedId?: string | null;
    isLoadingHistory?: boolean;
    isBootableMode?: boolean;
    onBootRecent?: (entry: RecentBootEntry) => void;
    onForgetRecent?: (entry: RecentBootEntry) => void;
  }

  let {
    deviceIdentifier, selected, isWorking, onSelect, onClear, onManualVersion,
    recentEntries = [], lastBootedId = null, isLoadingHistory = false,
    isBootableMode = false, onBootRecent, onForgetRecent,
  }: Props = $props();

  type Tab = 'recent' | 'download' | 'workspace' | 'browse';
  let activeTab = $state<Tab>('recent');

  function formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const diffMs = Date.now() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  let workspaceIpsws = $state<ExistingIpswEntry[]>([]);
  let workspaceLoading = $state(false);
  let workspaceLoadError = $state<string | null>(null);

  let resolving = $state(false);
  let resolveError = $state<string | null>(null);
  let manualVersionInput = $state('');

  $effect(() => {
    if (activeTab === 'workspace' && deviceIdentifier) {
      void loadWorkspace(deviceIdentifier);
    }
  });

  async function loadWorkspace(device: string) {
    workspaceLoading = true;
    workspaceLoadError = null;
    try {
      const result = await listExistingIpsws({ deviceIdentifier: device });
      workspaceIpsws = result.ipsws;
    } catch (err) {
      workspaceLoadError = err instanceof Error ? err.message : String(err);
      workspaceIpsws = [];
    } finally {
      workspaceLoading = false;
    }
  }

  function fileNameOf(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
  }

  function deriveFallbackBuildId(path: string): string {
    const fileName = fileNameOf(path).replace(/\.ipsw$/i, '');
    const match = fileName.match(/[0-9]+[A-Z][0-9]+[A-Za-z]?/);
    return match ? match[0] : fileName || 'unknown';
  }

  async function pathChosen(path: string, sha1: string | null, sizeBytes: number | null) {
    resolving = true;
    resolveError = null;
    manualVersionInput = '';
    try {
      const metadata = await extractIpswMetadata(path);
      onSelect({
        path,
        fileName: fileNameOf(path),
        sizeBytes,
        buildId: metadata.buildId,
        iosVersion: metadata.iosVersion,
        sha1,
        supportedProductTypes: metadata.supportedProductTypes,
        metadataResolved: true,
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      resolveError = msg;
      onSelect({
        path,
        fileName: fileNameOf(path),
        sizeBytes,
        buildId: deriveFallbackBuildId(path),
        iosVersion: null,
        sha1,
        supportedProductTypes: [],
        metadataResolved: false,
      });
    } finally {
      resolving = false;
    }
  }

  async function handleBrowse() {
    try {
      const choice = await openDialog({ filters: [{ name: 'IPSW', extensions: ['ipsw'] }] });
      if (typeof choice === 'string') {
        await pathChosen(choice, null, null);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toastStore.error('Browse failed', msg);
    }
  }

  function handleClear() {
    resolveError = null;
    manualVersionInput = '';
    onClear();
  }

  function formatBytes(bytes: number | null): string {
    if (bytes === null || bytes === undefined) return '';
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
  }

  function commitManualVersion() {
    const trimmed = manualVersionInput.trim();
    if (trimmed) onManualVersion(trimmed);
  }

</script>

<div class="picker">
  <div class="tabs" role="tablist" aria-label="IPSW source">
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'recent'}
      class:active={activeTab === 'recent'}
      onclick={() => (activeTab = 'recent')}
      disabled={isWorking}
    >
      Recent
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'download'}
      class:active={activeTab === 'download'}
      onclick={() => (activeTab = 'download')}
      disabled={isWorking}
    >
      Download
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'workspace'}
      class:active={activeTab === 'workspace'}
      onclick={() => (activeTab = 'workspace')}
      disabled={isWorking}
    >
      Workspace
    </button>
    <button
      type="button"
      role="tab"
      aria-selected={activeTab === 'browse'}
      class:active={activeTab === 'browse'}
      onclick={() => (activeTab = 'browse')}
      disabled={isWorking}
    >
      Browse
    </button>
  </div>

  <div class="tab-body" class:embedded={activeTab === 'download'}>
    {#if activeTab === 'recent'}
      {#if isLoadingHistory}
        <div class="placeholder-row">Loading…</div>
      {:else if recentEntries.length === 0}
        <div class="placeholder-row">No previous boots — pick a build below.</div>
      {:else}
        <ul class="history-list">
          {#each recentEntries as entry (entry.id)}
            <li class="history-item">
              <div class="history-info">
                <span class="history-version">
                  iOS {entry.iosVersion ?? '?'} <span class="history-build">· {entry.buildId}</span>
                </span>
                <span class="history-sub">
                  {entry.deviceName ?? entry.productType} · {formatRelativeTime(entry.lastBootedAt)}
                  {#if entry.id === lastBootedId} · <span class="last-badge">Last booted</span>{/if}
                </span>
              </div>
              <div class="history-actions">
                <button
                  class="hist-boot"
                  onclick={() => onBootRecent?.(entry)}
                  disabled={isWorking || !isBootableMode}
                >
                  Boot
                </button>
                <button
                  class="hist-forget"
                  onclick={() => onForgetRecent?.(entry)}
                  disabled={isWorking}
                  aria-label="Forget"
                  title="Forget"
                >
                  ✕
                </button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    {:else if activeTab === 'download'}
      {#if !deviceIdentifier}
        <div class="hint">Connect a device to list compatible firmwares.</div>
      {:else}
        <div class="downloader-host">
          <IpswDownloaderPanel
            {deviceIdentifier}
            onUseIpsw={(path, sha1) => pathChosen(path, sha1, null)}
          />
        </div>
      {/if}
    {:else if activeTab === 'workspace'}
      {#if !deviceIdentifier}
        <div class="hint">Connect a device to scan its workspace folder.</div>
      {:else if workspaceLoading}
        <div class="hint">Loading…</div>
      {:else if workspaceLoadError}
        <div class="error-inline">{workspaceLoadError}</div>
      {:else if workspaceIpsws.length === 0}
        <div class="placeholder-row">No IPSWs in workspace for this device. Try Download or Browse.</div>
      {:else}
        <div class="workspace-list">
          {#each workspaceIpsws as ipsw (ipsw.path)}
            <button
              type="button"
              class="workspace-item"
              onclick={() => pathChosen(ipsw.path, null, ipsw.sizeBytes)}
              disabled={isWorking || resolving}
            >
              <span class="ws-name">{ipsw.fileName}</span>
              <span class="ws-size">{formatBytes(ipsw.sizeBytes)}</span>
            </button>
          {/each}
        </div>
      {/if}
    {:else}
      {#if resolving}
        <div class="placeholder-row">Reading IPSW metadata…</div>
      {:else if selected}
        <div class="browse-inline" class:warning={!selected.metadataResolved}>
          <div class="browse-file-info">
            <span class="browse-filename">{selected.fileName}</span>
            <span class="browse-meta">
              {#if selected.metadataResolved}
                iOS {selected.iosVersion} · {selected.buildId}
                {#if selected.sizeBytes}· {formatBytes(selected.sizeBytes)}{/if}
                {#if selected.sha1}· <span class="ok">✓ verified</span>{/if}
              {:else}
                <span class="warn-text">Couldn't read BuildManifest — enter iOS version below.</span>
              {/if}
            </span>
          </div>
          <button type="button" class="ghost" onclick={() => { handleClear(); void handleBrowse(); }} disabled={isWorking}>Change</button>
        </div>
        {#if !selected.metadataResolved}
          <label class="manual-field">
            <span>iOS version</span>
            <input
              type="text"
              bind:value={manualVersionInput}
              oninput={commitManualVersion}
              placeholder="e.g. 9.3.5"
              disabled={isWorking}
            />
          </label>
          {#if resolveError}
            <p class="error-detail">{resolveError}</p>
          {/if}
        {/if}
      {:else}
        <div class="placeholder-row browse-retry" role="button" tabindex="0" onclick={() => void handleBrowse()} onkeydown={(e) => e.key === 'Enter' && void handleBrowse()}>
          No file selected — click to browse
        </div>
      {/if}
    {/if}
  </div>

  {#if activeTab !== 'browse'}
    {#if resolving}
      <div class="chip resolving">Reading IPSW metadata…</div>
    {:else if selected}
      <div class="chip" class:warning={!selected.metadataResolved}>
        <div class="chip-main">
          <span class="chip-name">{selected.fileName}</span>
          <span class="chip-meta">
            {#if selected.metadataResolved}
              iOS {selected.iosVersion} · {selected.buildId}
              {#if selected.sizeBytes}· {formatBytes(selected.sizeBytes)}{/if}
              {#if selected.sha1}· <span class="ok">✓ verified</span>{/if}
            {:else}
              <span class="warn-text">Couldn't read BuildManifest — enter iOS version below.</span>
            {/if}
          </span>
        </div>
        <button type="button" class="ghost" onclick={handleClear} disabled={isWorking}>Change</button>
      </div>

      {#if !selected.metadataResolved}
        <label class="manual-field">
          <span>iOS version</span>
          <input
            type="text"
            bind:value={manualVersionInput}
            oninput={commitManualVersion}
            placeholder="e.g. 9.3.5"
            disabled={isWorking}
          />
        </label>
        {#if resolveError}
          <p class="error-detail">{resolveError}</p>
        {/if}
      {/if}
    {/if}
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .tabs {
    display: flex;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
  }

  .tabs button {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: calc(var(--radius-sm) - 2px);
    padding: 5px 12px;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    cursor: pointer;
    flex: 1;
    text-align: center;
  }

  .tabs button.active {
    background: var(--color-bg-primary);
    border-color: var(--color-border);
    color: var(--color-text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
  }

  .tabs button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-body {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    min-height: 60px;
  }

  .tab-body.embedded {
    padding: 0;
    overflow: hidden;
  }

  .downloader-host {
  }

  .placeholder-row {
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    padding: 10px 12px;
    background: var(--color-bg-secondary);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-sm);
    text-align: center;
    line-height: 1.4;
  }

  .placeholder-row.browse-retry {
    cursor: pointer;
  }

  .placeholder-row.browse-retry:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .hint {
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    line-height: 1.4;
  }

  .error-inline {
    color: var(--color-danger);
    font-size: 0.8125rem;
  }

  .workspace-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .workspace-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 7px 10px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
  }

  .workspace-item:hover:not(:disabled) {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-bg-primary));
  }

  .workspace-item:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .ws-name {
    font-size: 0.82rem;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
  }

  .ws-size {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }

  .browse-inline {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 8px 12px;
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-bg-primary));
    border: 1px solid color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    border-radius: var(--radius-sm);
  }

  .browse-inline.warning {
    background: color-mix(in srgb, var(--color-warning) 10%, var(--color-bg-primary));
    border-color: color-mix(in srgb, var(--color-warning) 35%, var(--color-border));
  }

  .browse-file-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .browse-filename {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .browse-meta {
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .browse-meta .ok {
    color: var(--color-success);
    font-weight: 600;
  }

  .browse-meta .warn-text {
    color: var(--color-warning);
  }

  .chip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 8px 12px;
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-bg-primary));
    border: 1px solid color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    border-radius: var(--radius-sm);
  }

  .chip.warning {
    background: color-mix(in srgb, var(--color-warning) 10%, var(--color-bg-primary));
    border-color: color-mix(in srgb, var(--color-warning) 35%, var(--color-border));
  }

  .chip.resolving {
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
    font-style: italic;
  }

  .chip-main {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }

  .chip-name {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip-meta {
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }

  .chip-meta .ok {
    color: var(--color-success);
    font-weight: 600;
  }

  .chip-meta .warn-text {
    color: var(--color-warning);
  }

  .ghost {
    appearance: none;
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-secondary);
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .ghost:hover:not(:disabled) {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .manual-field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 0.8125rem;
  }

  .manual-field span {
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .manual-field input {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    padding: 6px 10px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.8125rem;
  }

  .manual-field input:disabled {
    opacity: 0.6;
  }

  .error-detail {
    margin: 0;
    color: var(--color-danger);
    font-size: 0.75rem;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    line-height: 1.4;
    word-break: break-word;
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
    background: var(--color-bg-primary);
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

  .hist-boot {
    border: 1px solid var(--color-accent);
    background: var(--color-accent);
    color: white;
    border-radius: var(--radius-sm);
    padding: 5px 12px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
  }

  .hist-boot:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .hist-forget {
    appearance: none;
    background: color-mix(in srgb, var(--color-danger) 12%, var(--color-bg-primary));
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, var(--color-border));
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    padding: 5px 10px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    line-height: 1;
  }

  .hist-forget:hover:not(:disabled) {
    background: var(--color-danger);
    border-color: var(--color-danger);
    color: #fff;
  }

  .hist-forget:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
