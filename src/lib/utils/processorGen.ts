import type { RestoreOptionsResponse } from '$lib/api/restore';

/**
 * Extract processor generation from a RestoreOptionsResponse.
 * This is the preferred way to get processor generation since the backend
 * already computes it via infer_processor_gen.
 */
export function processorGenFromResponse(response: RestoreOptionsResponse): number | null {
  return response.processorGeneration ?? null;
}

/**
 * Infer processor generation from a device product type.
 *
 * The backend exposes processor generation only as a field on
 * `RestoreOptionsResponse` (computed via `infer_processor_gen` in
 * `src-tauri/src/services/device_meta.rs`); it is *not* present on
 * `DeviceInfo` and is not exposed as a standalone IPC command. Therefore:
 *
 * - Views that already fetch restore options (e.g. `RestoreView.svelte`)
 *   should prefer `processorGenFromResponse` to read the authoritative value.
 * - Views/components that do not fetch restore options
 *   (`JailbreakView`, `SSHRamdiskView`, `JustBootDialog`, `PwnDfuHelper`)
 *   should call `inferProcessorGen` directly. This table is the canonical
 *   frontend mirror of the Rust implementation; keep them in sync.
 *
 * @param product - Apple device product identifier (e.g. "iPhone6,1")
 * @returns Processor generation number or null if unknown
 */
export function inferProcessorGen(product: string | null): number | null {
  if (!product) return null;
  if (/^iPhone(1|2),/.test(product) || /^iPod(1|2),/.test(product)) return 1;
  if (product === 'iPod3,1') return 3;
  if (/^iPhone3,/.test(product) || product === 'iPad1,1' || product === 'iPod4,1') return 4;
  if (product === 'iPhone4,1' || /^iPad2,/.test(product) || /^iPad3,[1-3]/.test(product) || product === 'iPod5,1') return 5;
  if (/^iPhone5,/.test(product) || /^iPad3,[4-6]/.test(product)) return 6;
  if (/^iPhone6,/.test(product) || /^iPad4,/.test(product)) return 7;
  if (/^iPhone7,/.test(product) || product === 'iPod7,1' || /^iPad5,/.test(product)) return 8;
  if (/^iPhone8,/.test(product) || /^iPad6,/.test(product)) return 9;
  if (/^iPhone9,/.test(product) || /^iPad7,/.test(product)) return 10;
  return null;
}
