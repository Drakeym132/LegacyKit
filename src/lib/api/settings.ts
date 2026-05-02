import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  workspaceRoot: string | null;
  onboarded: boolean;
  glassChrome?: boolean;
}

export interface WorkspacePaths {
  root: string;
  ipsw: string;
  ipswCustom: string;
  shsh: string;
  extracted: string;
  sshBinaries: string;
  backups: string;
  logs: string;
  tmp: string;
}

export function getAppSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_app_settings');
}

export function setWorkspaceRoot(path: string): Promise<AppSettings> {
  return invoke<AppSettings>('set_workspace_root', { request: { path } });
}

export function completeOnboarding(): Promise<AppSettings> {
  return invoke<AppSettings>('complete_onboarding');
}

export function ensureWorkspaceLayout(): Promise<WorkspacePaths> {
  return invoke<WorkspacePaths>('ensure_workspace_layout');
}

export function revealWorkspace(): Promise<void> {
  return invoke<void>('reveal_workspace');
}

export function setWindowShadow(enabled: boolean): Promise<void> {
  return invoke<void>('set_window_shadow', { enabled });
}
