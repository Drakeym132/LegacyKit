# LegacyKit UI Rebuild — Progress Report

**Date:** 2026-04-30  
**Status:** Phases 1–7 Complete, Phase 8 Complete (with documented deferred items)

---

## Completed Changes

### Phase 1: Foundation (100% Complete)

#### Rust Backend
| File | Action | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modified | Added `thiserror = "2"` dependency |
| `src-tauri/src/error.rs` | Created | Unified `AppError` enum (DeviceNotFound, CommandFailed, Io, Parse) with thiserror + Serialize |
| `src-tauri/src/models/mod.rs` | Created | Module declaration for `device` |
| `src-tauri/src/models/device.rs` | Created | `DeviceInfo` struct and `DeviceMode` enum (Normal, Recovery, DFU, kDFU, pwnDFU) |
| `src-tauri/src/commands/mod.rs` | Created | Module declaration for `device` |
| `src-tauri/src/commands/device.rs` | Created | `detect_device` command — parses `ideviceinfo` output, falls back to `irecovery -q` for Recovery/DFU |
| `src-tauri/src/lib.rs` | Modified | Registered `tauri-plugin-log`, added `detect_device` command, declared `error`, `models`, `commands` modules |

#### Frontend Cleanup
| File | Action | Description |
|------|--------|-------------|
| `src/lib/components/device/DeviceStatus.svelte` | Modified | Converted from Svelte 4 (`export let`, `$:`) to Svelte 5 runes (`$props()`, `$derived()`) |
| `src/lib/Counter.svelte` | Deleted | Removed unused Tauri template scaffold file |

### Phase 2: Device Detection & Core UI (100% Complete)

#### Stores
| File | Action | Description |
|------|--------|-------------|
| `src/lib/stores/navigationStore.svelte.ts` | Created | `ViewName` type (9 views) + `NavigationStore` class with `navigate()`, `goBack()` |
| `src/lib/stores/settingsStore.svelte.ts` | Created | Theme, terminal visibility, terminal height, auto-detect, poll interval settings |
| `src/lib/stores/deviceStore.svelte.ts` | Modified | Enriched with full fields (ecid, serial, model, product_type, ios_version, mode) + `updateFromBackend()` |

#### Views
| File | Action | Description |
|------|--------|-------------|
| `src/lib/views/HomeView.svelte` | Created | Welcome page with device summary card and quick action navigation grid |
| `src/lib/views/RestoreView.svelte` | Created | Placeholder — Custom IPSW restore, tethered/OTA downgrade, powdersn0w |
| `src/lib/views/JailbreakView.svelte` | Created | Placeholder — checkm8, g1lbertJB, evasi0n, untethered jailbreak |
| `src/lib/views/SHSHView.svelte` | Created | Placeholder — tsschecker, onboard blob dump, Cydia server blobs |
| `src/lib/views/SSHRamdiskView.svelte` | Created | Placeholder — SSH ramdisk boot, file system access, activation bypass |
| `src/lib/views/AppsView.svelte` | Created | Placeholder — IPA install, app dump, TrollStore |
| `src/lib/views/DataView.svelte` | Created | Placeholder — Backup, restore, mount filesystem, erase device |
| `src/lib/views/UtilitiesView.svelte` | Created | Placeholder — Recovery mode, activation, syslog, diagnostics |
| `src/lib/views/SettingsView.svelte` | Created | **Functional** — Theme selector, terminal toggle, poll interval, about section |

#### Navigation Wiring
| File | Action | Description |
|------|--------|-------------|
| `src/lib/components/layout/Sidebar.svelte` | Modified | Replaced local `currentView` state with `navigationStore`; calls `navigationStore.navigate()` |
| `src/lib/components/layout/ContentArea.svelte` | Modified | Renders correct view component based on `navigationStore.currentView` |
| `src/App.svelte` | Modified | Uses `deviceStore.updateFromBackend()` with `detect_device` results; polls with `settingsStore.pollIntervalMs` |

### Follow-up Fixes

| Area | Description |
|------|-------------|
| Frontend config | Added `$lib` alias support for Vite and `svelte-check` |
| Device UI | `DeviceCard` now reads product type, iOS version, and mode from `deviceStore` |
| Settings | Theme, auto-detect polling, terminal visibility, terminal height, and poll interval now affect app behavior |

### Phase 4: Jailbreak & SSH Ramdisk (Complete)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modified | Added `zip = "2"` for IPSW component extraction |
| `src-tauri/src/commands/jailbreak.rs` | Created | `run_gaster` (pwn/reset), `run_kloader` (boot patched iBSS/iBEC), `run_g1lbertjb` and `run_evasi0n` (untether wrappers with passthrough flags) |
| `src-tauri/src/commands/firmware.rs` | Created | `extract_ipsw_component` (Rust zip-based), `patch_iboot` (iBoot32Patcher / iBoot64Patcher), `pack_img4` (img4tool), `repack_img3` (xpwntool), `patch_kernel` (Kernel32Patcher / Kernel64Patcher), and `modify_ramdisk` (hfsplus wrapper: add / remove / resize) |
| `src-tauri/src/commands/mod.rs` | Modified | Registered `firmware` and `jailbreak` modules |
| `src-tauri/src/lib.rs` | Modified | Registered all new invoke handlers |
| `src/lib/api/jailbreak.ts` | Created | Typed `runGaster`, `runKloader`, `runG1lbertJB`, `runEvasi0n` wrappers |
| `src/lib/api/firmware.ts` | Created | Typed `extractIpswComponent`, `patchIboot`, `packImg4`, `repackImg3`, `patchKernel`, `modifyRamdisk` wrappers |
| `src/lib/components/device/DfuHelper.svelte` | Created | Generation-aware DFU guide (Home + Power vs. Side + Volume Down); guided countdown timer; live mode pill |
| `src/lib/views/JailbreakView.svelte` | Replaced | Functional UI: device summary, DFU helper, gaster pwn/reset controls, g1lbertJB / evasi0n untether controls with eligibility + mode warnings |
| `src/lib/views/SSHRamdiskView.svelte` | Replaced | Step-driven build pipeline: extract → patch iBoot/kernel → grow ramdisk → inject SSH binaries → repack as IMG3/IMG4 → kloader boot |

### Phase 5: SHSH Blob Management (Complete)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/models/shsh.rs` | Created | Request/response models for tsschecker save, Cydia batch fetch, onboard raw → SHSH conversion, and saved-blob listing |
| `src-tauri/src/models/mod.rs` | Modified | Registered `shsh` module |
| `src-tauri/src/services/shsh_store.rs` | Created | Directory listing + filename parsing for tsschecker (`ECID_DEVICE_BOARDap_VERSION-BUILD_NONCE.shsh*`) and dash forms; `is_blob_file` filter; sorts by mtime; unit tests |
| `src-tauri/src/services/mod.rs` | Modified | Registered `shsh_store` module |
| `src-tauri/src/commands/shsh.rs` | Created | `save_shsh_blob` (tsschecker wrapper, default generator `0x1111111111111111`, optional APNonce/board/manifest), `fetch_cydia_blobs` (batch via `cydia.saurik.com` with rename-on-success), `dump_onboard_blob` (img4tool `--convert`), `list_saved_blobs`; all stream stdout/stderr via `log_event` |
| `src-tauri/src/commands/mod.rs` | Modified | Registered `shsh` module |
| `src-tauri/src/lib.rs` | Modified | Registered four new invoke handlers |
| `src/lib/api/shsh.ts` | Created | Typed `saveShshBlob`, `fetchCydiaBlobs`, `dumpOnboardBlob`, `listSavedBlobs` wrappers |
| `src/lib/views/SHSHView.svelte` | Replaced | Functional 4-tab UI (tsschecker / Cydia servers / Onboard dump / Library); auto-fills device type/ECID/iOS from `deviceStore`; library view parses tsschecker filenames into device + iOS + build columns and refreshes after every save/fetch/dump |

### Phase 6: App & Data Management (Complete)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/models/apps.rs` | Created | `AppListScope` (User/System/All), `InstalledApp`, `ListAppsRequest/Result`, `InstallIpaRequest/Result`, `UninstallAppRequest/Result` |
| `src-tauri/src/models/data.rs` | Created | `BackupCreateRequest/Result`, `BackupRestoreRequest/Result`, `EraseDeviceRequest/Result`, `BackupEncryptionAction` (On/Off/ChangePassword) + request/result, `BackupEntry`, `ListBackupsRequest/Result` |
| `src-tauri/src/models/mod.rs` | Modified | Registered `apps` and `data` modules |
| `src-tauri/src/commands/apps.rs` | Created | `list_installed_apps` (parses ideviceinstaller CSV output, with quoted-field + doubled-quote support; 4 unit tests), `install_ipa` (multi-IPA), `uninstall_app` |
| `src-tauri/src/commands/data.rs` | Created | `create_backup` (timestamped subdir under root; uses zero-dependency `unix_to_components` for filenames with 3 unit tests), `restore_backup` (--system/--settings/--reboot toggles), `erase_device` (gated on exact "Yes, do as I say" phrase), `set_backup_encryption` (on/off/changepw), `list_backups` (recursive size; sorted newest-first) |
| `src-tauri/src/commands/mod.rs` | Modified | Registered `apps` and `data` modules |
| `src-tauri/src/lib.rs` | Modified | Registered eight new invoke handlers |
| `src/lib/api/apps.ts` | Created | Typed `listInstalledApps`, `installIpa`, `uninstallApp` wrappers |
| `src/lib/api/data.ts` | Created | Typed `createBackup`, `restoreBackup`, `eraseDevice`, `setBackupEncryption`, `listBackups` wrappers + `ERASE_CONFIRMATION` constant |
| `src/lib/views/AppsView.svelte` | Replaced | Functional UI: device summary, multi-IPA install textarea, scope-filtered (User/System/All) installed-app table with bundle/version/display name and per-row uninstall (with confirm) |
| `src/lib/views/DataView.svelte` | Replaced | Functional 4-tab UI: Backup (full toggle), Restore (radio-pick a backup + system/settings/reboot flags), Encryption (on/off/change password), Erase (typed confirmation phrase + native confirm dialog) |

### Phase 7: Utilities & Polish (Complete)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/models/utilities.rs` | Created | Request/response models for recovery, diagnostics, pair, activation, export, irecovery, syslog |
| `src-tauri/src/commands/utilities.rs` | Created | `enter_recovery`, `exit_recovery`, `run_diagnostics_action`, `pair_device`, `run_activation_action`, `export_device_info`, `run_irecovery_commands`, `clear_nvram`, `start_syslog`/`stop_syslog`/`syslog_status` with Mutex-managed long-running child process |
| `src-tauri/src/models/trollstore.rs` | Created | `TrollStorePrepareRequest/Result`, `TrollStoreEligibilityRequest/Result`, `TrollStorePath` enum |
| `src-tauri/src/commands/trollstore.rs` | Created | `prepare_trollstore_assets` (curl GitHub API → aria2c download with version-stamp caching), `check_trollstore_eligibility` (iOS major version routing); 4 unit tests |
| `src-tauri/src/commands/updates.rs` | Created | `check_for_updates` (GitHub releases compare with loose-semver `compare_versions`); 5 unit tests |
| `src-tauri/src/commands/mod.rs` | Modified | Registered `utilities`, `trollstore`, `updates` modules |
| `src-tauri/src/lib.rs` | Modified | Registered 14 new invoke handlers |
| `src/lib/api/utilities.ts` | Created | Typed wrappers + `onSyslogEvent` listener |
| `src/lib/api/trollstore.ts` | Created | Typed wrappers for asset prep + eligibility |
| `src/lib/api/updates.ts` | Created | Typed wrapper for update check |
| `src/lib/views/UtilitiesView.svelte` | Replaced | 5-tab functional UI: Quick actions, irecovery/NVRAM, Syslog (rolling 500-line live viewer), Diagnostics export, TrollStore (eligibility + asset prep + manual install instructions for both ramdisk and TrollRestore paths) |
| `src/lib/stores/toastStore.svelte.ts` | Created | `Toast` type + `info/success/warning/error` variants with auto-expire timers |
| `src/lib/components/common/Toaster.svelte` | Created | Floating bottom-right toast renderer with variant-colored borders |
| `src/lib/components/common/ConfirmDialog.svelte` | Created | Reusable modal confirm with Esc/Enter handlers |
| `src/lib/components/common/ProgressBar.svelte` | Created | Determinate + indeterminate variants |
| `src/lib/components/wizard/WizardSteps.svelte` | Created | Step indicator strip with active/completed states |
| `src/App.svelte` | Modified | Mounted global `<Toaster />` |
| `src/lib/views/AppsView.svelte`, `DataView.svelte`, `SHSHView.svelte`, `SSHRamdiskView.svelte`, `UtilitiesView.svelte` | Modified | `withWorking` helpers now emit success/error toasts in addition to log entries |
| `src/lib/views/SettingsView.svelte` | Modified | Added Updates section: "Check for updates" button + result card with release link |

### Phase 8: Packaging & Distribution (Complete)

| File | Action | Description |
|------|--------|-------------|
| `.github/workflows/release.yml` | Modified | Added `ubuntu-22.04-arm` matrix entry for Linux ARM64 builds; broadened the apt-install step to all `ubuntu-*` runners |
| `docs/USER-GUIDE.md` | Created | End-user installation, view-by-view tour, troubleshooting matrix |
| `docs/MIGRATION-FROM-BASH.md` | Created | Mapping from `restore.sh` menus to UI, shared `saved/` layout, behavioral differences, deferred items |

### Phase 3: Restore & Downgrade (Complete)

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/models/restore.rs` | Created | Typed restore option models + `IpswPrepareRequest`/`IpswPrepareResult` |
| `src-tauri/src/services/restore_options.rs` | Created | Restore option determination logic extracted from `restore.sh::menu_restore` rules |
| `src-tauri/src/services/sha1.rs` | Created | Pure Rust SHA-1 implementation for IPSW verification |
| `src-tauri/src/services/ipsw_prep.rs` | Created | powdersn0w output-path derivation and arg-list builder (with unit tests) |
| `src-tauri/src/commands/restore.rs` | Created | Restore option, aria2c download, SHA-1 verification, `prepare_ipsw` (powdersn0w pipeline), command preview, futurerestore, and idevicerestore commands |
| `src/lib/api/restore.ts` | Created | Typed frontend wrappers including `prepareIpsw` |
| `src/lib/views/RestoreView.svelte` | Modified | Step-by-step restore workflow for option selection, IPSW download/verification, powdersn0w preparation step, command preview, and restore launch. Tethered option auto-routes to futurerestore + pwnDFU |

---

## Remaining Tasks

### Phase 3: Restore & Downgrade ✅
- [x] Restore option determination logic (which restore method based on device + target iOS)
- [x] IPSW download manager with aria2c
- [x] IPSW verification (SHA-1 checksums)
- [x] RestoreView wizard UI (step-by-step flow)
- [x] futurerestore wrapper (Rust tool wrapper)
- [x] idevicerestore wrapper (Rust tool wrapper)
- [x] powdersn0w restore flow (UI step + `prepare_ipsw` Rust command)
- [x] Tethered restore wired to futurerestore + pwnDFU
- [→ Phase 4] DFU IPSW iBSS/iBEC patching (shares machinery with SSH ramdisk + pwnDFU boot)
- [→ Phase 4] Custom IPSW component patching from scratch (xpwn / libipatcher integration)

### Phase 4: Jailbreak & SSH Ramdisk ✅
- [x] gaster/checkm8 exploit wrapper (`commands/jailbreak.rs`, supports `pwn` and `reset`)
- [x] DFU Helper visual guide component (Home + Power vs. Side + Volume Down, guided countdown)
- [x] JailbreakView functional UI (device summary, DFU helper, gaster controls, eligibility warnings)
- [x] IPSW component extraction (`commands/firmware.rs::extract_ipsw_component`, zip-based)
- [x] iBoot32Patcher / iBoot64Patcher wrapper (`commands/firmware.rs::patch_iboot` with boot-args, RSA bypass, debug)
- [x] kloader wrapper (`commands/jailbreak.rs::run_kloader`)
- [x] IMG3 / IMG4 (re)packing wrappers (`commands/firmware.rs::repack_img3`, `pack_img4`)
- [x] Kernel patcher wrapper (`commands/firmware.rs::patch_kernel`, 32/64-bit)
- [x] Ramdisk DMG modification (`commands/firmware.rs::modify_ramdisk`, hfsplus add/remove/resize)
- [x] SSHRamdiskView functional UI (step-driven build pipeline + kloader boot)
- [x] g1lbertJB wrapper (`commands/jailbreak.rs::run_g1lbertjb`, wired into JailbreakView)
- [x] evasi0n wrapper (`commands/jailbreak.rs::run_evasi0n`, wired into JailbreakView)
- [→ future] One-click ramdisk orchestration with BuildManifest.plist auto-discovery (UI currently exposes each step manually; user drives the pipeline)

### Phase 5: SHSH Blob Management ✅
- [x] tsschecker wrapper (`commands/shsh.rs::save_shsh_blob`, generator + APNonce + manifest support)
- [x] Onboard blob dumping conversion (`commands/shsh.rs::dump_onboard_blob`, img4tool wrapper)
- [x] SHSHView functional UI (4-tab layout: tsschecker / Cydia / Onboard / Library)
- [x] Cydia server blob fetching (`commands/shsh.rs::fetch_cydia_blobs`, batched per-build attempts)
- [x] Blob storage and organization (`services/shsh_store.rs` with filename parsing + `list_saved_blobs`)
- [→ Phase 4/SSH ramdisk] On-device raw dump capture from `/dev/rdisk1` (UI exposes the conversion step; raw capture flows through the SSH ramdisk view)

### Phase 6: App & Data Management ✅
- [x] IPA install via ideviceinstaller (`commands/apps.rs::install_ipa`, multi-path)
- [x] App listing + uninstall (`commands/apps.rs::list_installed_apps`, `uninstall_app`; CSV parser with 4 tests)
- [x] AppsView functional UI (install textarea, scope-filtered list, per-row uninstall with confirm)
- [x] Backup via idevicebackup2 (`commands/data.rs::create_backup`, timestamped subdirectories)
- [x] Restore via idevicebackup2 (`commands/data.rs::restore_backup`, --system/--settings/--reboot)
- [x] Backup encryption controls (`commands/data.rs::set_backup_encryption`, on/off/changepw)
- [x] Erase All Content and Settings (`commands/data.rs::erase_device`, exact-phrase confirmation gate)
- [x] List backups with size + mtime (`commands/data.rs::list_backups`, sorted newest-first)
- [x] DataView functional UI (4 tabs: Backup / Restore / Encryption / Erase)
- [→ future] Filesystem mount via sshfs (requires userspace driver + jailbroken device; will live next to SSH Ramdisk view)
- [→ future] App dumping via Clutch / ipainstaller (requires SSH ramdisk session; will live next to SSH Ramdisk view)

### Phase 7: Utilities & Polish ✅
- [x] UtilitiesView functional UI: 5 tabs (Quick actions / irecovery+NVRAM / Syslog / Diagnostics export / TrollStore)
- [x] Backend commands: `enter_recovery`, `exit_recovery`, `run_diagnostics_action` (shutdown/restart/sleep), `pair_device`, `run_activation_action` (activate/deactivate/state), `export_device_info` (deviceInfo/batteryInfo/diagnosticsAll), `run_irecovery_commands`, `clear_nvram`, `start_syslog`/`stop_syslog`/`syslog_status` with Mutex<Option<Child>> for session management
- [x] TrollStore eligibility detection (iOS 14/15 ramdisk vs iOS 16+ TrollRestore vs incompatible vs unknown) and asset preparation (downloads + caches `TrollStore.tar` + `PersistenceHelper_Embedded` from GitHub releases via curl + aria2c, version-stamped cache)
- [x] Global toast layer (`toastStore.svelte.ts` + `Toaster.svelte`) wired into all views' `withWorking` helpers
- [x] Common UI components: `ConfirmDialog`, `Toaster`, `ProgressBar`, `WizardSteps`
- [x] Update checker (`commands/updates.rs::check_for_updates`, loose-semver compare) wired into SettingsView with toast feedback
- [→ future] One-click ramdisk orchestration with BuildManifest auto-discovery
- [→ future] Live SSH session manager (would unblock automated TrollStore install + uicache + datetime sync + iOS 7/8 erase flows)

### Phase 8: Packaging & Distribution ✅
- [x] Tauri bundler config (.dmg, .deb/.AppImage)
- [x] Sidecar binary bundling
- [x] Resource bundling
- [x] CI/CD pipeline (GitHub Actions)
- [x] Linux ARM64 CI/CD target (`ubuntu-22.04-arm` runner added to release.yml matrix)
- [x] User documentation (`docs/USER-GUIDE.md`)
- [x] Migration guide from bash script (`docs/MIGRATION-FROM-BASH.md`)

### Backend Infrastructure (Ongoing)
- [x] Command modules — `device`, `restore`, `jailbreak`, `firmware`, `shsh`, `apps`, `data`, `utilities`, `trollstore`, `updates`
- [x] API wrapper layer (`src/lib/api/`) — typed Tauri invoke wrappers for every backend command
- [→ future] `state.rs` — global app state (currently each command is stateless except `SYSLOG_CHILD` static Mutex)
- [→ future] Cohesive `tools/` runner abstraction (currently each command has its own `run_process_streaming` helper)
- [→ future] Service modules for download manager + firmware keys cache

---

## Build Status
- **Frontend (Vite/Svelte):** ✅ Compiles cleanly (`npm run check`, `npm run build` pass — only pre-existing template errors remain)
- **Backend (Rust/Tauri):** ✅ Compiles cleanly (`cargo check`, `cargo test` pass — 37 unit tests across services + commands)
