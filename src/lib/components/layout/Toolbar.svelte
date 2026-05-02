<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { navigationStore, viewTitles } from '$lib/stores/navigationStore.svelte';

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

<header class="titlebar" class:macos={isMacOS}>
  <div class="title-row" data-tauri-drag-region>
    <div class="title-left" data-tauri-drag-region>
      <h1 class="title-text" data-tauri-drag-region>
        {viewTitles[navigationStore.currentView]}
      </h1>
    </div>

    <div class="title-right">
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
</header>

<style>
  .titlebar {
    display: flex;
    flex-direction: column;
    background-color: transparent;
    flex-shrink: 0;
    position: relative;
    z-index: 100;
    user-select: none;
    -webkit-user-select: none;
  }

  .title-row {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding-left: 24px;
  }

  .title-left {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  .title-text {
    margin: 0;
    font-size: 17px;
    font-weight: 700;
    color: var(--color-text-primary);
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .title-right {
    display: flex;
    align-items: center;
    min-width: 70px;
    justify-content: flex-end;
    height: 100%;
  }

  /* Windows/Linux window controls */
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 48px;
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
</style>
