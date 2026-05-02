<script lang="ts">
  let {
    mode = 'Normal',
    connected = true,
  }: {
    mode?: 'Normal' | 'Recovery' | 'DFU' | 'kDFU' | 'pwnDFU' | 'WTF';
    connected?: boolean;
  } = $props();

  let statusColor = $derived(connected ? getStatusColor(mode) : 'var(--color-danger)');
  let modeLabel = $derived(mode === 'Normal' ? 'Booted' : mode);
  let label = $derived(connected ? `Connected · ${modeLabel}` : 'Disconnected');

  function getStatusColor(m: string): string {
    switch(m) {
      case 'Normal': return 'var(--color-success)';
      case 'Recovery':
      case 'DFU':
      case 'kDFU':
      case 'pwnDFU':
      case 'WTF': return '#AF52DE';
      default: return 'var(--color-text-secondary)';
    }
  }
</script>

<div class="status-indicator">
  <span class="dot" style="background-color: {statusColor}"></span>
  <span class="mode-text">{label}</span>
</div>

<style>
  .status-indicator {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background-color: var(--color-bg-secondary);
    padding: 2px 8px;
    border-radius: 10px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    display: inline-block;
  }

  .mode-text {
    font-size: 11px;
    font-weight: 500;
  }
</style>
