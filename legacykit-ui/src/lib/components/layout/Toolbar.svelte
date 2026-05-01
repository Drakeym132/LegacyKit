<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import { navigationStore } from '../../stores/navigationStore.svelte';
  import type { ViewName } from '../../stores/navigationStore.svelte';

  let isMacOS = $state(false);
  let isMaximized = $state(false);

  const appWindow = getCurrentWindow();

  onMount(async () => {
    isMacOS = navigator.userAgent.includes('Mac');
    isMaximized = await appWindow.isMaximized();
  });

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
    isMaximized = await appWindow.isMaximized();
  }

  async function close() {
    await appWindow.close();
  }

  let isConnected = $derived(deviceStore.state.connected);

  const toolbarActions: { label: string; view: ViewName; icon: string }[] = [
    {
      label: 'Restore',
      view: 'restore',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>`,
    },
    {
      label: 'Jailbreak',
      view: 'jailbreak',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/><circle cx="12" cy="16" r="1" fill="currentColor"/></svg>`,
    },
    {
      label: 'SHSH Blobs',
      view: 'shsh',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17,21 17,13 7,13 7,21"/><polyline points="7,3 7,8 15,8"/></svg>`,
    },
    {
      label: 'SSH Ramdisk',
      view: 'ssh-ramdisk',
      icon: `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>`,
    },
  ];
</script>

<header class="titlebar" class:macos={isMacOS}>
  <div class="title-row" data-tauri-drag-region>
    <div class="title-left" data-tauri-drag-region>
      {#if isMacOS}
        <div class="traffic-light-spacer" data-tauri-drag-region></div>
      {/if}
    </div>

    <div class="title-center" data-tauri-drag-region>
      <span class="title-text" data-tauri-drag-region>LegacyKit</span>
    </div>

    <div class="title-right">
      <div class="device-status" title={isConnected ? 'Device Connected' : 'No device'}>
        <span class="device-dot" class:connected={isConnected}></span>
        <span class="device-label">{isConnected ? 'Connected' : 'Ready'}</span>
      </div>
      {#if !isMacOS}
        <button class="window-control" onclick={minimize} aria-label="Minimize">
          <svg width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="window-control" onclick={toggleMaximize} aria-label={isMaximized ? 'Restore' : 'Maximize'}>
          {#if isMaximized}
            <svg width="10" height="10" viewBox="0 0 10 10">
              <path d="M2 0h6a2 2 0 012 2v6a2 2 0 01-2 2H2a2 2 0 01-2-2V2a2 2 0 012-2zm0 1a1 1 0 00-1 1v6a1 1 0 001 1h6a1 1 0 001-1V2a1 1 0 00-1-1H2z" fill="currentColor" />
            </svg>
          {:else}
            <svg width="10" height="10" viewBox="0 0 10 10">
              <rect x="0" y="0" width="10" height="10" rx="1.5" stroke="currentColor" stroke-width="1" fill="none" />
            </svg>
          {/if}
        </button>
        <button class="window-control window-control-close" onclick={close} aria-label="Close">
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <div class="action-row" data-tauri-drag-region>
    {#each toolbarActions as action}
      <button
        class="toolbar-action"
        onclick={() => navigationStore.navigate(action.view)}
        disabled={!isConnected}
        aria-label={action.label}
        title={action.label}
      >
        <span class="action-icon">
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html action.icon}
        </span>
        <span class="action-label">{action.label}</span>
      </button>
    {/each}
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-sidebar);
    border-bottom: 1px solid var(--color-border);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    flex-shrink: 0;
    position: relative;
    z-index: 100;
    user-select: none;
    -webkit-user-select: none;
  }

  .title-row {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .title-left {
    display: flex;
    align-items: center;
    min-width: 70px;
    height: 100%;
  }

  .traffic-light-spacer {
    width: 70px;
    height: 100%;
  }

  .title-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .title-text {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-primary);
    opacity: 0.7;
  }

  .title-right {
    display: flex;
    align-items: center;
    min-width: 70px;
    justify-content: flex-end;
    height: 100%;
  }

  /* Action row */
  .action-row {
    height: 52px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 16px;
    border-top: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
  }

  .toolbar-action {
    -webkit-app-region: no-drag;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-width: 64px;
    height: 44px;
    padding: 4px 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    cursor: pointer;
    font-family: inherit;
    color: var(--color-text-primary);
    transition: background-color 0.1s ease, border-color 0.1s ease, opacity 0.1s ease;
  }

  .toolbar-action:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-text-primary) 5%, transparent);
  }

  .toolbar-action:active:not(:disabled) {
    background: var(--color-bg-secondary);
  }

  .toolbar-action:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .action-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .action-label {
    font-size: 10px;
    font-weight: 500;
    color: var(--color-text-secondary);
    letter-spacing: 0.01em;
    line-height: 1;
  }

  /* Windows/Linux window controls */
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--color-text-primary);
    cursor: pointer;
    transition: background-color 0.1s ease;
    padding: 0;
    opacity: 0.7;
  }

  .window-control:hover {
    background-color: var(--color-bg-secondary);
    opacity: 1;
  }

  .window-control-close:hover {
    background-color: var(--color-danger);
    color: white;
    opacity: 1;
  }

  .titlebar.macos .title-right {
    min-width: 140px;
    padding-right: 8px;
  }

  .device-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    font-size: 11px;
    color: var(--color-text-secondary);
    -webkit-app-region: no-drag;
  }

  .device-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--color-text-secondary);
    opacity: 0.6;
  }

  .device-dot.connected {
    background: var(--color-success);
    opacity: 1;
  }

  .device-label {
    line-height: 1;
  }
</style>
