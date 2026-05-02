import { logStore } from '$lib/stores/logStore.svelte';
import { toastStore } from '$lib/stores/toastStore.svelte';

export interface WorkingController {
  /** Reactive: true while a `run` call is in flight. */
  readonly isWorking: boolean;
  /** Reactive: last error message (cleared at the start of every `run`). */
  readonly errorMessage: string | null;
  /** Manually clear the current error message. */
  clearError(): void;
  /** Manually set the current error message (used for client-side validation). */
  setError(msg: string | null): void;
  /**
   * Wrap an async call with logging, toasts, and busy-state tracking.
   * Returns the result on success, or null on failure (does not rethrow).
   */
  run<T>(label: string, fn: () => Promise<T>): Promise<T | null>;
}

export function createWorkingController(): WorkingController {
  let isWorking = $state(false);
  let errorMessage = $state<string | null>(null);

  return {
    get isWorking() {
      return isWorking;
    },
    get errorMessage() {
      return errorMessage;
    },
    clearError() {
      errorMessage = null;
    },
    setError(msg: string | null) {
      errorMessage = msg;
    },
    async run<T>(label: string, fn: () => Promise<T>): Promise<T | null> {
      isWorking = true;
      errorMessage = null;
      logStore.append(`${label}...`, 'info');
      try {
        const result = await fn();
        logStore.append(`${label} ok`, 'info');
        toastStore.success(label, 'Completed');
        return result;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        errorMessage = msg;
        logStore.append(`${label} failed: ${msg}`, 'stderr');
        toastStore.error(`${label} failed`, msg);
        return null;
      } finally {
        isWorking = false;
      }
    },
  };
}
