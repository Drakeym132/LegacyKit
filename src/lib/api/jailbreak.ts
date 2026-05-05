import { invoke } from '@tauri-apps/api/core';
import type { DeviceMode } from '$lib/stores/deviceStore.svelte';

export type GasterAction = 'pwn' | 'reset';

export interface GasterRequest {
  action: GasterAction;
}

export interface GasterResult {
  action: GasterAction;
  binary: string;
  args: string[];
}

export function runGaster(request: GasterRequest): Promise<GasterResult> {
  return invoke<GasterResult>('run_gaster', { request });
}

/**
 * @deprecated Use sendBootchain instead.
 * kloader is an ARM binary that runs ON the iOS device, not on the host.
 * This function is kept for backwards compatibility but will return an error.
 */
export interface KloaderRequest {
  ibssPath: string;
  ibecPath: string | null;
}

/**
 * @deprecated Use SendBootchainResult instead.
 */
export interface KloaderResult {
  binary: string;
  args: string[];
}

/**
 * @deprecated Use sendBootchain instead. This command was incorrectly trying
 * to run kloader as a host binary. kloader is an ARM binary that runs on the
 * iOS device, not on the host computer.
 */
export function runKloader(request: KloaderRequest): Promise<KloaderResult> {
  return invoke<KloaderResult>('run_kloader', { request });
}

/**
 * Request for sending bootchain components to a device in pwnDFU mode.
 * Uses irecovery -f to send patched iBSS/iBEC and decrypted DeviceTree/Kernelcache.
 */
export interface SendBootchainRequest {
  /** Path to the patched iBSS file */
  ibssPath: string;
  /** Optional path to the patched iBEC file */
  ibecPath: string | null;
  /** Optional path to the decrypted DeviceTree file */
  deviceTreePath?: string | null;
  /** Optional path to the decrypted Kernelcache file */
  kernelcachePath?: string | null;
  /** Processor generation (e.g., 6 for A6). Used to determine if gaster reset is needed. */
  processorGeneration: number | null;
}

/**
 * Sends patched iBSS/iBEC and decrypted DeviceTree/Kernelcache to a device
 * in pwnDFU mode using irecovery. This is the correct flow for tethered boot.
 *
 * For A6 devices, this will also reset gaster before sending the bootchain.
 * After iBEC is delivered the backend waits for USB re-enumeration into
 * recovery PID 0x1281, then stages DeviceTree (`devicetree`) and Kernelcache
 * (`bootx`) so the device boots into the OS.
 */
export function sendBootchain(request: SendBootchainRequest): Promise<void> {
  return invoke<void>('send_bootchain', { request });
}

export interface UntetherRequest {
  extraArgs: string[];
}

export interface UntetherResult {
  binary: string;
  args: string[];
}

export function runG1lbertJB(request: UntetherRequest): Promise<UntetherResult> {
  return invoke<UntetherResult>('run_g1lbertjb', { request });
}

export function runEvasi0n(request: UntetherRequest): Promise<UntetherResult> {
  return invoke<UntetherResult>('run_evasi0n', { request });
}

export interface EnterPwnDfuRequest {
  productType: string;
}

export interface EnterPwnDfuResult {
  tool: string;
  args: string[];
  pwnd: string | null;
  mode: DeviceMode;
}

export function enterPwndfu(request: EnterPwnDfuRequest): Promise<EnterPwnDfuResult> {
  return invoke<EnterPwnDfuResult>('enter_pwndfu', { request });
}

export interface DownloadPwnToolRequest {
  tool: 'kuroutadori';
}

export interface DownloadPwnToolResult {
  tool: string;
  binaryPath: string;
}

export function downloadPwnTool(request: DownloadPwnToolRequest): Promise<DownloadPwnToolResult> {
  return invoke<DownloadPwnToolResult>('download_pwn_tool', { request });
}

// ============================================================================
// kDFU Mode Support (for jailbroken devices with SSH access)
// ============================================================================

/**
 * kloader variant selection for kDFU mode.
 */
export type KloaderVariant = 'standard' | 'kloader5' | 'axi0mX';

/**
 * Request for getting the path to a kloader binary.
 */
export interface GetKloaderPathRequest {
  variant: KloaderVariant;
}

/**
 * Result of getting the kloader path.
 */
export interface GetKloaderPathResult {
  /** Path to the kloader binary in the app's resources */
  path: string;
  /** The variant that was resolved */
  variant: KloaderVariant;
}

/**
 * Returns the path to a kloader binary in the app's resources.
 *
 * IMPORTANT: kloader is an ARM binary that runs ON the iOS device, not on the host.
 * This command returns the path so the frontend can send it to the device via SSH.
 */
export function getKloaderPath(request: GetKloaderPathRequest): Promise<GetKloaderPathResult> {
  return invoke<GetKloaderPathResult>('get_kloader_path', { request });
}

/**
 * Instructions for entering kDFU mode from a jailbroken device.
 */
export interface KdfuInstructions {
  /** Path to the kloader binary (send to device via SCP) */
  kloaderPath: string;
  /** Path to the patched iBSS file (send to device via SCP) */
  ibssPath: string;
  /** SSH command to run kloader on the device */
  sshCommand: string;
  /** The kloader variant used */
  kloaderVariant: KloaderVariant;
}

/**
 * Returns instructions for entering kDFU mode from a jailbroken device.
 *
 * Prerequisites:
 * - Device must be jailbroken with OpenSSH installed
 * - Device must be in Normal mode with SSH accessible
 * - For iOS 10, Dropbear must be installed instead of OpenSSH
 *
 * @param ibssPath - Path to the patched iBSS file
 * @param iosMajor - Major iOS version (e.g., 6 for iOS 6.1.3)
 * @param productType - Device product type (e.g., "iPhone5,1")
 * @param sshPort - SSH port (default: 22)
 */
export function getKdfuInstructions(
  ibssPath: string,
  iosMajor: number,
  productType: string,
  sshPort?: number
): Promise<KdfuInstructions> {
  return invoke<KdfuInstructions>('get_kdfu_instructions', {
    ibssPath,
    iosMajor,
    productType,
    sshPort
  });
}
