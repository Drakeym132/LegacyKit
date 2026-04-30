<script lang="ts">
  interface Props {
    open: boolean;
    title: string;
    body?: string | null;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: 'default' | 'danger';
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    open,
    title,
    body = null,
    confirmLabel = 'Confirm',
    cancelLabel = 'Cancel',
    variant = 'default',
    onConfirm,
    onCancel,
  }: Props = $props();

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') onCancel();
    if (e.key === 'Enter') onConfirm();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div class="overlay" role="presentation" onclick={onCancel}>
    <div
      class="dialog"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-labelledby="confirm-title"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <h3 id="confirm-title">{title}</h3>
      {#if body}
        <p>{body}</p>
      {/if}
      <div class="actions">
        <button class="secondary" onclick={onCancel}>{cancelLabel}</button>
        <button class:danger={variant === 'danger'} class:primary={variant !== 'danger'} onclick={onConfirm}>
          {confirmLabel}
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
    width: min(420px, 100%);
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  }
  h3 { margin: 0 0 var(--spacing-sm); font-size: 1rem; }
  p { color: var(--color-text-secondary); font-size: 0.85rem; margin: 0 0 var(--spacing-md); line-height: 1.5; }
  .actions { display: flex; justify-content: flex-end; gap: var(--spacing-sm); }
  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
  }
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
  .danger {
    background: var(--color-danger);
    border: 1px solid var(--color-danger);
    color: white;
  }
</style>
