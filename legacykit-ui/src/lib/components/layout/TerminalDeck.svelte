<script lang="ts">
  import { slide } from 'svelte/transition';
  import { quintOut } from 'svelte/easing';
  import { settingsStore } from '../../stores/settingsStore.svelte';
  import { logStore } from '../../stores/logStore.svelte';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import TerminalLog from '../common/TerminalLog.svelte';

  let logCount = $derived(logStore.logs.length);
  let isConnected = $derived(deviceStore.state.connected);
</script>

{#if settingsStore.terminalVisible}
  <div
    class="shrink-0 flex flex-col bg-[#1C1C1E] border-t border-[#38383A] overflow-hidden"
    style={`height: ${settingsStore.terminalHeight}px`}
    transition:slide={{ duration: 180, easing: quintOut, axis: 'y' }}
  >
    <div class="h-[28px] shrink-0 flex justify-between items-center px-3 text-[11px] text-[#98989D] bg-[#2C2C2E] border-b border-[#38383A]">
      <div class="flex items-center gap-2">
        <div class="w-2 h-2 rounded-full {isConnected ? 'bg-[var(--color-success)]' : 'bg-[#86868B]'}"></div>
        <span>Terminal Output</span>
      </div>
      <div class="flex items-center gap-3">
        <button class="hover:text-white transition-colors" onclick={() => logStore.clear()}>
          Clear
        </button>
        <button
          class="flex items-center hover:text-white transition-colors -mr-1 p-1"
          onclick={() => settingsStore.toggleTerminal()}
          aria-label="Collapse terminal"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      </div>
    </div>
    <TerminalLog />
  </div>
{:else}
  <button
    type="button"
    class="h-[28px] shrink-0 flex items-center justify-between px-2 bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] text-[11px] text-[var(--color-text-secondary)] hover:bg-[color-mix(in_srgb,var(--color-text-primary)_4%,var(--color-bg-secondary))] transition-colors w-full text-left cursor-pointer"
    onclick={() => settingsStore.toggleTerminal()}
    aria-label="Open terminal"
  >
    <div class="flex items-center gap-2">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
      <span>Terminal</span>
      {#if logCount > 0}
        <span class="px-1.5 py-px rounded-full bg-[var(--color-bg-primary)] border border-[var(--color-border)] text-[10px] leading-none text-[var(--color-text-primary)]">
          {logCount}
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-3">
      <span>v1.0.0</span>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="18 15 12 9 6 15" />
      </svg>
    </div>
  </button>
{/if}
