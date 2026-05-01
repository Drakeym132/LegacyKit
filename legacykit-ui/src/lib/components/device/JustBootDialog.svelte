<script lang="ts">
  import { runKloader } from '$lib/api/jailbreak';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { toastStore } from '$lib/stores/toastStore.svelte';

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  let ibssPath = $state('');
  let ibecPath = $state('');
  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);

  function handleClose() {
    if (isWorking) return;
    errorMessage = null;
    onClose();
  }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') handleClose();
  }

  async function handleBoot() {
    if (!ibssPath.trim()) {
      errorMessage = 'Patched iBSS path is required.';
      return;
    }
    isWorking = true;
    errorMessage = null;
    const label = 'Booting via kloader';
    logStore.append(`${label}...`, 'info');
    try {
      await runKloader({ ibssPath: ibssPath.trim(), ibecPath: ibecPath.trim() || null });
      logStore.append(`${label} ok`, 'info');
      toastStore.success(label, 'Completed');
      ibssPath = '';
      ibecPath = '';
      onClose();
    } catch (error) {
      const msg = error instanceof Error ? error.message : String(error);
      errorMessage = msg;
      logStore.append(`${label} failed: ${msg}`, 'stderr');
      toastStore.error(`${label} failed`, msg);
    } finally {
      isWorking = false;
    }
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div class="overlay" role="presentation" onclick={handleClose}>
    <div
      class="dialog"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="just-boot-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 id="just-boot-title">Just Boot</h3>
      <p>Tethered-boot a device in pwned DFU using a previously patched iBSS (and optional iBEC).</p>

      <label class="field">
        <span>Patched iBSS path</span>
        <input bind:value={ibssPath} placeholder="/path/to/iBSS.repacked" disabled={isWorking} />
      </label>

      <label class="field">
        <span>Patched iBEC path <em>(optional)</em></span>
        <input bind:value={ibecPath} placeholder="/path/to/iBEC.repacked" disabled={isWorking} />
      </label>

      {#if errorMessage}
        <div class="error">{errorMessage}</div>
      {/if}

      <div class="actions">
        <button class="secondary" onclick={handleClose} disabled={isWorking}>Cancel</button>
        <button class="primary" onclick={handleBoot} disabled={isWorking}>
          {isWorking ? 'Booting…' : 'Boot'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: grid;
    place-items: center;
    z-index: 999;
    padding: 16px;
  }
  .dialog {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    width: min(480px, 100%);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }
  h3 { margin: 0; font-size: 1rem; }
  p { color: var(--color-text-secondary); font-size: 0.85rem; margin: 0; line-height: 1.5; }
  .field { display: flex; flex-direction: column; gap: 4px; font-size: 0.8125rem; }
  .field span { color: var(--color-text-secondary); }
  .field em { font-style: normal; opacity: 0.7; }
  .field input {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    padding: 6px 10px;
    font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
    font-size: 0.8125rem;
  }
  .field input:disabled { opacity: 0.6; }
  .error {
    color: var(--color-danger);
    font-size: 0.8125rem;
    background: color-mix(in srgb, var(--color-danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }
  .actions { display: flex; justify-content: flex-end; gap: var(--spacing-sm); margin-top: var(--spacing-xs); }
  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
  }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .secondary {
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }
  .primary {
    background: var(--color-accent);
    border: 1px solid var(--color-accent);
    color: white;
  }
</style>
