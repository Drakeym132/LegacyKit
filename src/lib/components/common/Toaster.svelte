<script lang="ts">
  import { toastStore } from '$lib/stores/toastStore.svelte';
</script>

<div class="toaster" aria-live="polite">
  {#each toastStore.toasts as toast (toast.id)}
    <div class="toast" data-variant={toast.variant} role="status">
      <div class="body">
        <strong>{toast.title}</strong>
        {#if toast.body}
          <p>{toast.body}</p>
        {/if}
      </div>
      <button class="close" onclick={() => toastStore.dismiss(toast.id)} aria-label="Dismiss">×</button>
    </div>
  {/each}
</div>

<style>
  .toaster {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 1000;
    max-width: min(360px, calc(100vw - 32px));
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-left: 4px solid var(--color-accent);
    border-radius: var(--radius-md);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    font-size: 0.85rem;
    animation: toast-in 140ms ease-out;
  }
  .toast[data-variant="success"] { border-left-color: var(--color-success); }
  .toast[data-variant="warning"] { border-left-color: var(--color-warning); }
  .toast[data-variant="error"] { border-left-color: var(--color-danger); }
  .toast[data-variant="info"] { border-left-color: var(--color-accent); }

  .body { flex: 1; min-width: 0; }
  .body strong {
    display: block;
    color: var(--color-text-primary);
    font-size: 0.85rem;
    line-height: 1.3;
  }
  .body p {
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    line-height: 1.4;
    margin: 4px 0 0;
    word-break: break-word;
  }
  .close {
    background: none;
    border: none;
    color: var(--color-text-secondary);
    font-size: 1.2rem;
    line-height: 1;
    cursor: pointer;
    padding: 0 4px;
  }
  .close:hover { color: var(--color-text-primary); }

  @keyframes toast-in {
    from { transform: translateY(8px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
</style>
