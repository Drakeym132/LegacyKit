import { invoke } from '@tauri-apps/api/core';

export interface TrollStorePrepareRequest {
  savedDir: string;
  forceVersion?: string | null;
}

export interface TrollStorePrepareResult {
  version: string;
  tarPath: string;
  helperPath: string;
  cached: boolean;
}

export interface TrollStoreEligibilityRequest {
  productType: string | null;
  iosVersion: string | null;
}

export type TrollStorePath =
  | 'ios14-15-ramdisk'
  | 'ios16-trollrestore'
  | 'incompatible'
  | 'unknown';

export interface TrollStoreEligibilityResult {
  path: TrollStorePath;
  reason: string;
  iosMajor: number | null;
}

export function prepareTrollstoreAssets(
  request: TrollStorePrepareRequest,
): Promise<TrollStorePrepareResult> {
  return invoke<TrollStorePrepareResult>('prepare_trollstore_assets', { request });
}

export function checkTrollstoreEligibility(
  request: TrollStoreEligibilityRequest,
): Promise<TrollStoreEligibilityResult> {
  return invoke<TrollStoreEligibilityResult>('check_trollstore_eligibility', { request });
}
