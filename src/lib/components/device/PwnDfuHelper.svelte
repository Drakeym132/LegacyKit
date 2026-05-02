<script lang="ts">
  import { deviceStore } from '$lib/stores/deviceStore.svelte';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { toastStore } from '$lib/stores/toastStore.svelte';
  import { settingsStore } from '$lib/stores/settingsStore.svelte';
  import { enterPwndfu } from '$lib/api/jailbreak';
  import { enterRecovery } from '$lib/api/utilities';
  import { inferProcessorGen } from '$lib/utils/processorGen';
  import DfuHelper from './DfuHelper.svelte';

  let device = $derived(deviceStore.state);
  let mode = $derived(device.mode);
  let productType = $derived(device.product_type);
  let procGen = $derived(inferProcessorGen(productType));

  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);

  async function handleEnterRecovery() {
    if (!device.udid) {
      errorMessage = 'Pair the device in Normal mode first.';
      return;
    }
    isWorking = true;
    errorMessage = null;
    try {
      await enterRecovery({ udid: device.udid });
      logStore.append('Sending device to Recovery...', 'info');
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isWorking = false;
    }
  }

  async function handleEnterPwndfu() {
    if (!productType) {
      errorMessage = 'Device product type is unknown.';
      return;
    }
    isWorking = true;
    errorMessage = null;
    try {
      const result = await enterPwndfu({ productType });
      deviceStore.optimisticallySetMode(result.mode, result.pwnd);
      const label = result.pwnd ? `Pwned (${result.pwnd}) via ${result.tool}` : `Pwned via ${result.tool}`;
      logStore.append(label, 'info');
      toastStore.success('pwnDFU entered', label);
      // Start boosting the poller AFTER we get the result, so the next poll
      // runs immediately with fresh data instead of stale data from before pwn.
      settingsStore.boostPolling();
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : String(err);
      logStore.append(`enter_pwndfu failed: ${errorMessage}`, 'stderr');
    } finally {
      isWorking = false;
    }
  }
</script>

<div class="pwndfu-helper" class:disconnected={!device.connected}>
  {#if !device.connected}
    <div class="warning-banner">
      Connect a device to begin.
    </div>
  {:else if mode === 'pwnDFU' || (procGen === 6 && mode === 'kDFU')}
    <div class="success-banner">
      Device is in {mode}{device.pwnd ? ` (pwned: ${device.pwnd})` : ''} — ready to boot.
    </div>
  {:else if mode === 'Normal'}
    <div class="step-card">
      <div class="step-body">
        <strong>Step 1 of 3 — Send to Recovery</strong>
        <p>Then you'll guide the device into DFU and run the pwn exploit.</p>
      </div>
      <button class="primary" onclick={handleEnterRecovery} disabled={isWorking || !device.udid}>
        {isWorking ? 'Working…' : 'Enter Recovery'}
      </button>
    </div>
  {:else if mode === 'Recovery'}
    <div class="step-card stack">
      <div class="step-body">
        <strong>Step 2 of 3 — Enter DFU</strong>
        <p>Use the timed guide below. The mode pill flips to DFU when the device is ready.</p>
      </div>
      <DfuHelper />
    </div>
  {:else if mode === 'DFU'}
    {#if procGen === null}
      <div class="warning-banner">
        Unknown processor for {productType ?? 'this device'} — can't choose a pwn tool.
      </div>
    {:else if procGen === 5}
      <div class="warning-banner">
        A5 / A5X devices need external hardware (Arduino + USB Host Shield, or Pi Pico) for
        checkm8-a5. Use the SSH Ramdisk → kDFU path instead, or jailbreak the device and use
        kDFU over OpenSSH.
      </div>
    {:else}
      <div class="step-card">
        <div class="step-body">
          <strong>Step 3 of 3 — Enter pwnDFU</strong>
          <p>
            Runs the right exploit for A{procGen} ({productType}). The device will stay on a
            black screen — that's expected.
          </p>
        </div>
        <button class="primary" onclick={handleEnterPwndfu} disabled={isWorking}>
          {isWorking ? 'Pwning…' : 'Enter pwnDFU'}
        </button>
      </div>
    {/if}
  {:else}
    <div class="warning-banner">
      Unsupported mode for booting: {mode}.
    </div>
  {/if}

  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}
</div>

<style>
  .pwndfu-helper {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  .warning-banner {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) 40%, transparent);
    color: var(--color-warning-text, var(--color-text-primary));
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .success-banner {
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 40%, transparent);
    color: var(--color-success);
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .step-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm) var(--spacing-md);
  }

  .step-card.stack {
    flex-direction: column;
    align-items: stretch;
  }

  .step-body {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .step-body strong {
    color: var(--color-text-primary);
    font-size: 0.875rem;
  }

  .step-body p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    line-height: 1.45;
  }

  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
    border: 1px solid transparent;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: white;
  }

  .error {
    color: var(--color-danger);
    font-size: 0.8125rem;
    background: color-mix(in srgb, var(--color-danger) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 35%, transparent);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
  }
</style>
