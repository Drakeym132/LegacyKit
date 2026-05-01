<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';

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
</script>

<header
  class="titlebar"
  class:macos={isMacOS}
  data-tauri-drag-region
>
  <!-- macOS: traffic lights are rendered natively via Overlay; we just leave space -->
  <div class="titlebar-left" data-tauri-drag-region>
    {#if isMacOS}
      <!-- Spacer for native traffic light buttons -->
      <div class="traffic-light-spacer" data-tauri-drag-region></div>
    {/if}
  </div>

  <div class="titlebar-center" data-tauri-drag-region>
    <span class="titlebar-title" data-tauri-drag-region>LegacyKit</span>
  </div>

  <div class="titlebar-right">
    {#if !isMacOS}
      <!-- Windows/Linux window controls -->
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
</header>

<style>
  .titlebar {
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  .titlebar-left {
    display: flex;
    align-items: center;
    min-width: 70px;
  }

  .traffic-light-spacer {
    width: 70px;
    height: 100%;
  }

  .titlebar-center {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .titlebar-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
    opacity: 0.85;
  }

  .titlebar-right {
    display: flex;
    align-items: center;
    min-width: 70px;
    justify-content: flex-end;
  }

  /* Windows/Linux window controls */
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 38px;
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

  /* macOS-specific adjustments */
  .titlebar.macos .titlebar-right {
    min-width: 70px;
  }
</style>
