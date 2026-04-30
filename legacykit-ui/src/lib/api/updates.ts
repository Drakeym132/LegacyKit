import { invoke } from '@tauri-apps/api/core';

export interface UpdateCheckRequest {
  repo: string;
  currentVersion: string;
}

export interface UpdateCheckResult {
  current: string;
  latest: string;
  releaseUrl: string | null;
  updateAvailable: boolean;
}

export function checkForUpdates(request: UpdateCheckRequest): Promise<UpdateCheckResult> {
  return invoke<UpdateCheckResult>('check_for_updates', { request });
}
