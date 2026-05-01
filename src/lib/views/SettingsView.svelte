<script lang="ts">
  import { settingsStore } from '$lib/stores/settingsStore.svelte';
  import { checkForUpdates, type UpdateCheckResult } from '$lib/api/updates';
  import { toastStore } from '$lib/stores/toastStore.svelte';

  const APP_VERSION = '0.1.0';
  const UPDATE_REPO = 'Drakeym132/LegacyKit';

  let isCheckingUpdates = $state(false);
  let updateResult = $state<UpdateCheckResult | null>(null);

  async function handleCheckUpdates() {
    isCheckingUpdates = true;
    try {
      updateResult = await checkForUpdates({
        repo: UPDATE_REPO,
        currentVersion: APP_VERSION,
      });
      if (updateResult.updateAvailable) {
        toastStore.info(
          `Update available: ${updateResult.latest}`,
          updateResult.releaseUrl ?? null,
        );
      } else {
        toastStore.success('You are on the latest version', `v${updateResult.current}`);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      toastStore.error('Update check failed', msg);
    } finally {
      isCheckingUpdates = false;
    }
  }

  function handleThemeChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    settingsStore.setTheme(target.value as 'system' | 'light' | 'dark');
  }

  function handleTerminalToggle() {
    settingsStore.toggleTerminal();
  }

  function handleAutoDetectToggle() {
    settingsStore.autoDetectDevice = !settingsStore.autoDetectDevice;
  }

  function handleTerminalHeightChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.setTerminalHeight(Number(target.value));
  }

  function handlePollIntervalChange(event: Event) {
    const target = event.target as HTMLInputElement;
    settingsStore.setPollInterval(Number(target.value));
  }
</script>

<div class="view">
  <div class="view-header">
    <h1>Settings</h1>
  </div>

  <div class="settings-group">
    <h3>Appearance</h3>
    <div class="setting-row">
      <div class="setting-info">
        <label for="theme-select">Theme</label>
        <span class="setting-hint">Choose how LegacyKit appears</span>
      </div>
      <select
        id="theme-select"
        value={settingsStore.theme}
        onchange={handleThemeChange}
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </div>
  </div>

  <div class="settings-group">
    <h3>Terminal</h3>
    <div class="setting-row">
      <div class="setting-info">
        <label for="terminal-toggle">Show Terminal</label>
        <span class="setting-hint">Display the log terminal panel</span>
      </div>
      <label class="toggle">
        <input
          id="terminal-toggle"
          type="checkbox"
          checked={settingsStore.terminalVisible}
          onchange={handleTerminalToggle}
        />
        <span class="toggle-slider"></span>
      </label>
    </div>
    <div class="setting-row">
      <div class="setting-info">
        <label for="terminal-height">Terminal Height</label>
        <span class="setting-hint">Adjust the terminal panel size</span>
      </div>
      <div class="range-control">
        <input
          id="terminal-height"
          type="range"
          min="100"
          max="600"
          step="20"
          value={settingsStore.terminalHeight}
          oninput={handleTerminalHeightChange}
        />
        <span>{settingsStore.terminalHeight}px</span>
      </div>
    </div>
  </div>

  <div class="settings-group">
    <h3>Device Detection</h3>
    <div class="setting-row">
      <div class="setting-info">
        <label for="auto-detect-toggle">Auto-Detect Device</label>
        <span class="setting-hint">Automatically detect connected devices</span>
      </div>
      <label class="toggle">
        <input
          id="auto-detect-toggle"
          type="checkbox"
          checked={settingsStore.autoDetectDevice}
          onchange={handleAutoDetectToggle}
        />
        <span class="toggle-slider"></span>
      </label>
    </div>
    <div class="setting-row">
      <div class="setting-info">
        <label for="poll-interval">Poll Interval (ms)</label>
        <span class="setting-hint">How often to check for device changes (min: 1000ms)</span>
      </div>
      <input
        id="poll-interval"
        type="number"
        min="1000"
        step="500"
        value={settingsStore.pollIntervalMs}
        onchange={handlePollIntervalChange}
      />
    </div>
  </div>

  <div class="settings-group">
    <h3>Updates</h3>
    <div class="setting-row">
      <div class="setting-info">
        <label for="update-check">Check for updates</label>
        <span class="setting-hint">Compares against the latest GitHub release</span>
      </div>
      <button
        id="update-check"
        class="update-button"
        onclick={handleCheckUpdates}
        disabled={isCheckingUpdates}
      >
        {isCheckingUpdates ? 'Checking…' : 'Check now'}
      </button>
    </div>
    {#if updateResult}
      <div class="update-result" data-state={updateResult.updateAvailable ? 'available' : 'current'}>
        {#if updateResult.updateAvailable}
          <strong>Update available</strong>
          <p>You are on v{updateResult.current}. Latest is v{updateResult.latest}.</p>
          {#if updateResult.releaseUrl}
            <p><a href={updateResult.releaseUrl} target="_blank" rel="noopener">Open release page</a></p>
          {/if}
        {:else}
          <strong>You are up to date</strong>
          <p>v{updateResult.current} matches the latest release.</p>
        {/if}
      </div>
    {/if}
  </div>

  <div class="settings-group about-section">
    <h3>About</h3>
    <div class="about-card">
      <p class="app-name">LegacyKit <span class="version">v{APP_VERSION}</span></p>
      <p class="about-description">A modern toolkit for managing legacy iOS devices. Restore, jailbreak, manage SHSH blobs, and more — all from a single native application.</p>
    </div>
  </div>
</div>

<style>
  .view { padding: var(--spacing-xl); max-width: 640px; }
  .view-header { margin-bottom: var(--spacing-lg); }
  .view-header h1 { font-size: 1.5rem; font-weight: 700; color: var(--color-text-primary); margin: 0; }

  .settings-group {
    margin-bottom: var(--spacing-lg);
  }
  .settings-group h3 {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--spacing-sm);
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
    margin-bottom: 1px;
  }
  .setting-row:first-of-type { border-radius: var(--radius-md) var(--radius-md) 0 0; }
  .setting-row:last-of-type { border-radius: 0 0 var(--radius-md) var(--radius-md); margin-bottom: 0; }
  .setting-row:only-of-type { border-radius: var(--radius-md); }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .setting-info label {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-text-primary);
    cursor: default;
  }
  .setting-hint {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  select {
    appearance: none;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-xs) var(--spacing-md) var(--spacing-xs) var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-primary);
    cursor: pointer;
    min-width: 120px;
    font-family: inherit;
  }
  select:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.2);
  }

  input[type="number"] {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-xs) var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-primary);
    width: 100px;
    text-align: right;
    font-family: inherit;
  }
  input[type="number"]:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.2);
  }

  .range-control {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    min-width: 180px;
  }
  .range-control input[type="range"] {
    flex: 1;
    accent-color: var(--color-accent);
  }
  .range-control span {
    width: 44px;
    text-align: right;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  /* macOS-style toggle switch */
  .toggle {
    position: relative;
    display: inline-block;
    width: 40px;
    height: 24px;
    cursor: pointer;
  }
  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--color-border);
    border-radius: 12px;
    transition: background-color 0.2s ease;
  }
  .toggle-slider::before {
    content: "";
    position: absolute;
    height: 20px;
    width: 20px;
    left: 2px;
    bottom: 2px;
    background: white;
    border-radius: 50%;
    transition: transform 0.2s ease;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }
  .toggle input:checked + .toggle-slider {
    background: var(--color-accent);
  }
  .toggle input:checked + .toggle-slider::before {
    transform: translateX(16px);
  }
  .toggle input:focus-visible + .toggle-slider {
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.3);
  }

  .about-card {
    padding: var(--spacing-md);
    background: var(--color-bg-secondary);
    border-radius: var(--radius-md);
  }
  .app-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 var(--spacing-xs) 0;
  }
  .version {
    font-weight: 400;
    color: var(--color-text-secondary);
    font-size: 0.85rem;
  }
  .about-description {
    font-size: 0.85rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .update-button {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text-primary);
    cursor: pointer;
  }
  .update-button:disabled { opacity: 0.5; cursor: not-allowed; }
  .update-button:hover:not(:disabled) { border-color: var(--color-accent); color: var(--color-accent); }

  .update-result {
    margin-top: 8px;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    font-size: 0.85rem;
  }
  .update-result strong { color: var(--color-text-primary); display: block; }
  .update-result p { color: var(--color-text-secondary); margin: 4px 0 0; }
  .update-result[data-state="available"] strong { color: var(--color-accent); }
  .update-result[data-state="current"] strong { color: var(--color-success); }
  .update-result a { color: var(--color-accent); text-decoration: none; }
  .update-result a:hover { text-decoration: underline; }
</style>
