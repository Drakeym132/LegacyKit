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

export interface KloaderRequest {
  ibssPath: string;
  ibecPath: string | null;
}

export interface KloaderResult {
  binary: string;
  args: string[];
}

export function runKloader(request: KloaderRequest): Promise<KloaderResult> {
  return invoke<KloaderResult>('run_kloader', { request });
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
