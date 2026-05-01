<script lang="ts">
  import { deviceStore } from '../stores/deviceStore.svelte';
  import { navigationStore } from '../stores/navigationStore.svelte';
  import type { ViewName } from '../stores/navigationStore.svelte';
  import DeviceStatus from '../components/device/DeviceStatus.svelte';

  function formatBytes(bytes: number): string {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  let isConnected = $derived(deviceStore.state.connected);
  let deviceName = $derived(deviceStore.state.name ?? 'Unknown Device');
  let productType = $derived(deviceStore.state.product_type ?? '—');
  let deviceMode = $derived(deviceStore.state.mode);

  // Identity
  let deviceClass = $derived(deviceStore.state.device_class ?? '—');
  let model = $derived(deviceStore.state.model ?? '—');
  let modelNumber = $derived(deviceStore.state.model_number ?? '—');
  let regionInfo = $derived(deviceStore.state.region_info ?? '—');
  let deviceColor = $derived(deviceStore.state.device_color ?? '—');
  let serial = $derived(deviceStore.state.serial ?? '—');
  let udid = $derived(deviceStore.state.udid ?? '—');
  let ecid = $derived(deviceStore.state.ecid ?? '—');

  // Software
  let iosVersion = $derived(deviceStore.state.ios_version ?? '—');
  let buildVersion = $derived(deviceStore.state.build_version ?? '—');
  let firmwareVersion = $derived(deviceStore.state.firmware_version ?? '—');
  let basebandVersion = $derived(deviceStore.state.baseband_version ?? '—');
  let activationState = $derived(deviceStore.state.activation_state ?? '—');

  // Hardware
  let cpuArchitecture = $derived(deviceStore.state.cpu_architecture ?? '—');
  let hardwarePlatform = $derived(deviceStore.state.hardware_platform ?? '—');
  let batteryCapacity = $derived(deviceStore.state.battery_current_capacity != null ? `${deviceStore.state.battery_current_capacity}%` : '—');
  let totalDisk = $derived(deviceStore.state.total_disk_capacity != null ? formatBytes(deviceStore.state.total_disk_capacity) : '—');
  let totalAvailable = $derived(deviceStore.state.total_data_available != null ? formatBytes(deviceStore.state.total_data_available) : '—');
  let passwordProtected = $derived(deviceStore.state.password_protected != null ? (deviceStore.state.password_protected ? 'Yes' : 'No') : '—');

  // Network
  let telephonyCapability = $derived(deviceStore.state.telephony_capability != null ? (deviceStore.state.telephony_capability ? 'Yes' : 'No') : '—');
  let imei = $derived(deviceStore.state.imei ?? '—');
  let wifiAddress = $derived(deviceStore.state.wifi_address ?? '—');
  let bluetoothAddress = $derived(deviceStore.state.bluetooth_address ?? '—');

  // Recovery/DFU fields
  let cpid = $derived(deviceStore.state.cpid ?? '—');
  let cprv = $derived(deviceStore.state.cprv ?? '—');
  let bdid = $derived(deviceStore.state.bdid ?? '—');
  let ibfl = $derived(deviceStore.state.ibfl ?? '—');
  let apnonce = $derived(deviceStore.state.apnonce ?? '—');
  let sepnonce = $derived(deviceStore.state.sepnonce ?? '—');

  // Section visibility checks (Normal mode)
  let hasIdentity = $derived(
    deviceStore.state.device_class != null || deviceStore.state.model != null ||
    deviceStore.state.model_number != null || deviceStore.state.region_info != null ||
    deviceStore.state.device_color != null || deviceStore.state.serial != null ||
    deviceStore.state.udid != null || deviceStore.state.ecid != null
  );
  let hasSoftware = $derived(
    deviceStore.state.ios_version != null || deviceStore.state.build_version != null ||
    deviceStore.state.firmware_version != null || deviceStore.state.baseband_version != null ||
    deviceStore.state.activation_state != null
  );
  let hasHardware = $derived(
    deviceStore.state.cpu_architecture != null || deviceStore.state.hardware_platform != null ||
    deviceStore.state.battery_current_capacity != null || deviceStore.state.total_disk_capacity != null ||
    deviceStore.state.total_data_available != null || deviceStore.state.password_protected != null
  );
  let hasNetwork = $derived(
    deviceStore.state.telephony_capability != null || deviceStore.state.imei != null ||
    deviceStore.state.wifi_address != null || deviceStore.state.bluetooth_address != null
  );

  // Section visibility checks (Recovery/DFU mode)
  let hasRecoveryInfo = $derived(
    deviceStore.state.ecid != null || deviceStore.state.serial != null ||
    deviceStore.state.model != null || deviceStore.state.product_type != null ||
    deviceStore.state.cpid != null || deviceStore.state.cprv != null ||
    deviceStore.state.bdid != null || deviceStore.state.ibfl != null
  );
  let hasNonces = $derived(
    deviceStore.state.apnonce != null || deviceStore.state.sepnonce != null
  );

  let isNormalMode = $derived(deviceMode === 'Normal');

  // Accordion open state
  let openSections = $state({
    identity: true,
    software: true,
    hardware: false,
    network: false,
    recovery: true,
    nonces: false,
  });

  const quickActions: { label: string; view: ViewName; icon: string }[] = [
    {
      label: 'Restore',
      view: 'restore',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>`,
    },
    {
      label: 'Jailbreak',
      view: 'jailbreak',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/><circle cx="12" cy="16" r="1" fill="currentColor"/></svg>`,
    },
    {
      label: 'SHSH Blobs',
      view: 'shsh',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17,21 17,13 7,13 7,21"/><polyline points="7,3 7,8 15,8"/></svg>`,
    },
    {
      label: 'SSH Ramdisk',
      view: 'ssh-ramdisk',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
    },
    {
      label: 'Apps',
      view: 'apps',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>`,
    },
    {
      label: 'Utilities',
      view: 'utilities',
      icon: `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14M4.93 4.93a10 10 0 0 0 0 14.14"/></svg>`,
    },
  ];
</script>

<div class="home-view">
  <!-- Hero card -->
  <section class="hero-card">
    <div class="accent-stripe"></div>

    <div class="hero-body">
      {#if isConnected}
        <div class="hero-top">
          <!-- Device tile + identity -->
          <div class="hero-device">
            <div class="device-tile">
              <!-- iPhone 5 silhouette -->
              <svg width="67" height="140" viewBox="0 0 96 200" fill="none">
                <rect x="1" y="1" width="94" height="198" rx="15" fill="#1c2b4a" />
                <rect x="3" y="3" width="90" height="194" rx="13" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="1" />
                <rect x="7" y="30" width="82" height="140" rx="5" fill="#dbeafe" />
                <rect x="7" y="30" width="82" height="140" rx="5" fill="url(#sg)" />
                <defs>
                  <linearGradient id="sg" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0%" stop-color="white" stop-opacity="0.18" />
                    <stop offset="100%" stop-color="white" stop-opacity="0" />
                  </linearGradient>
                </defs>
                <circle cx="48" cy="185" r="8" fill="none" stroke="rgba(255,255,255,0.15)" stroke-width="1.5" />
                <circle cx="48" cy="185" r="4" fill="rgba(255,255,255,0.06)" />
                <circle cx="48" cy="18" r="3.5" fill="rgba(255,255,255,0.12)" />
                {#each [37,41,45,49,53,57,61] as x}
                  <rect {x} y="16" width="2" height="4" rx="1" fill="rgba(255,255,255,0.1)" />
                {/each}
                <rect x="15" y="46" width="66" height="10" rx="3" fill="rgba(79,142,247,0.3)" />
                <rect x="15" y="62" width="46" height="5" rx="2" fill="rgba(79,142,247,0.15)" />
                <rect x="15" y="73" width="56" height="5" rx="2" fill="rgba(79,142,247,0.1)" />
                <rect x="15" y="84" width="38" height="5" rx="2" fill="rgba(79,142,247,0.08)" />
                {#each [15,37,59] as x}
                  <rect {x} y="100" width="18" height="18" rx="4" fill="rgba(79,142,247,0.12)" />
                {/each}
                <rect x="-1" y="60" width="3" height="20" rx="1.5" fill="#162038" stroke="rgba(255,255,255,0.08)" stroke-width="0.5" />
                <rect x="-1" y="86" width="3" height="16" rx="1.5" fill="#162038" stroke="rgba(255,255,255,0.08)" stroke-width="0.5" />
                <rect x="94" y="72" width="3" height="24" rx="1.5" fill="#162038" stroke="rgba(255,255,255,0.08)" stroke-width="0.5" />
                <rect x="70" y="-1" width="18" height="3" rx="1.5" fill="#162038" stroke="rgba(255,255,255,0.08)" stroke-width="0.5" />
              </svg>
            </div>

            <div class="hero-text">
              <h2 class="device-name">{deviceName}</h2>
              <span class="device-product">{productType}</span>
              <DeviceStatus mode={deviceMode} />
            </div>
          </div>

          <!-- Action toolbar -->
          <div class="action-toolbar" role="toolbar" aria-label="Quick actions">
            {#each quickActions as action}
              <button
                class="action-pill"
                onclick={() => navigationStore.navigate(action.view)}
                title={action.label}
              >
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html action.icon}
                <span>{action.label}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Stat chips -->
        {#if isNormalMode}
          <div class="stat-chips">
            <div class="stat-chip">
              <span class="stat-label">iOS</span>
              <span class="stat-value">{iosVersion}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">Build</span>
              <span class="stat-value mono">{buildVersion}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">Baseband</span>
              <span class="stat-value mono">{basebandVersion}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">Model</span>
              <span class="stat-value mono">{model}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">ECID</span>
              <span class="stat-value mono">{ecid}</span>
            </div>
          </div>
        {:else if deviceMode}
          <div class="stat-chips">
            <div class="stat-chip">
              <span class="stat-label">Mode</span>
              <span class="stat-value">{deviceMode}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">CPID</span>
              <span class="stat-value mono">{cpid}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">CPRV</span>
              <span class="stat-value mono">{cprv}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">ECID</span>
              <span class="stat-value mono">{ecid}</span>
            </div>
            <div class="stat-chip">
              <span class="stat-label">Board ID</span>
              <span class="stat-value mono">{bdid}</span>
            </div>
          </div>
        {/if}
      {:else}
        <!-- Disconnected state -->
        <div class="hero-device">
          <div class="device-tile disconnected">
            <svg width="56" height="116" viewBox="0 0 120 200" fill="none">
              <rect x="10" y="4" width="100" height="192" rx="18" stroke="currentColor" stroke-width="2.5" opacity="0.15" stroke-dasharray="6 4" />
              <rect x="16" y="32" width="88" height="130" rx="2" fill="currentColor" opacity="0.03" />
              <circle cx="60" cy="178" r="10" stroke="currentColor" stroke-width="2" opacity="0.1" />
              <rect x="44" y="14" width="32" height="4" rx="2" fill="currentColor" opacity="0.08" />
            </svg>
          </div>
          <div class="hero-text">
            <h2 class="device-name muted">No Device Connected</h2>
            <span class="device-product muted">Connect a USB device to get started</span>
          </div>
        </div>
      {/if}
    </div>
  </section>

  <!-- Detail grid with accordions -->
  {#if isConnected}
    <div class="detail-grid">
      {#if isNormalMode}
        {#if hasIdentity}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.identity = !openSections.identity)}
              aria-expanded={openSections.identity}
            >
              <h3 class="card-title">Identity</h3>
              <svg class="chevron" class:open={openSections.identity} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.identity}
              <dl class="detail-list">
                <div class="detail-row"><dt>Device Class</dt><dd>{deviceClass}</dd></div>
                <div class="detail-row"><dt>Model</dt><dd>{model}</dd></div>
                <div class="detail-row"><dt>Model Number</dt><dd>{modelNumber}</dd></div>
                <div class="detail-row"><dt>Region</dt><dd>{regionInfo}</dd></div>
                <div class="detail-row"><dt>Color</dt><dd>{deviceColor}</dd></div>
                <div class="detail-row"><dt>Serial</dt><dd class="mono">{serial}</dd></div>
                <div class="detail-row"><dt>UDID</dt><dd class="mono truncate" title={udid}>{udid}</dd></div>
                <div class="detail-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}

        {#if hasSoftware}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.software = !openSections.software)}
              aria-expanded={openSections.software}
            >
              <h3 class="card-title">Software</h3>
              <svg class="chevron" class:open={openSections.software} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.software}
              <dl class="detail-list">
                <div class="detail-row"><dt>iOS</dt><dd>{iosVersion}</dd></div>
                <div class="detail-row"><dt>Build</dt><dd>{buildVersion}</dd></div>
                <div class="detail-row"><dt>Firmware</dt><dd class="mono truncate" title={firmwareVersion}>{firmwareVersion}</dd></div>
                <div class="detail-row"><dt>Baseband</dt><dd>{basebandVersion}</dd></div>
                <div class="detail-row"><dt>Activation</dt><dd>{activationState}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}

        {#if hasHardware}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.hardware = !openSections.hardware)}
              aria-expanded={openSections.hardware}
            >
              <h3 class="card-title">Hardware</h3>
              <svg class="chevron" class:open={openSections.hardware} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.hardware}
              <dl class="detail-list">
                <div class="detail-row"><dt>CPU Architecture</dt><dd>{cpuArchitecture}</dd></div>
                <div class="detail-row"><dt>Platform</dt><dd>{hardwarePlatform}</dd></div>
                <div class="detail-row"><dt>Battery</dt><dd>{batteryCapacity}</dd></div>
                <div class="detail-row"><dt>Storage</dt><dd>{totalDisk}</dd></div>
                <div class="detail-row"><dt>Available</dt><dd>{totalAvailable}</dd></div>
                <div class="detail-row"><dt>Passcode</dt><dd>{passwordProtected}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}

        {#if hasNetwork}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.network = !openSections.network)}
              aria-expanded={openSections.network}
            >
              <h3 class="card-title">Network</h3>
              <svg class="chevron" class:open={openSections.network} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.network}
              <dl class="detail-list">
                <div class="detail-row"><dt>Cellular</dt><dd>{telephonyCapability}</dd></div>
                <div class="detail-row"><dt>IMEI</dt><dd class="mono">{imei}</dd></div>
                <div class="detail-row"><dt>WiFi MAC</dt><dd class="mono">{wifiAddress}</dd></div>
                <div class="detail-row"><dt>Bluetooth MAC</dt><dd class="mono">{bluetoothAddress}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}
      {:else}
        {#if hasRecoveryInfo}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.recovery = !openSections.recovery)}
              aria-expanded={openSections.recovery}
            >
              <h3 class="card-title">Recovery Info</h3>
              <svg class="chevron" class:open={openSections.recovery} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.recovery}
              <dl class="detail-list">
                <div class="detail-row"><dt>ECID</dt><dd class="mono">{ecid}</dd></div>
                <div class="detail-row"><dt>Serial</dt><dd class="mono">{serial}</dd></div>
                <div class="detail-row"><dt>Model</dt><dd>{model}</dd></div>
                <div class="detail-row"><dt>Product</dt><dd>{productType}</dd></div>
                <div class="detail-row"><dt>CPID</dt><dd class="mono">{cpid}</dd></div>
                <div class="detail-row"><dt>CPRV</dt><dd class="mono">{cprv}</dd></div>
                <div class="detail-row"><dt>Board ID</dt><dd class="mono">{bdid}</dd></div>
                <div class="detail-row"><dt>iBoot Flags</dt><dd class="mono">{ibfl}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}

        {#if hasNonces}
          <section class="detail-card">
            <button
              class="card-header"
              onclick={() => (openSections.nonces = !openSections.nonces)}
              aria-expanded={openSections.nonces}
            >
              <h3 class="card-title">Nonces</h3>
              <svg class="chevron" class:open={openSections.nonces} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9" /></svg>
            </button>
            {#if openSections.nonces}
              <dl class="detail-list">
                <div class="detail-row"><dt>APNonce</dt><dd class="mono truncate" title={apnonce}>{apnonce}</dd></div>
                <div class="detail-row"><dt>SEPNonce</dt><dd class="mono truncate" title={sepnonce}>{sepnonce}</dd></div>
              </dl>
            {/if}
          </section>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .home-view {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    height: 100%;
    min-height: 0;
    padding: var(--spacing-lg);
    overflow-y: auto;
  }

  /* ── Hero card ── */
  .hero-card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .accent-stripe {
    height: 4px;
    background: linear-gradient(90deg, #4f8ef7 0%, #8b5cf6 100%);
  }

  .hero-body {
    padding: var(--spacing-lg) 36px;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .hero-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--spacing-lg);
    flex-wrap: wrap;
  }

  .hero-device {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    min-width: 0;
  }

  /* Device tile */
  .device-tile {
    width: 108px;
    height: 144px;
    background: rgba(79, 142, 247, 0.1);
    border-radius: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .device-tile.disconnected {
    background: var(--color-bg-secondary);
    color: var(--color-text-secondary);
  }

  .hero-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .device-name {
    font-size: 1.375rem;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0;
    line-height: 1.2;
    word-break: break-word;
  }

  .device-name.muted {
    color: var(--color-text-secondary);
    font-weight: 600;
  }

  .device-product {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .device-product.muted {
    color: var(--color-text-secondary);
  }

  /* ── Pill action toolbar ── */
  .action-toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-xs);
    align-items: center;
    flex-shrink: 0;
  }

  .action-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
    transition: border-color 0.12s, box-shadow 0.12s;
    white-space: nowrap;
  }

  .action-pill:hover {
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 15%, transparent);
  }

  .action-pill:active {
    opacity: 0.8;
  }

  /* ── Stat chips ── */
  .stat-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-sm);
  }

  .stat-chip {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 14px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    min-width: 80px;
  }

  .stat-label {
    font-size: 0.625rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-text-secondary);
  }

  .stat-value {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .stat-value.mono {
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.75rem;
    font-weight: 500;
  }

  /* ── Detail grid ── */
  .detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--spacing-md);
    align-items: start;
  }

  .detail-card {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    min-width: 0;
  }

  /* ── Accordion card header ── */
  .card-header {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px var(--spacing-md);
    background: none;
    border: none;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.1s;
  }

  .card-header:hover {
    background: color-mix(in srgb, var(--color-text-primary) 3%, transparent);
  }

  .card-title {
    font-size: 0.6875rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .chevron {
    color: var(--color-text-secondary);
    flex-shrink: 0;
    transition: transform 0.2s ease;
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  /* ── Detail rows ── */
  .detail-list {
    margin: 0;
    display: flex;
    flex-direction: column;
    padding: 0 var(--spacing-md) var(--spacing-sm);
    border-top: 1px solid var(--color-border);
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: var(--spacing-sm);
    padding: 6px 0;
    border-bottom: 1px solid var(--color-border);
    min-width: 0;
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-row dt {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    font-weight: 500;
    flex-shrink: 0;
  }

  .detail-row dd {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-primary);
    font-weight: 500;
    text-align: right;
    min-width: 0;
  }

  .detail-row dd.mono {
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.6875rem;
    letter-spacing: -0.01em;
  }

  .detail-row dd.truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Responsive ── */
  @media (max-width: 640px) {
    .hero-body {
      padding: var(--spacing-md);
    }

    .hero-top {
      flex-direction: column;
    }

    .action-toolbar {
      width: 100%;
    }
  }
</style>
