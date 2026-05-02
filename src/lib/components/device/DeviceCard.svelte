<script lang="ts">
  import DeviceStatus from './DeviceStatus.svelte';
  import DeviceImage from './DeviceImage.svelte';
  import { deviceStore } from '../../stores/deviceStore.svelte';
  import { getDeviceFriendlyName } from '../../utils/deviceModels';

  let isConnected = $derived(deviceStore.state.connected);
  let deviceName = $derived(
    deviceStore.state.name
      || getDeviceFriendlyName(deviceStore.state.product_type)
      || deviceStore.state.product_type
      || deviceStore.state.model
      || deviceStore.state.udid
      || 'Unknown Device'
  );
  let iosVersion = $derived(deviceStore.state.ios_version || 'Unknown');
  let deviceMode = $derived(deviceStore.state.mode);
  let batteryCapacity = $derived(deviceStore.state.battery_current_capacity);
  let activationState = $derived(deviceStore.state.activation_state || '');
  let cpid = $derived(deviceStore.state.cpid || '');
  let productType = $derived(deviceStore.state.product_type);
  let deviceColor = $derived(deviceStore.state.device_color);
</script>

<div class="bg-[var(--color-bg-elevated)] border border-[var(--color-border)] rounded-[var(--radius-lg)] p-3 shadow-sm transition-all duration-200 {isConnected ? 'ring-1 ring-[var(--color-accent)] ring-opacity-30' : ''}">
  <div class="flex items-center gap-2">
    <div class="w-10 h-10 flex items-center justify-center shrink-0">
      {#if isConnected && productType}
        <DeviceImage {productType} {deviceColor} width={40} height={40} />
      {:else}
        <span class="text-2xl leading-none">📱</span>
      {/if}
    </div>
    <div class="flex-1 min-w-0">
      {#if isConnected}
        <h3 class="m-0 text-[14px] font-semibold text-[var(--color-text-primary)] truncate">{deviceName}</h3>
      {:else}
        <h3 class="m-0 text-[14px] font-semibold text-[var(--color-text-primary)]">No Device</h3>
        <span class="text-[11px] text-[var(--color-text-secondary)]">Connect USB to begin</span>
      {/if}
    </div>
  </div>

  <div class="mt-3 pt-3 border-t border-[var(--color-border)]">
    {#if isConnected && deviceMode === 'Normal'}
      <div class="flex justify-between items-center text-[12px] mb-1">
        <span class="text-[var(--color-text-secondary)]">iOS</span>
        <span class="font-medium text-[var(--color-text-primary)]">{iosVersion}</span>
      </div>
    {/if}
    <div class="flex justify-between items-center text-[12px] mb-1">
      <span class="text-[var(--color-text-secondary)]">Mode</span>
      <span class="font-medium text-[var(--color-text-primary)] -mr-2">
        <DeviceStatus mode={deviceMode} connected={isConnected} />
      </span>
    </div>
    {#if isConnected && deviceMode === 'Normal'}
      {#if batteryCapacity != null}
        <div class="flex justify-between items-center text-[12px] mb-1">
          <span class="text-[var(--color-text-secondary)]">Battery</span>
          <span class="font-medium text-[var(--color-text-primary)]">{batteryCapacity}%</span>
        </div>
      {/if}
      {#if activationState}
        <div class="flex justify-between items-center text-[12px]">
          <span class="text-[var(--color-text-secondary)]">Activation</span>
          <span class="font-medium text-[var(--color-text-primary)]">{activationState}</span>
        </div>
      {/if}
    {:else if isConnected && cpid}
      <div class="flex justify-between items-center text-[12px]">
        <span class="text-[var(--color-text-secondary)]">CPID</span>
        <span class="font-medium text-[var(--color-text-primary)]">{cpid}</span>
      </div>
    {/if}
  </div>
</div>
