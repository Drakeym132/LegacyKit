<script lang="ts">
  interface Props {
    value?: number | null;
    max?: number;
    label?: string | null;
    indeterminate?: boolean;
  }

  let { value = null, max = 100, label = null, indeterminate = false }: Props = $props();

  let pct = $derived(
    value === null || indeterminate ? null : Math.max(0, Math.min(100, (value / max) * 100)),
  );
</script>

<div class="wrap" role="progressbar" aria-valuemin={0} aria-valuemax={max} aria-valuenow={value ?? undefined}>
  {#if label}
    <div class="header">
      <span class="label">{label}</span>
      {#if pct !== null}
        <span class="pct">{pct.toFixed(0)}%</span>
      {/if}
    </div>
  {/if}
  <div class="track">
    {#if pct !== null}
      <div class="fill" style="width: {pct}%"></div>
    {:else}
      <div class="fill indeterminate"></div>
    {/if}
  </div>
</div>

<style>
  .wrap { width: 100%; }
  .header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin-bottom: 8px;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
  }
  .label { font-weight: 600; color: var(--color-text-primary); }
  .pct { font-variant-numeric: tabular-nums; }
  .track {
    height: 8px;
    border-radius: 999px;
    background: var(--color-bg-secondary);
    overflow: hidden;
    border: 1px solid var(--color-border);
  }
  .fill {
    height: 100%;
    background: var(--color-accent);
    transition: width 200ms ease;
  }
  .fill.indeterminate {
    width: 30%;
    animation: slide 1.4s ease-in-out infinite;
  }
  @keyframes slide {
    0% { transform: translateX(-100%); }
    50% { transform: translateX(150%); }
    100% { transform: translateX(330%); }
  }
</style>
