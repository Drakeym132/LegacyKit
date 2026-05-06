import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DeviceInfo } from '$lib/stores/deviceStore.svelte';

export type RestoreOptionKind =
  | 'otaDowngrade'
  | 'powdersnow'
  | 'latest'
  | 'blobRestore'
  | 'tethered'
  | 'customIpsw'
  | 'dfuIpsw'
  | 'setNonce'
  | 'ipswDownloader'
  | 'moreVersions';

export interface RestoreOption {
  kind: RestoreOptionKind;
  title: string;
  description: string;
  targetVersion: string | null;
  requiresBlobs: boolean;
  requiresDfu: boolean;
}

export interface RestoreOptionsResponse {
  productType: string | null;
  processorGeneration: number | null;
  options: RestoreOption[];
  warnings: string[];
}

export interface IpswDownloadRequest {
  url: string;
  outputDir: string;
  deviceIdentifier?: string | null;
  fileName: string | null;
  expectedSha1: string | null;
  downloadId: string | null;
}

export interface IpswDownloadResult {
  path: string;
  sha1: string;
  expectedSha1: string | null;
  sha1Matches: boolean | null;
  downloadId: string;
}

export interface FirmwareListRequest {
  deviceIdentifier: string;
}

export interface FirmwareListEntry {
  version: string;
  buildId: string;
  url: string;
  sha1: string | null;
  sizeBytes: number | null;
  signed: boolean | null;
}

export interface FirmwareListResult {
  deviceIdentifier: string;
  fetchedAtUnix: number;
  cached: boolean;
  firmwares: FirmwareListEntry[];
}

export interface CheckIpswSigningRequest {
  deviceIdentifier: string;
  buildId: string;
}

export interface CheckIpswSigningResult {
  deviceIdentifier: string;
  buildId: string;
  signed: boolean;
  output: string;
}

export interface CancelIpswDownloadRequest {
  downloadId: string;
}

export interface CancelIpswDownloadResult {
  downloadId: string;
  cancelled: boolean;
}

export interface IpswDownloadProgressEvent {
  downloadId: string;
  percent: number | null;
  downloadedBytes: number | null;
  totalBytes: number | null;
  speedBps: number | null;
  etaSeconds: number | null;
}

export interface IpswVerifyRequest {
  path: string;
  expectedSha1: string | null;
}

export interface IpswVerifyResult {
  path: string;
  calculatedSha1: string;
  expectedSha1: string | null;
  matches: boolean | null;
}

export type RestoreTool = 'ideviceRestore' | 'futureRestore';

export interface RestoreRunRequest {
  tool: RestoreTool;
  ipswPath: string;
  shshPath: string | null;
  erase: boolean;
  update: boolean;
  usePwndfu: boolean;
  skipBlob: boolean;
  setNonce: boolean;
  noBaseband: boolean;
  latestSep: boolean;
  latestBaseband: boolean;
  dryRun: boolean;
}

export interface RestoreCommandPreview {
  supported: boolean;
  tool: RestoreTool;
  binary: string;
  args: string[];
  warnings: string[];
}

export interface IpswPrepareRequest {
  ipswPath: string;
  outputDir: string;
  deviceIdentifier?: string | null;
  shshPath: string | null;
  deviceEcid: string | null;
}

export interface IpswPrepareResult {
  outputPath: string;
}

export function getRestoreOptions(device: DeviceInfo): Promise<RestoreOptionsResponse> {
  return invoke<RestoreOptionsResponse>('get_restore_options', { device });
}

export function downloadIpsw(request: IpswDownloadRequest): Promise<IpswDownloadResult> {
  return invoke<IpswDownloadResult>('download_ipsw', { request });
}

export function listFirmwares(request: FirmwareListRequest): Promise<FirmwareListResult> {
  return invoke<FirmwareListResult>('list_firmwares', { request });
}

export interface ExistingIpswEntry {
  path: string;
  fileName: string;
  sizeBytes: number;
  deviceIdentifier: string | null;
}

export interface ListExistingIpswsRequest {
  deviceIdentifier?: string | null;
}

export interface ListExistingIpswsResult {
  ipsws: ExistingIpswEntry[];
}

export function listExistingIpsws(
  request: ListExistingIpswsRequest,
): Promise<ListExistingIpswsResult> {
  return invoke<ListExistingIpswsResult>('list_existing_ipsws', { request });
}

export function checkIpswSigning(
  request: CheckIpswSigningRequest,
): Promise<CheckIpswSigningResult> {
  return invoke<CheckIpswSigningResult>('check_ipsw_signing', { request });
}

export function cancelIpswDownload(
  request: CancelIpswDownloadRequest,
): Promise<CancelIpswDownloadResult> {
  return invoke<CancelIpswDownloadResult>('cancel_ipsw_download', { request });
}

export function onIpswDownloadProgress(
  handler: (event: IpswDownloadProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<IpswDownloadProgressEvent>('ipsw-download-progress', (e) => handler(e.payload));
}

export function verifyIpsw(request: IpswVerifyRequest): Promise<IpswVerifyResult> {
  return invoke<IpswVerifyResult>('verify_ipsw', { request });
}

export function prepareIpsw(request: IpswPrepareRequest): Promise<IpswPrepareResult> {
  return invoke<IpswPrepareResult>('prepare_ipsw', { request });
}

export function previewRestoreCommand(request: RestoreRunRequest): Promise<RestoreCommandPreview> {
  return invoke<RestoreCommandPreview>('preview_restore_command', { request });
}

export function startRestore(request: RestoreRunRequest): Promise<RestoreCommandPreview> {
  return invoke<RestoreCommandPreview>('start_restore', { request });
}
