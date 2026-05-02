<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    clearNvram,
    enterRecovery,
    exitRecovery,
    exportDeviceInfo,
    onSyslogEvent,
    pairDevice,
    runActivationAction,
    runDiagnosticsAction,
    runIrecoveryCommands,
    startSyslog,
    stopSyslog,
    syslogStatus,
    type ActivationAction,
    type DiagnosticsAction,
    type ExportInfoKind,
    type PairAction,
    type SyslogEvent,
  } from '$lib/api/utilities';
  import {
    checkTrollstoreEligibility,
    prepareTrollstoreAssets,
    type TrollStoreEligibilityResult,
    type TrollStorePrepareResult,
  } from '$lib/api/trollstore';
  import { deviceStore } from '$lib/stores/deviceStore.svelte';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { createWorkingController } from '$lib/utils/workingState.svelte';

  type TabId = 'actions' | 'irecovery' | 'syslog' | 'export' | 'trollstore';
  let activeTab = $state<TabId>('actions');

  let trollstoreSavedDir = $state('');
  let trollstoreEligibility = $state<TrollStoreEligibilityResult | null>(null);
  let trollstoreAssets = $state<TrollStorePrepareResult | null>(null);

  let device = $derived(deviceStore.state);
  const work = createWorkingController();

  let exportDir = $state('');
  let exportLabel = $state('');
  let lastExportPath = $state<string | null>(null);

  let irecoveryRaw = $state('setenv auto-boot true\nsaveenv');
  let rebootAfter = $state(true);

  let syslogRunning = $state(false);
  let syslogPid = $state<number | null>(null);
  let syslogLines = $state<SyslogEvent[]>([]);
  let syslogUnlisten: (() => void) | null = null;
  const SYSLOG_MAX_LINES = 500;

  $effect(() => {
    void syslogStatus().then((s) => {
      syslogRunning = s.running;
      syslogPid = s.pid;
    });
  });

  onDestroy(() => {
    syslogUnlisten?.();
  });

  function udid(): string | null {
    return device.udid ?? null;
  }

  async function handleEnterRecovery() {
    if (!device.udid) {
      work.setError('Pair the device in Normal mode first.');
      return;
    }
    if (!confirm('Send the device to Recovery mode?')) return;
    await work.run('Enter recovery', () => enterRecovery({ udid: device.udid }));
  }

  async function handleExitRecovery() {
    await work.run('Exit recovery (irecovery -n)', () => exitRecovery());
  }

  async function handleDiagnostics(action: DiagnosticsAction) {
    const verb = action[0].toUpperCase() + action.slice(1);
    if (!confirm(`${verb} the device now?`)) return;
    await work.run(`Diagnostics ${action}`, () =>
      runDiagnosticsAction({ udid: udid(), action }),
    );
  }

  async function handlePair(action: PairAction) {
    await work.run(`idevicepair ${action}`, () =>
      pairDevice({ udid: udid(), action }),
    );
  }

  async function handleActivation(action: ActivationAction) {
    if (action === 'deactivate' && !confirm('Deactivate the device? This requires re-activation.')) {
      return;
    }
    const result = await work.run(`Activation ${action}`, () =>
      runActivationAction({ udid: udid(), action }),
    );
    if (result?.state) {
      logStore.append(`Activation state: ${result.state}`, 'info');
    }
  }

  async function handleExport(kind: ExportInfoKind) {
    if (!exportDir.trim()) {
      work.setError('Pick an output directory first.');
      return;
    }
    const result = await work.run(`Export ${kind}`, () =>
      exportDeviceInfo({
        udid: udid(),
        outputDir: exportDir.trim(),
        kind,
        label: exportLabel.trim() || null,
      }),
    );
    if (result) {
      lastExportPath = result.path;
    }
  }

  async function handleClearNvram() {
    if (device.mode !== 'Recovery' && device.mode !== 'DFU') {
      const ok = confirm(
        'Device is not in Recovery mode. Clear NVRAM anyway? (irecovery requires Recovery mode.)',
      );
      if (!ok) return;
    }
    await work.run('Clear NVRAM', () => clearNvram());
  }

  async function handleRunIrecovery() {
    const commands = irecoveryRaw
      .split(/\r?\n/)
      .map((s) => s.trim())
      .filter(Boolean);
    if (commands.length === 0) {
      work.setError('Provide at least one irecovery command (one per line).');
      return;
    }
    await work.run(`irecovery (${commands.length} cmds)`, () =>
      runIrecoveryCommands({ commands, rebootAfter }),
    );
  }

  async function handleStartSyslog() {
    if (syslogRunning) return;
    syslogLines = [];
    if (!syslogUnlisten) {
      syslogUnlisten = await onSyslogEvent((event) => {
        syslogLines = [...syslogLines.slice(-(SYSLOG_MAX_LINES - 1)), event];
      });
    }
    const result = await work.run('Start syslog', () =>
      startSyslog({ udid: udid() }),
    );
    if (result) {
      syslogRunning = result.running;
      syslogPid = result.pid;
    }
  }

  async function handleStopSyslog() {
    const result = await work.run('Stop syslog', () => stopSyslog());
    if (result) {
      syslogRunning = result.running;
      syslogPid = result.pid;
    }
  }

  function clearSyslog() {
    syslogLines = [];
  }

  async function handleCheckEligibility() {
    const result = await work.run('Check TrollStore eligibility', () =>
      checkTrollstoreEligibility({
        productType: device.product_type ?? null,
        iosVersion: device.ios_version ?? null,
      }),
    );
    if (result) {
      trollstoreEligibility = result;
    }
  }

  async function handlePrepareAssets() {
    if (!trollstoreSavedDir.trim()) {
      work.setError('Pick a saved directory for TrollStore assets first.');
      return;
    }
    const result = await work.run('Prepare TrollStore assets', () =>
      prepareTrollstoreAssets({
        savedDir: trollstoreSavedDir.trim(),
      }),
    );
    if (result) {
      trollstoreAssets = result;
    }
  }
</script>

<div class="view">
  <div class="view-header">
    <div>
      <h1>Utilities</h1>
      <p>
        Recovery, activation, diagnostics export, NVRAM, and live syslog. These wrap
        <code>idevice*</code> and <code>irecovery</code> binaries.
      </p>
    </div>
  </div>

  <section class="device-summary">
    <div>
      <span class="label">Device</span>
      <strong>{device.product_type ?? 'Not detected'}</strong>
    </div>
    <div>
      <span class="label">Mode</span>
      <strong class="mode" data-mode={device.mode}>{device.mode}</strong>
    </div>
    <div>
      <span class="label">iOS</span>
      <strong>{device.ios_version ?? 'Unknown'}</strong>
    </div>
    <div>
      <span class="label">UDID</span>
      <strong title={device.udid ?? ''}
        >{device.udid ? `${device.udid.slice(0, 12)}…` : 'Unknown'}</strong
      >
    </div>
  </section>

  {#if work.errorMessage}
    <div class="error-state">{work.errorMessage}</div>
  {/if}

  <div class="tabs" role="tablist">
    <button
      class:active={activeTab === 'actions'}
      onclick={() => (activeTab = 'actions')}
      role="tab">Quick actions</button
    >
    <button
      class:active={activeTab === 'irecovery'}
      onclick={() => (activeTab = 'irecovery')}
      role="tab">irecovery / NVRAM</button
    >
    <button
      class:active={activeTab === 'syslog'}
      onclick={() => (activeTab = 'syslog')}
      role="tab">Syslog</button
    >
    <button
      class:active={activeTab === 'export'}
      onclick={() => (activeTab = 'export')}
      role="tab">Diagnostics export</button
    >
    <button
      class:active={activeTab === 'trollstore'}
      onclick={() => (activeTab = 'trollstore')}
      role="tab">TrollStore</button
    >
  </div>

  {#if activeTab === 'actions'}
    <section class="panel">
      <div class="section-title"><span>1</span><h2>Mode</h2></div>
      <div class="action-grid">
        <button onclick={handleEnterRecovery} disabled={work.isWorking || device.mode !== 'Normal'}>
          Enter Recovery
          <small>From Normal mode</small>
        </button>
        <button onclick={handleExitRecovery} disabled={work.isWorking}>
          Exit Recovery
          <small>irecovery -n (reboot)</small>
        </button>
        <button onclick={() => handleDiagnostics('shutdown')} disabled={work.isWorking || device.mode !== 'Normal'}>
          Shutdown
        </button>
        <button onclick={() => handleDiagnostics('restart')} disabled={work.isWorking || device.mode !== 'Normal'}>
          Restart
        </button>
        <button onclick={() => handleDiagnostics('sleep')} disabled={work.isWorking || device.mode !== 'Normal'}>
          Sleep
        </button>
      </div>
    </section>

    <section class="panel">
      <div class="section-title"><span>2</span><h2>Pairing</h2></div>
      <div class="action-grid">
        <button onclick={() => handlePair('pair')} disabled={work.isWorking}>
          Pair (Trust)
        </button>
        <button onclick={() => handlePair('validate')} disabled={work.isWorking}>
          Validate pairing
        </button>
        <button class="danger" onclick={() => handlePair('unpair')} disabled={work.isWorking}>
          Unpair
        </button>
      </div>
    </section>

    <section class="panel">
      <div class="section-title"><span>3</span><h2>Activation</h2></div>
      <p class="panel-note">
        For iPhone 4 / older devices a valid SIM card is usually required to activate.
        For hacktivation (iOS ≤ 6) use <em>Restore/Downgrade</em> with the Hacktivate flag,
        or <em>Jailbreak → Hacktivate</em>.
      </p>
      <div class="action-grid">
        <button onclick={() => handleActivation('activate')} disabled={work.isWorking}>
          Attempt activation
        </button>
        <button onclick={() => handleActivation('state')} disabled={work.isWorking}>
          Show state
        </button>
        <button class="danger" onclick={() => handleActivation('deactivate')} disabled={work.isWorking}>
          Deactivate
        </button>
      </div>
    </section>
  {/if}

  {#if activeTab === 'irecovery'}
    <section class="panel">
      <div class="section-title"><span>1</span><h2>Run irecovery commands</h2></div>
      <p class="panel-note">
        One command per line. Device must be in Recovery (or DFU) mode.
        Common patterns: <code>setenv auto-boot true</code>, <code>saveenv</code>,
        <code>fsboot</code>, <code>reset</code>.
      </p>
      <label class="field">
        <span>Commands</span>
        <textarea rows="5" bind:value={irecoveryRaw}></textarea>
      </label>
      <label class="checkbox">
        <input type="checkbox" bind:checked={rebootAfter} />
        Reboot device after running (<code>irecovery -n</code>)
      </label>
      <div class="actions">
        <button class="primary" onclick={handleRunIrecovery} disabled={work.isWorking}>
          {work.isWorking ? 'Working…' : 'Run'}
        </button>
      </div>
    </section>

    <section class="panel">
      <div class="section-title"><span>2</span><h2>Clear NVRAM</h2></div>
      <p class="panel-note">
        Sets <code>auto-boot true</code> and saves. Recovers from
        <em>auto-boot=false</em> states left by tethered jailbreaks.
      </p>
      <div class="actions">
        <button class="danger" onclick={handleClearNvram} disabled={work.isWorking}>
          Clear NVRAM
        </button>
      </div>
    </section>
  {/if}

  {#if activeTab === 'syslog'}
    <section class="panel">
      <div class="section-title"><span>1</span><h2>Live syslog</h2></div>
      <p class="panel-note">
        Streams <code>idevicesyslog -q</code>. Last {SYSLOG_MAX_LINES} lines are
        retained in this view; the global log captures everything.
      </p>
      <div class="row">
        {#if !syslogRunning}
          <button class="primary" onclick={handleStartSyslog} disabled={work.isWorking}>
            {work.isWorking ? 'Working…' : 'Start'}
          </button>
        {:else}
          <button class="danger" onclick={handleStopSyslog} disabled={work.isWorking}>
            {work.isWorking ? 'Working…' : 'Stop'}
          </button>
          <span class="status-pill" data-state="running">
            running{syslogPid ? ` · pid ${syslogPid}` : ''}
          </span>
        {/if}
        <button class="secondary" onclick={clearSyslog} disabled={syslogLines.length === 0}>
          Clear
        </button>
      </div>

      <div class="syslog">
        {#if syslogLines.length === 0}
          <div class="syslog-empty">No syslog lines yet.</div>
        {:else}
          {#each syslogLines as line, i (i)}
            <div class="syslog-line" data-kind={line.type}>{line.text}</div>
          {/each}
        {/if}
      </div>
    </section>
  {/if}

  {#if activeTab === 'export'}
    <section class="panel">
      <div class="section-title"><span>1</span><h2>Export device diagnostics</h2></div>
      <p class="panel-note">
        Saves a timestamped <code>.txt</code> in the chosen directory. Useful for bug
        reports.
      </p>
      <label class="field">
        <span>Output directory (absolute)</span>
        <input bind:value={exportDir} placeholder="/Users/you/legacykit/exports" />
      </label>
      <label class="field">
        <span>Optional label prefix</span>
        <input bind:value={exportLabel} placeholder="device-info" />
      </label>
      <div class="actions">
        <button onclick={() => handleExport('device-info')} disabled={work.isWorking}>
          ideviceinfo
        </button>
        <button onclick={() => handleExport('battery-info')} disabled={work.isWorking}>
          Battery (AppleSmartBattery)
        </button>
        <button onclick={() => handleExport('diagnostics-all')} disabled={work.isWorking}>
          Diagnostics All
        </button>
      </div>

      {#if lastExportPath}
        <p class="footer-note">
          Last export → <code class="wrap">{lastExportPath}</code>
        </p>
      {/if}
    </section>
  {/if}

  {#if activeTab === 'trollstore'}
    <section class="panel">
      <div class="section-title"><span>1</span><h2>Eligibility</h2></div>
      <p class="panel-note">
        Determines the right install path based on connected device's iOS major version.
        TrollStore needs iOS 14+. iOS 14/15 install via SSH ramdisk; iOS 16+ uses TrollRestore.
      </p>
      <div class="actions">
        <button class="primary" onclick={handleCheckEligibility} disabled={work.isWorking}>
          {work.isWorking ? 'Working…' : 'Check this device'}
        </button>
      </div>
      {#if trollstoreEligibility}
        <div class="elig" data-path={trollstoreEligibility.path}>
          <strong>
            {#if trollstoreEligibility.path === 'ios14-15-ramdisk'}
              ✓ Use SSH ramdisk path
            {:else if trollstoreEligibility.path === 'ios16-trollrestore'}
              ✓ Use TrollRestore path
            {:else if trollstoreEligibility.path === 'incompatible'}
              ✗ Not compatible
            {:else}
              ? Unknown
            {/if}
          </strong>
          <p>{trollstoreEligibility.reason}</p>
          {#if trollstoreEligibility.iosMajor}
            <small>iOS major: {trollstoreEligibility.iosMajor}</small>
          {/if}
        </div>
      {/if}
    </section>

    <section class="panel">
      <div class="section-title"><span>2</span><h2>Asset preparation</h2></div>
      <p class="panel-note">
        Downloads <code>TrollStore.tar</code> + <code>PersistenceHelper_Embedded</code> from the
        latest <code>opa334/TrollStore</code> GitHub release into the saved directory.
        Cached if version stamp matches.
      </p>
      <label class="field">
        <span>Saved directory (absolute)</span>
        <input bind:value={trollstoreSavedDir} placeholder="/Users/you/legacykit/saved" />
      </label>
      <div class="actions">
        <button class="primary" onclick={handlePrepareAssets} disabled={work.isWorking}>
          {work.isWorking ? 'Working…' : 'Download / refresh assets'}
        </button>
      </div>
      {#if trollstoreAssets}
        <div class="elig" data-path="ready">
          <strong
            >TrollStore {trollstoreAssets.version} ready{trollstoreAssets.cached ? ' (cached)' : ''}</strong
          >
          <p>tar → <code class="wrap">{trollstoreAssets.tarPath}</code></p>
          <p>helper → <code class="wrap">{trollstoreAssets.helperPath}</code></p>
        </div>
      {/if}
    </section>

    <section class="panel">
      <div class="section-title"><span>3</span><h2>Install (manual)</h2></div>
      <p class="panel-note">
        Live install orchestration is intentionally a manual step until the SSH ramdisk
        runtime session lands. Once you have the assets ready, follow the path below.
      </p>
      <details>
        <summary>iOS 14/15 — via SSH ramdisk</summary>
        <ol class="howto">
          <li>Boot SSH ramdisk from the <strong>SSH Ramdisk</strong> view.</li>
          <li>Open a terminal and connect: <code>iproxy 2222 22</code> then <code>ssh -p 2222 root@127.0.0.1</code> (default password <code>alpine</code>).</li>
          <li>Run <code>mount_filesystems</code> on the device.</li>
          <li>Find Tips.app: <code>find /mnt2/containers/Bundle/Application/ -name 'Tips.app'</code>.</li>
          <li>From your Mac/Linux: <code>scp -P 2222 TrollStore.tar PersistenceHelper_Embedded TrollStore.app/trollstorehelper root@127.0.0.1:&lt;TIPS_PATH&gt;</code>.</li>
          <li>On the device: <code>cd &lt;TIPS_PATH&gt; && ./trollstorehelper install-trollstore TrollStore.tar</code>.</li>
          <li>Reboot the device — TrollStore should appear after Tips opens.</li>
        </ol>
      </details>
      <details>
        <summary>iOS 16+ — via TrollRestore</summary>
        <ol class="howto">
          <li>Ensure Python 3 is installed on this host.</li>
          <li>Pair the device (Utilities → Quick actions → Pair).</li>
          <li>Make sure Tips.app is installed on the device (via App Store).</li>
          <li>Run TrollRestore from the saved venv (<code>~/saved/TrollRestore_venv/bin/python3 ~/saved/TrollRestore/trollstore.py</code>) — when prompted for an app name, type <code>Tips</code>.</li>
          <li>Reboot. Open Tips once to surface TrollStore.</li>
        </ol>
      </details>
    </section>
  {/if}
</div>

<style>
  .view { padding: var(--spacing-xl); max-width: 1024px; }
  .view-header { margin-bottom: var(--spacing-lg); }
  .view-header h1 {
    color: var(--color-text-primary);
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0 0 var(--spacing-xs);
  }
  .view-header p {
    color: var(--color-text-secondary);
    font-size: 0.9rem;
    line-height: 1.5;
    margin: 0;
  }

  .device-summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 1px;
    overflow: hidden;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-border);
    margin-bottom: var(--spacing-lg);
  }
  .device-summary div {
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: var(--color-bg-secondary);
    padding: var(--spacing-md);
  }
  .label {
    color: var(--color-text-secondary);
    font-size: 0.75rem;
  }
  .device-summary strong { color: var(--color-text-primary); font-size: 0.95rem; }
  .device-summary strong.mode[data-mode="Normal"] { color: var(--color-success); }
  .device-summary strong.mode[data-mode="Recovery"] { color: var(--color-warning); }
  .device-summary strong.mode[data-mode="DFU"],
  .device-summary strong.mode[data-mode="kDFU"],
  .device-summary strong.mode[data-mode="pwnDFU"] { color: var(--color-accent); }

  .tabs {
    display: flex;
    gap: var(--spacing-xs);
    border-bottom: 1px solid var(--color-border);
    margin-bottom: var(--spacing-md);
  }
  .tabs button {
    background: none;
    border: none;
    padding: 8px 12px;
    color: var(--color-text-secondary);
    font-size: 0.85rem;
    font-weight: 600;
    border-bottom: 2px solid transparent;
    cursor: pointer;
  }
  .tabs button.active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
  }

  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-md);
  }
  .section-title span {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--color-accent);
    color: white;
    font-size: 0.75rem;
    font-weight: 700;
  }
  .section-title h2 {
    color: var(--color-text-primary);
    font-size: 1rem;
    margin: 0;
  }
  .panel-note {
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    line-height: 1.5;
    margin: 0 0 var(--spacing-md);
  }
  .footer-note {
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    margin-top: var(--spacing-md);
  }

  .error-state {
    border: 1px solid color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    border-radius: var(--radius-md);
    background: var(--color-bg-secondary);
    color: var(--color-danger);
    font-size: 0.875rem;
    padding: var(--spacing-md);
    margin-bottom: var(--spacing-md);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--color-text-secondary);
    font-size: 0.75rem;
    font-weight: 600;
    margin-bottom: var(--spacing-sm);
  }
  .field input, .field textarea {
    width: 100%;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font: inherit;
    font-size: 0.85rem;
    padding: 8px 10px;
  }
  .field textarea {
    font-family: var(--font-mono, ui-monospace, SFMono-Regular, monospace);
    font-size: 0.78rem;
    resize: vertical;
  }
  .checkbox {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    color: var(--color-text-primary);
    font-size: 0.85rem;
    margin-bottom: var(--spacing-sm);
  }

  .row {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
    margin-bottom: var(--spacing-md);
    flex-wrap: wrap;
  }

  .actions {
    display: flex;
    gap: var(--spacing-sm);
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .action-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--spacing-sm);
  }
  .action-grid button {
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: flex-start;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    text-align: left;
  }
  .action-grid button:hover:not(:disabled) {
    border-color: var(--color-accent);
  }
  .action-grid button small {
    color: var(--color-text-secondary);
    font-size: 0.7rem;
    font-weight: 400;
  }
  .action-grid button.danger {
    border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    color: var(--color-danger);
  }

  button.primary, button.secondary, button.danger {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 12px;
    cursor: pointer;
  }
  button.primary {
    background: var(--color-accent);
    border: 1px solid var(--color-accent);
    color: white;
  }
  button.secondary {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }
  button.danger {
    background: var(--color-bg-primary);
    border: 1px solid color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
    color: var(--color-danger);
  }
  button:disabled { cursor: not-allowed; opacity: 0.5; }

  .status-pill {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 2px 10px;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }
  .status-pill[data-state="running"] {
    color: var(--color-success);
    border-color: color-mix(in srgb, var(--color-success) 45%, var(--color-border));
  }

  .syslog {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    height: 360px;
    overflow-y: auto;
    padding: var(--spacing-sm);
    font-family: var(--font-mono, ui-monospace, SFMono-Regular, monospace);
    font-size: 0.72rem;
    line-height: 1.4;
  }
  .syslog-empty {
    color: var(--color-text-secondary);
    text-align: center;
    padding: var(--spacing-md);
  }
  .syslog-line {
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-all;
  }
  .syslog-line[data-kind="stderr"] {
    color: var(--color-warning);
  }

  code { font-family: var(--font-mono, ui-monospace, SFMono-Regular, monospace); font-size: 0.78rem; }
  .wrap { word-break: break-all; }

  .elig {
    margin-top: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg-primary);
  }
  .elig strong { display: block; color: var(--color-text-primary); font-size: 0.9rem; }
  .elig p { color: var(--color-text-secondary); font-size: 0.8rem; margin: 4px 0; }
  .elig small { color: var(--color-text-secondary); font-size: 0.7rem; }
  .elig[data-path="incompatible"] strong { color: var(--color-danger); }
  .elig[data-path="ios14-15-ramdisk"] strong,
  .elig[data-path="ios16-trollrestore"] strong,
  .elig[data-path="ready"] strong { color: var(--color-success); }
  .elig[data-path="unknown"] strong { color: var(--color-warning); }

  details { margin-top: var(--spacing-sm); }
  summary {
    cursor: pointer;
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 6px 0;
  }
  .howto {
    color: var(--color-text-secondary);
    font-size: 0.82rem;
    line-height: 1.6;
    padding-left: 1.4em;
    margin: 0;
  }
  .howto li { margin-bottom: 4px; }
  .howto code {
    background: var(--color-bg-primary);
    padding: 1px 4px;
    border-radius: 3px;
  }
</style>
