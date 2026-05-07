<script lang="ts">
  import { deviceStore } from '$lib/stores/deviceStore.svelte';
  import { logStore } from '$lib/stores/logStore.svelte';
  import { toastStore } from '$lib/stores/toastStore.svelte';
  import { enterPwndfu } from '$lib/api/jailbreak';
  import { inferProcessorGen } from '$lib/utils/processorGen';
  import DfuHelper from './DfuHelper.svelte';

  let device = $derived(deviceStore.state);
  let mode = $derived(device.mode);
  let productType = $derived(device.product_type);
  let procGen = $derived(inferProcessorGen(productType));

  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);

  type StepKey = 'connect' | 'dfu' | 'pwndfu';
  const STEP_ORDER: StepKey[] = ['connect', 'dfu', 'pwndfu'];

  let isReady = $derived(mode === 'pwnDFU' || (procGen === 6 && mode === 'kDFU'));
  let isDisconnected = $derived(!device.connected);

  // Recovery maps to 'connect' — it's part of the "getting to DFU" phase
  let currentStep = $derived<StepKey>(
    isReady ? 'pwndfu' : mode === 'DFU' ? 'dfu' : 'connect'
  );

  let progress = $derived(STEP_ORDER.indexOf(currentStep) / (STEP_ORDER.length - 1));

  function stepState(
    key: StepKey,
  ): 'done' | 'active' | 'pending' | 'disabled' {
    if (isDisconnected && key === 'connect') return 'disabled';
    if (isReady) return 'done';
    const ci = STEP_ORDER.indexOf(currentStep);
    const ki = STEP_ORDER.indexOf(key);
    if (ki < ci) return 'done';
    if (ki === ci) return 'active';
    return 'pending';
  }

  let firstStepLabel = $derived(
    isDisconnected ? 'Connect' : mode === 'Recovery' ? 'Recovery' : 'Normal'
  );
  let isA5InDfu = $derived(procGen === 5 && mode === 'DFU');
  let unknownProcInDfu = $derived(mode === 'DFU' && procGen === null);

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
      const label = result.pwnd
        ? `Pwned (${result.pwnd}) via ${result.tool}`
        : `Pwned via ${result.tool}`;
      logStore.append(label, 'info');
      toastStore.success('pwnDFU entered', label);
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : String(err);
      logStore.append(`enter_pwndfu failed: ${errorMessage}`, 'stderr');
    } finally {
      isWorking = false;
    }
  }
</script>

<div class="strip">
  <div class="progress" class:ready={isReady} style="--progress: {progress}">
    <div class="track-base"></div>
    <div class="track-fill"></div>

    <div class="node" data-edge="start" style="--pos: 0">
      <span class="bullet" data-state={stepState('connect')}></span>
      <span class="label" data-state={stepState('connect')}>{firstStepLabel}</span>
    </div>
    <div class="node" data-edge="mid" style="--pos: 0.5">
      <span class="bullet" data-state={stepState('dfu')}></span>
      <span class="label" data-state={stepState('dfu')}>DFU</span>
    </div>
    <div class="node" data-edge="end" style="--pos: 1">
      <span class="bullet" data-state={stepState('pwndfu')}></span>
      <span class="label" data-state={stepState('pwndfu')}>{isReady ? 'pwnDFU — ready to boot' : 'pwnDFU'}</span>
    </div>
  </div>

  {#if isReady}
    <!-- ready state shown via green progress bar -->
  {:else if isDisconnected}
    <div class="step-row">
      <div class="step-text">
        <strong>Connect device to begin</strong>
      </div>
    </div>
  {:else if isA5InDfu}
    <div class="warning-banner">
      A5 / A5X devices need external hardware (Arduino + USB Host Shield, or Pi Pico) for checkm8-a5.
      Use the SSH Ramdisk → kDFU path instead.
    </div>
  {:else if unknownProcInDfu}
    <div class="warning-banner">
      Unknown processor for {productType ?? 'this device'} — can't choose a pwn tool.
    </div>
  {:else if mode === 'Normal' || mode === 'Recovery'}
    <DfuHelper />
  {:else if mode === 'DFU'}
    <div class="step-row">
      <div class="step-text">
        <strong>Step 2 of 2 — Enter pwnDFU</strong>
        <span>Runs the right exploit for A{procGen} ({productType}). Black screen is expected.</span>
      </div>
      <button class="primary" onclick={handleEnterPwndfu} disabled={isWorking}>
        {isWorking ? 'Pwning…' : 'Enter pwnDFU'}
      </button>
    </div>
  {:else}
    <div class="warning-banner">Unsupported mode for booting: {mode}.</div>
  {/if}

  {#if errorMessage}
    <div class="error">{errorMessage}</div>
  {/if}
</div>

<style>
  .strip {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
  }

  /* === Progress track === */
  .progress {
    position: relative;
    height: 38px;
    width: 100%;
    box-sizing: border-box;
  }

  .track-base,
  .track-fill {
    position: absolute;
    top: 6px;
    height: 2px;
    margin-top: -1px;
    border-radius: 1px;
    pointer-events: none;
  }

  .track-base {
    left: 6px;
    right: 6px;
    background: var(--color-border);
  }

  .track-fill {
    left: 6px;
    width: calc((100% - 12px) * var(--progress, 0));
    background: var(--color-accent);
    transition:
      width 0.6s cubic-bezier(0.65, 0, 0.35, 1),
      background 0.4s ease;
  }

  .progress.ready .track-fill {
    background: var(--color-success);
  }

  .progress.ready .bullet[data-state='done'],
  .progress.ready .bullet[data-state='active'] {
    background: var(--color-success);
    border-color: var(--color-success);
  }

  .node {
    position: absolute;
    top: 0;
    left: calc(6px + (100% - 12px) * var(--pos));
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }

  .node[data-edge='start'] {
    transform: translateX(-6px);
    align-items: flex-start;
  }

  .node[data-edge='mid'] {
    transform: translateX(-50%);
  }

  .node[data-edge='end'] {
    transform: translateX(calc(-100% + 6px));
    align-items: flex-end;
  }

  .bullet {
    display: block;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-bg-primary);
    border: 1.5px solid var(--color-border);
    flex-shrink: 0;
    z-index: 1;
    transition:
      background 0.35s cubic-bezier(0.65, 0, 0.35, 1),
      border-color 0.35s cubic-bezier(0.65, 0, 0.35, 1),
      box-shadow 0.35s cubic-bezier(0.65, 0, 0.35, 1);
  }

  .bullet[data-state='done'] {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }

  .bullet[data-state='active'] {
    background: var(--color-accent);
    border-color: var(--color-accent);
    animation: pulse 1.8s ease-in-out infinite;
  }

  .bullet[data-state='disabled'] {
    background: var(--color-bg-primary);
    border-color: color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 8%, transparent);
    opacity: 0.6;
  }

  @keyframes pulse {
    0%,
    100% {
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-accent) 28%, transparent);
    }
    50% {
      box-shadow: 0 0 0 6px color-mix(in srgb, var(--color-accent) 0%, transparent);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .bullet[data-state='active'] {
      animation: none;
    }
    .track-fill {
      transition: none;
    }
  }

  /* === Labels === */
  .label {
    white-space: nowrap;
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--color-text-tertiary, var(--color-text-secondary));
    transition: color 0.25s ease;
  }

  .label[data-state='done'],
  .label[data-state='active'] {
    color: var(--color-text-primary);
  }

  .label[data-state='disabled'] {
    color: var(--color-text-secondary);
    opacity: 0.7;
  }

  /* === Step row prompt === */
  .step-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-top: 4px;
  }

  .step-row.stack {
    flex-direction: column;
    align-items: stretch;
  }

  .step-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .step-text strong {
    color: var(--color-text-primary);
    font-size: 0.85rem;
  }

  .step-text span {
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    line-height: 1.4;
  }

  /* === Ready pill === */
  .ready-pill {
    display: inline-flex;
    align-self: flex-start;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 40%, transparent);
    color: var(--color-success);
    border-radius: 999px;
    font-size: 0.8125rem;
    font-weight: 600;
    margin-top: 4px;
  }

  .ready-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--color-success);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-success) 25%, transparent);
  }

  .warning-banner {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) 40%, transparent);
    color: var(--color-warning-text, var(--color-text-primary));
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    font-weight: 500;
    margin-top: 4px;
  }

  button {
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    font-weight: 600;
    padding: 8px 14px;
    cursor: pointer;
    border: 1px solid transparent;
    flex-shrink: 0;
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
