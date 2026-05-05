# Sidecar Binaries

## Overview

The `src-tauri/binaries/` directory contains vendored CLI sidecar tools that LegacyKit shells out to at runtime. It is **gitignored** — large binary files and heterogeneous licenses are kept out of the repository to avoid bloat. The directory is populated at release time from binaries inherited from the upstream Legacy iOS Kit project, and contributors running `pnpm tauri dev` must populate it locally before sidecar-using flows will work.

Runtime resolution is handled by [`src-tauri/src/platform.rs`](../src-tauri/src/platform.rs:7), which maps the current OS and architecture to a candidate path under `binaries/`. The resolver is **name-based** — there is no static manifest. Code calls `resolve_binary_path(app, "toolname")` and the platform module probes the expected locations.

## Expected layout

The resolver first tries an arch-specific subdirectory, then falls back to a flat layout:

```
src-tauri/binaries/
├── macos/
│   ├── arm64/           ← preferred for aarch64 (Apple Silicon)
│   │   ├── futurerestore
│   │   ├── gaster
│   │   ├── ipwnder
│   │   └── …
│   ├── x86_64/          ← preferred for x86_64 (Intel)
│   │   └── …
│   └── <flat>           ← fallback if no arch subdir exists
│       └── …
└── linux/
    ├── arm64/
    │   └── …
    ├── x86_64/
    │   └── …
    └── <flat>
        └── …
```

For example, on macOS arm64, `resolve_binary_path(app, "gaster")` tries in order:
1. `binaries/macos/arm64/gaster`
2. `binaries/macos/gaster`

On Linux x86_64, `resolve_binary_path(app, "irecovery")` tries:
1. `binaries/linux/x86_64/irecovery`
2. `binaries/linux/irecovery`

## Tool inventory

The following binaries are expected by the codebase. Obtain them from the project's GitHub Releases (once published) or build from upstream sources.

> **Note:** `kuroutadori`/`litera1n` is fetched on-demand at runtime from [`src-tauri/src/services/external_tools.rs`](../src-tauri/src/services/external_tools.rs:1) and is **not** bundled in `binaries/`.

| Tool | Purpose | Used by | Source |
|------|---------|---------|--------|
| `tsschecker` | Request SHSH blobs from Apple TSS | [`commands/shsh.rs`](../src-tauri/src/commands/shsh.rs:97), [`commands/restore.rs`](../src-tauri/src/commands/restore.rs:135) | Upstream community tool (tsschecker) |
| `img4tool` | Create/verify IMG4 containers | [`commands/shsh.rs`](../src-tauri/src/commands/shsh.rs:267), [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:295), [`services/bootchain.rs`](../src-tauri/src/services/bootchain.rs:143) | Upstream community tool (img4tool) |
| `xpwntool` | Encrypt/decrypt IMG3 payloads | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:406), [`services/bootchain.rs`](../src-tauri/src/services/bootchain.rs:159) | Upstream community tool (xpwntool) |
| `hfsplus` | Manipulate HFS+ disk images | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:496) | Upstream community tool (hfsplus) |
| `iBoot32Patcher` | Patch 32-bit iBoot for boot args | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:181), [`services/bootchain.rs`](../src-tauri/src/services/bootchain.rs:126) | Upstream community tool |
| `iBoot64Patcher` | Patch 64-bit iBoot for boot args | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:181) | Upstream community tool |
| `Kernel32Patcher` | Patch 32-bit kernels | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:614) | Upstream community tool |
| `Kernel64Patcher` | Patch 64-bit kernels | [`commands/firmware.rs`](../src-tauri/src/commands/firmware.rs:615) | Upstream community tool |
| `aria2c` | High-performance HTTP/FTP downloader | [`commands/restore.rs`](../src-tauri/src/commands/restore.rs:261), [`commands/trollstore.rs`](../src-tauri/src/commands/trollstore.rs:70) | aria2 project |
| `powdersn0w` | Build custom IPSWs for A4-era devices | [`commands/restore.rs`](../src-tauri/src/commands/restore.rs:418) | Upstream community tool |
| `idevicerestore` | Restore IPSW to device | [`commands/restore.rs`](../src-tauri/src/commands/restore.rs:458) | libimobiledevice project |
| `futurerestore_new` | Restore with custom SHSH blobs | [`commands/restore.rs`](../src-tauri/src/commands/restore.rs:501) | futurerestore project |
| `gaster` | checkm8 exploit + pwnDFU operations | [`commands/jailbreak.rs`](../src-tauri/src/commands/jailbreak.rs:47) | Upstream community tool |
| `ipwnder` | checkm8 pwnDFU for A6-A7 (macOS) | [`commands/jailbreak.rs`](../src-tauri/src/commands/jailbreak.rs:386) | Upstream community tool (macOS only) |
| `irecovery` | Interact with Recovery/DFU mode | [`commands/jailbreak.rs`](../src-tauri/src/commands/jailbreak.rs:250), [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:40), [`commands/device.rs`](../src-tauri/src/commands/device.rs:20) | libimobiledevice project |
| `ideviceinfo` | Query device properties | [`commands/device.rs`](../src-tauri/src/commands/device.rs:10), [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:170) | libimobiledevice project |
| `ideviceinstaller` | Install/uninstall IPA apps | [`commands/apps.rs`](../src-tauri/src/commands/apps.rs:16) | libimobiledevice project |
| `idevicebackup2` | Create/restore device backups | [`commands/data.rs`](../src-tauri/src/commands/data.rs:42) | libimobiledevice project |
| `ideviceenterrecovery` | Enter Recovery mode | [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:28) | libimobiledevice project |
| `idevicediagnostics` | Run device diagnostics | [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:53) | libimobiledevice project |
| `idevicepair` | Manage device pairings | [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:84) | libimobiledevice project |
| `ideviceactivation` | Activate device | [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:112) | libimobiledevice project |
| `idevicesyslog` | Stream device syslog | [`commands/utilities.rs`](../src-tauri/src/commands/utilities.rs:316) | libimobiledevice project |

## Device-side binaries (NOT host tools)

The following binaries are **ARM executables that run on the iOS device itself**, not on the host computer. They are stored in `resources/kloader/` and are sent to the device via SSH when needed:

| Binary | Purpose | When used |
|--------|---------|-----------|
| `kloader` | Load kernels over USB on iOS 6+ | kDFU mode (entering DFU from jailbroken device via SSH) |
| `kloader5` | Load kernels on iOS 5 | kDFU mode for iPad 3 on iOS 5 |
| `kloader_axi0mX` | Alternative kloader for older iOS | kDFU mode for iOS ≤5 devices |

**IMPORTANT**: These are NOT host-side sidecar binaries. Do not place them in `src-tauri/binaries/`. They are bundled as app resources in `resources/kloader/` and are sent to the device via SSH when entering kDFU mode from a jailbroken state.

### kDFU vs pwnDFU boot flows

- **pwnDFU tethered boot**: Uses `irecovery -f` to send patched iBSS/iBEC directly. This is the correct flow for A6+ devices in pwnDFU mode.
- **kDFU mode**: Requires the device to be jailbroken with SSH access. The kloader binary is sent to the device and executed there to enter a software-based DFU mode.

## Per-platform notes

### macOS

- Binaries can be unsigned or ad-hoc signed for local development.
- The release bundle workflow handles proper code signing and notarization.
- Apple Silicon (arm64) and Intel (x86_64) are both supported; use the appropriate arch subdirectory.

### Linux

- Ensure binaries are marked executable: `chmod +x src-tauri/binaries/**/*`
- Some tools (libimobiledevice suite) require runtime libraries from the system:
  - `libusb-1.0-0`
  - `libimobiledevice`, `libplist`, `libusbmuxd`
- Install via your distro's package manager (e.g., `apt install libimobiledevice-dev libusb-1.0-0-dev`).

### File permissions

After extracting or copying binaries, always run:

```bash
chmod +x src-tauri/binaries/**/*
```

## Verifying the setup

1. Run `pnpm tauri dev` to start the app in development mode.
2. Open the **Utilities** view from the sidebar.
3. Watch the terminal log deck at the bottom of the window.
4. If a sidecar is missing, you'll see an error like:
   ```
   Binary 'gaster' not found for macos aarch64 (tried: .../binaries/macos/arm64/gaster, .../binaries/macos/gaster)
   ```
5. The error originates from [`src-tauri/src/tools/runner.rs`](../src-tauri/src/tools/runner.rs:1) when the spawn fails.

## Adding a new sidecar tool

1. Drop the binary into the appropriate path:
   - `src-tauri/binaries/macos/arm64/<tool>` (and/or `x86_64/`)
   - `src-tauri/binaries/linux/arm64/<tool>` (and/or `x86_64/`)
2. Call it from a service or command using the runner:
   ```rust
   let binary = resolve_binary_path(&app, "toolname").map_err(AppError::CommandFailed)?;
   crate::tools::runner::run_streaming(&app, binary, &args)?;
   ```
3. No changes to [`tauri.conf.json`](../src-tauri/tauri.conf.json:42) are needed — the bundle already includes `binaries/**/*` as a resource glob.

## Why isn't this automated?

A `scripts/fetch-binaries.sh` puller is planned as a future improvement. For now, contributors should:

- Download the sidecar archive from the project's GitHub Releases page (once published), or
- Build tools from their upstream sources.

The binaries inherited from the upstream Legacy iOS Kit fork are bundled unchanged in releases, so contributors can reuse them once a release is published.
