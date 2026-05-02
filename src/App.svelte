<script lang="ts">
  import Toolbar from './lib/components/layout/Toolbar.svelte';
  import Sidebar from './lib/components/layout/Sidebar.svelte';
  import ContentArea from './lib/components/layout/ContentArea.svelte';
  import Toaster from './lib/components/common/Toaster.svelte';
  import WorkspaceOnboarding from './lib/components/onboarding/WorkspaceOnboarding.svelte';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { deviceStore } from './lib/stores/deviceStore.svelte';
  import type { DeviceInfo } from './lib/stores/deviceStore.svelte';
  import type { LogEventPayload } from './lib/stores/logStore.svelte';
  import { logStore } from './lib/stores/logStore.svelte';
  import { settingsStore } from './lib/stores/settingsStore.svelte';
  import { setWindowShadow } from './lib/api/settings';
  
  import './app.css';

  let pollInterval: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    const autoDetectDevice = settingsStore.autoDetectDevice;
    const pollIntervalMs = settingsStore.pollBoostMs ?? settingsStore.pollIntervalMs;

    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }

    if (autoDetectDevice) {
      detectDevice();
      pollInterval = setInterval(detectDevice, pollIntervalMs);
    }

    return () => {
      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
    };
  });

  $effect(() => {
    const theme = settingsStore.theme;
    const root = document.documentElement;
    const darkQuery = window.matchMedia('(prefers-color-scheme: dark)');

    function applyTheme() {
      if (theme === 'system') {
        root.removeAttribute('data-theme');
      } else {
        root.dataset.theme = theme;
      }
      root.classList.toggle('dark', theme === 'dark' || (theme === 'system' && darkQuery.matches));
    }

    applyTheme();
    darkQuery.addEventListener('change', applyTheme);

    return () => darkQuery.removeEventListener('change', applyTheme);
  });

  $effect(() => {
    const root = document.documentElement;
    const glassChrome = settingsStore.glassChrome;
    if (glassChrome) {
      root.dataset.glassChrome = 'true';
    } else {
      delete root.dataset.glassChrome;
    }

    void setWindowShadow(true).catch(() => {
      // Non-fatal: shadow toggling is a cosmetic enhancement on supported
      // platforms and should never block rendering.
    });
  });

  onMount(() => {
    void settingsStore.load();

    const unlistenLog = listen<LogEventPayload>('log_event', (event) => {
      const { text, type } = event.payload;
      logStore.append(text, type);
    });

    return () => {
      unlistenLog.then(fn => fn());
    };
  });

  async function detectDevice() {
    try {
      const info = await invoke<DeviceInfo>('detect_device');
      if (info && info.connected) {
        deviceStore.updateFromBackend(info);
      } else {
        deviceStore.clearDevice();
      }
    } catch {
      deviceStore.clearDevice();
    }
  }
</script>

<div class="app-shell">
  <Sidebar />
  <div class="app-content">
    <Toolbar />
    <ContentArea />
  </div>
</div>
<Toaster />

{#if settingsStore.loaded && !settingsStore.onboarded}
  <WorkspaceOnboarding />
{/if}

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    color: var(--color-text-primary);
    background: var(--shell-bg);
    border-radius: var(--shell-radius);
    gap: var(--shell-gap);
    padding: 0 var(--shell-inset) var(--shell-inset);
  }

  .app-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    background-color: var(--color-bg-elevated);
    border: var(--content-border);
    border-radius: var(--content-radius);
    margin-top: var(--shell-inset-top);
    box-shadow: var(--content-shadow);
    backdrop-filter: blur(18px) saturate(180%);
    -webkit-backdrop-filter: blur(18px) saturate(180%);
  }
</style>
