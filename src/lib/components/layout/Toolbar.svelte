<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import {
    navigationStore,
    viewTitles,
    type ViewName,
  } from '$lib/stores/navigationStore.svelte';

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

  function iconForView(view: ViewName): string {
    switch (view) {
      case 'restore':
        return 'M21 15v3.5A2.5 2.5 0 0 1 18.5 21h-13A2.5 2.5 0 0 1 3 18.5V15M7.5 11 12 15.5 16.5 11M12 3v12.5';
      case 'jailbreak':
        return 'M4 11h16v10H4zM8 11V7a4 4 0 0 1 7.5-2';
      case 'shsh':
        return 'M12 3 4 6v6c0 4.5 3.4 8.3 8 9 4.6-.7 8-4.5 8-9V6l-8-3M9 12l2 2 4-4';
      case 'ssh-ramdisk':
        return 'M2.5 4h19v13h-19zM7 9.5 9.5 12 7 14.5M12.5 14.5h4M9 21h6M12 17v4';
      case 'apps':
        return 'M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z';
      case 'data':
        return 'M4 5.5c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5-3.6-2.5-8-2.5-8 1.1-8 2.5zm0 0v6c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5v-6m-16 6v7c0 1.4 3.6 2.5 8 2.5s8-1.1 8-2.5v-7';
      case 'utilities':
        return 'M14.7 6.3a4 4 0 0 0-5.4 5.4L3 18l3 3 6.3-6.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.7-2.7 2.3-2.9z';
      case 'settings':
        return 'M12 9a3 3 0 1 1 0 6 3 3 0 0 1 0-6zm7.4 6a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z';
      case 'home':
      default:
        return 'M3 10.5 12 3l9 7.5V20a1 1 0 0 1-1 1h-4v-7h-8v7H4a1 1 0 0 1-1-1v-9.5Z';
    }
  }
</script>

<header class="titlebar" class:macos={isMacOS}>
  <div class="title-row" data-tauri-drag-region>
    <div class="title-left" data-tauri-drag-region>
      <div class="title-chip" data-tauri-drag-region>
        <span class="title-icon-wrap" aria-hidden="true" data-tauri-drag-region>
          <svg
            class="title-icon"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d={iconForView(navigationStore.currentView)} />
          </svg>
        </span>
        <h1 class="title-text" data-tauri-drag-region>
          {viewTitles[navigationStore.currentView]}
        </h1>
      </div>
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
    background: var(--color-accent);
    flex-shrink: 0;
    position: relative;
    z-index: 100;
   box-shadow: 0 10px 10px rgba(0, 0, 0, 0.06), 0 -1px 0 rgba(0, 0, 0, 0.04);
    user-select: none;
    -webkit-user-select: none;
  }

  .title-row {
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding-left: 16px;
  }

  .title-left {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  .title-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    color: #fff;
  }

  .title-icon-wrap {
    width: 12px;
    height: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    opacity: 0.95;
  }

  .title-icon {
    width: 12px;
    height: 12px;
  }

  .title-text {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    color: #fff;
    letter-spacing: 0;
    line-height: 1.4;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .title-right {
    display: flex;
    align-items: center;
    min-width: 56px;
    justify-content: flex-end;
    height: 100%;
  }

  /* Windows/Linux window controls */
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 32px;
    border: none;
    background: transparent;
    color: #fff;
    cursor: pointer;
    transition: background-color 0.1s ease;
    padding: 0;
    opacity: 0.82;
  }

  .window-control:hover {
    background-color: color-mix(in srgb, #000 18%, var(--color-accent));
    opacity: 1;
  }

  .window-control-close:hover {
    background-color: var(--color-danger);
    color: white;
    opacity: 1;
  }
</style>
