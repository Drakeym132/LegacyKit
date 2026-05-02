import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface UdidRequest {
  udid: string | null;
}

export interface CommandRunResult {
  args: string[];
}

export type ExportInfoKind = 'device-info' | 'battery-info' | 'diagnostics-all';

export interface ExportInfoRequest {
  udid: string | null;
  outputDir?: string | null;
  kind: ExportInfoKind;
  label?: string | null;
}

export interface ExportInfoResult {
  path: string;
  bytes: number;
}

export type ActivationAction = 'activate' | 'deactivate' | 'state';

export interface ActivationRequest {
  udid: string | null;
  action: ActivationAction;
}

export interface ActivationResult {
  action: ActivationAction;
  state: string | null;
  args: string[];
}

export type PairAction = 'pair' | 'unpair' | 'validate';

export interface PairRequest {
  udid: string | null;
  action: PairAction;
}

export interface PairResult {
  action: PairAction;
  args: string[];
}

export type DiagnosticsAction = 'shutdown' | 'restart' | 'sleep';

export interface DiagnosticsRequest {
  udid: string | null;
  action: DiagnosticsAction;
}

export interface DiagnosticsResult {
  action: DiagnosticsAction;
  args: string[];
}

export interface IrecoveryCommandRequest {
  commands: string[];
  rebootAfter: boolean;
}

export interface IrecoveryCommandResult {
  args: string[];
  commands: string[];
}

export interface SyslogStartRequest {
  udid: string | null;
}

export interface SyslogStatusResult {
  running: boolean;
  pid: number | null;
}

export interface SyslogEvent {
  text: string;
  type: 'stdout' | 'stderr' | 'info';
}

export function enterRecovery(request: UdidRequest): Promise<CommandRunResult> {
  return invoke<CommandRunResult>('enter_recovery', { request });
}

export function exitRecovery(): Promise<CommandRunResult> {
  return invoke<CommandRunResult>('exit_recovery');
}

export function runDiagnosticsAction(request: DiagnosticsRequest): Promise<DiagnosticsResult> {
  return invoke<DiagnosticsResult>('run_diagnostics_action', { request });
}

export function pairDevice(request: PairRequest): Promise<PairResult> {
  return invoke<PairResult>('pair_device', { request });
}

export function runActivationAction(request: ActivationRequest): Promise<ActivationResult> {
  return invoke<ActivationResult>('run_activation_action', { request });
}

export function exportDeviceInfo(request: ExportInfoRequest): Promise<ExportInfoResult> {
  return invoke<ExportInfoResult>('export_device_info', { request });
}

export function runIrecoveryCommands(
  request: IrecoveryCommandRequest,
): Promise<IrecoveryCommandResult> {
  return invoke<IrecoveryCommandResult>('run_irecovery_commands', { request });
}

export function clearNvram(): Promise<IrecoveryCommandResult> {
  return invoke<IrecoveryCommandResult>('clear_nvram');
}

export function startSyslog(request: SyslogStartRequest): Promise<SyslogStatusResult> {
  return invoke<SyslogStatusResult>('start_syslog', { request });
}

export function stopSyslog(): Promise<SyslogStatusResult> {
  return invoke<SyslogStatusResult>('stop_syslog');
}

export function syslogStatus(): Promise<SyslogStatusResult> {
  return invoke<SyslogStatusResult>('syslog_status');
}

export function onSyslogEvent(handler: (event: SyslogEvent) => void): Promise<UnlistenFn> {
  return listen<SyslogEvent>('syslog_event', (e) => handler(e.payload));
}
