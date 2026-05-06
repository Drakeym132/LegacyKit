<script lang="ts">
  import { listExistingIpsws, type ExistingIpswEntry } from '$lib/api/restore';

  interface Props {
    deviceIdentifier: string | null;
    onSelect: (path: string, sha1: string | null) => void;
  }

  let { deviceIdentifier, onSelect }: Props = $props();

  let ipsws = $state<ExistingIpswEntry[]>([]);

  $effect(() => {
    if (deviceIdentifier) {
      void load(deviceIdentifier);
    } else {
      ipsws = [];
    }
  });

  async function load(device: string) {
    try {
      const result = await listExistingIpsws({ deviceIdentifier: device });
      ipsws = result.ipsws;
    } catch {
      ipsws = [];
    }
  }

  function formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = bytes;
    let unit = 0;
    while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
    }
    return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
  }
</script>

{#if ipsws.length > 0}
  <div class="picker">
    <h4>Existing IPSWs in Workspace</h4>
    <div class="list">
      {#each ipsws as ipsw}
        <div class="item">
          <div class="info">
            <span class="name">{ipsw.fileName}</span>
            <span class="size">{formatBytes(ipsw.sizeBytes)}</span>
          </div>
          <button onclick={() => onSelect(ipsw.path, null)}>Use</button>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .picker {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm);
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-bg-primary));
    margin-bottom: var(--spacing-md);
  }

  h4 {
    margin: 0 0 var(--spacing-sm);
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .list {
    display: grid;
    gap: 4px;
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-sm);
    padding: 7px 10px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    font-size: 0.82rem;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size {
    font-size: 0.72rem;
    color: var(--color-text-secondary);
  }

  button {
    flex-shrink: 0;
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-accent);
    font-size: 0.8rem;
    font-weight: 600;
    padding: 5px 12px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  button:hover {
    background: var(--color-accent);
    color: #fff;
  }
</style>
