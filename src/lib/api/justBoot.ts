import { invoke } from '@tauri-apps/api/core';

export interface JustBootEntry {
  id: string;
  ecid: string;
  productType: string;
  deviceName: string | null;
  buildId: string;
  iosVersion: string | null;
  bootArgs: string | null;
  repackedIbssPath: string | null;
  repackedIbecPath: string | null;
  sourceIpswPath: string | null;
  createdAt: string;
  lastBootedAt: string;
}

export interface JustBootEntryInput {
  ecid: string;
  productType: string;
  deviceName?: string | null;
  buildId: string;
  iosVersion?: string | null;
  bootArgs?: string | null;
  repackedIbssPath?: string | null;
  repackedIbecPath?: string | null;
  sourceIpswPath?: string | null;
}

export interface PrepareAndJustBootRequest {
  ecid: string;
  productType: string;
  deviceName?: string | null;
  buildId: string;
  iosVersion?: string | null;
  ipswPath: string;
  bootArgs?: string | null;
  processorGeneration?: string | null;
  includeIbec: boolean;
}

export function listJustBootHistory(): Promise<JustBootEntry[]> {
  return invoke<JustBootEntry[]>('list_just_boot_history');
}

export function recordJustBoot(entry: JustBootEntryInput): Promise<JustBootEntry> {
  return invoke<JustBootEntry>('record_just_boot', { entry });
}

export function forgetJustBoot(id: string): Promise<void> {
  return invoke<void>('forget_just_boot', { id });
}

export function prepareAndJustBoot(request: PrepareAndJustBootRequest): Promise<JustBootEntry> {
  return invoke<JustBootEntry>('prepare_and_just_boot', { request });
}
