<script lang="ts">
  import { logStore } from '../../stores/logStore.svelte';
  import { tick } from 'svelte';

  let terminalContainer: HTMLDivElement | null = null;

  $effect(() => {
    // Track logStore.logs reactively to trigger scroll on changes
    logStore.logs;

    if (terminalContainer) {
      tick().then(() => {
        if (terminalContainer) {
          terminalContainer.scrollTop = terminalContainer.scrollHeight;
        }
      });
    }
  });
</script>

<div bind:this={terminalContainer} class="flex-1 p-3 overflow-y-auto font-mono text-[11px] leading-snug bg-[#1C1C1E] text-[#F5F5F7]">
  {#each logStore.logs as log}
    <div class="mb-1" class:text-red-400={log.type === 'stderr'} class:text-blue-400={log.type === 'info'}>
      <span class="opacity-50 mr-2">[{new Date(log.timestamp).toLocaleTimeString()}]</span>
      <span>{log.text}</span>
    </div>
  {/each}
  {#if logStore.logs.length === 0}
    <div class="text-[#86868B] italic">No logs to display...</div>
  {/if}
</div>
