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

<div class="flex h-screen w-screen overflow-hidden rounded-[28px] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] gap-2 p-2 pt-0">
  <Sidebar />
  <div class="flex flex-col flex-1 overflow-hidden rounded-[20px] bg-[var(--color-bg-elevated)] border border-[var(--color-border)] mt-2">
    <Toolbar />
    <ContentArea />
  </div>
</div>
<Toaster />

{#if settingsStore.loaded && !settingsStore.onboarded}
  <WorkspaceOnboarding />
{/if}
