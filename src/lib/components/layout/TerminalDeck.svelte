<script lang="ts">
  import { settingsStore } from '../../stores/settingsStore.svelte';
  import { logStore } from '../../stores/logStore.svelte';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import TerminalLog from '../common/TerminalLog.svelte';

  const HEADER_HEIGHT = 28;

  let logCount = $derived(logStore.logs.length);
  let isConnected = $derived(deviceStore.state.connected);
  let isOpen = $derived(settingsStore.terminalVisible);
  let bodyHeight = $derived(
    isOpen ? Math.max(0, settingsStore.terminalHeight - HEADER_HEIGHT) : 0,
  );

  function toggle() {
    settingsStore.toggleTerminal();
  }

  function onHeaderKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggle();
    }
  }

  function stop(event: Event) {
    event.stopPropagation();
  }
</script>

<div
  class="terminal-deck shrink-0 flex flex-col bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] overflow-hidden"
  class:is-open={isOpen}
>
  <!-- Header (always visible, click anywhere to toggle) -->
  <div
    class="terminal-header h-[28px] shrink-0 flex justify-between items-center px-3 text-[11px] select-none cursor-pointer transition-colors"
    class:header-open={isOpen}
    role="button"
    tabindex="0"
    aria-expanded={isOpen}
    aria-label={isOpen ? 'Collapse terminal' : 'Open terminal'}
    onclick={toggle}
    onkeydown={onHeaderKeydown}
  >
    <div class="flex items-center gap-2 pointer-events-none">
      {#if isOpen}
        <div
          class="w-2 h-2 rounded-full {isConnected
            ? 'bg-[var(--color-success)]'
            : 'bg-[#86868B]'}"
        ></div>
        <span>Terminal Output</span>
      {:else}
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="4 17 10 11 4 5" />
          <line x1="12" y1="19" x2="20" y2="19" />
        </svg>
        <span>Terminal</span>
        {#if logCount > 0}
          <span
            class="px-1.5 py-px rounded-full bg-[var(--color-bg-primary)] border border-[var(--color-border)] text-[10px] leading-none text-[var(--color-text-primary)]"
          >
            {logCount}
          </span>
        {/if}
      {/if}
    </div>

    <div class="flex items-center gap-3">
      {#if isOpen}
        <button
          type="button"
          class="hover:text-white transition-colors cursor-pointer"
          onclick={(e) => {
            stop(e);
            logStore.clear();
          }}
        >
          Clear
        </button>
      {:else}
        <span class="pointer-events-none">v1.0.0</span>
      {/if}
      <span
        class="chevron flex items-center"
        class:chevron-open={isOpen}
        aria-hidden="true"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </span>
    </div>
  </div>

  <!-- Animated body -->
  <div
    class="terminal-body"
    style:height="{bodyHeight}px"
    aria-hidden={!isOpen}
  >
    <div class="terminal-body-inner">
      <TerminalLog />
    </div>
  </div>
</div>

<style>
  .terminal-deck {
    box-shadow: 0 -10px 10px rgba(0, 0, 0, 0.06), 0 -1px 0 rgba(0, 0, 0, 0.04);
  }
  .terminal-header {
    color: var(--color-text-secondary);
    background: var(--color-bg-secondary);
  }
  .terminal-header:hover {
    background: color-mix(
      in srgb,
      var(--color-text-primary) 4%,
      var(--color-bg-secondary)
    );
  }
  .terminal-header.header-open {
    color: #98989d;
    background: #2c2c2e;
    border-bottom: 1px solid #38383a;
  }
  .terminal-header.header-open:hover {
    background: #3a3a3c;
  }

  /* Animated open/close: height transition on body */
  .terminal-body {
    overflow: hidden;
    background: #1c1c1e;
    /* Apple-style ease for natural motion in both directions */
    transition: height 240ms cubic-bezier(0.32, 0.72, 0, 1);
    will-change: height;
  }

  .terminal-body-inner {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  /* Chevron rotates smoothly between states */
  .chevron {
    transition: transform 240ms cubic-bezier(0.32, 0.72, 0, 1);
    transform: rotate(0deg);
  }
  .chevron-open {
    transform: rotate(180deg);
  }

  /* Respect reduced motion */
  @media (prefers-reduced-motion: reduce) {
    .terminal-body,
    .chevron {
      transition: none;
    }
  }
</style>

