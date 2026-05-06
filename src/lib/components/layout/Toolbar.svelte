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

<header class="titlebar" class:macos={isMacOS}>
  <div class="title-row" data-tauri-drag-region>
    <div class="title-left" data-tauri-drag-region>
      <h1 class="title-text" data-tauri-drag-region>LegacyKit</h1>
    </div>

    <div class="title-right">
      {#if !isMacOS}
        <button class="window-control" onclick={minimize} aria-label="Minimize">
          <svg aria-hidden="true" focusable="false" width="10" height="1" viewBox="0 0 10 1">
            <rect width="10" height="1" fill="currentColor" />
          </svg>
        </button>
        <button class="window-control" onclick={toggleMaximize} aria-label={isMaximized ? 'Restore' : 'Maximize'}>
          {#if isMaximized}
            <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10">
              <path d="M2 0h6a2 2 0 012 2v6a2 2 0 01-2 2H2a2 2 0 01-2-2V2a2 2 0 012-2zm0 1a1 1 0 00-1 1v6a1 1 0 001 1h6a1 1 0 001-1V2a1 1 0 00-1-1H2z" fill="currentColor" />
            </svg>
          {:else}
            <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10">
              <rect x="0" y="0" width="10" height="10" rx="1.5" stroke="currentColor" stroke-width="1" fill="none" />
            </svg>
          {/if}
        </button>
        <button class="window-control window-control-close" onclick={close} aria-label="Close">
          <svg aria-hidden="true" focusable="false" width="10" height="10" viewBox="0 0 10 10">
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
    background: color-mix(in srgb, var(--color-accent) 85%, rgba(255,255,255,0.08));
    flex-shrink: 0;
    position: relative;
    z-index: 100;
    box-shadow: 0 10px 10px rgba(0, 0, 0, 0.06), 0 -1px 0 rgba(0, 0, 0, 0.04);
    backdrop-filter: blur(12px) saturate(180%);
    -webkit-backdrop-filter: blur(12px) saturate(180%);
    user-select: none;
    -webkit-user-select: none;
  }

  /* Toolbar geometry mirrors the terminal deck header (28px tall, 11px
   * text) so the top and bottom chrome bars read as a matched pair
   * framing the content. The left inset is slightly larger than the
   * terminal deck's so the title clears the rounded corner of the
   * content panel and sits visually centered over the column. */
  .title-row {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding-left: 18px;
    padding-right: 10px;
  }

  .title-left {
    display: flex;
    align-items: center;
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  /* App title. Lives in the toolbar so it scans as the document/app
   * title rather than sidebar branding. Sized to match the terminal
   * deck label (11px) so the two chrome bars stay symmetric. */
  .title-text {
    margin: 0;
    font-size: 11px;
    font-weight: 600;
    color: #fff;
    letter-spacing: 0.01em;
    line-height: 1.2;
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

  /* Windows/Linux window controls — sized to fit inside the 28px bar. */
  .window-control {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 28px;
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
