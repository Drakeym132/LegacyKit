# LegacyKit

<img width="1326" height="1065" alt="Screenshot 2026-05-04 at 3 30 15 PM" src="https://github.com/user-attachments/assets/5a33d42e-5a85-486c-9e1f-39aee3000ddb" />

> Cross-platform iOS device restore, downgrade, and jailbreak toolkit.

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%7C%20Linux-lightgrey.svg)](#install--download)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://v2.tauri.app/)
[![Built with Svelte](https://img.shields.io/badge/built%20with-Svelte%205-FF3E00.svg)](https://svelte.dev/)

LegacyKit is a desktop application for restoring, downgrading, jailbreaking,
and managing legacy iOS devices. It provides a graphical front-end over the
established command-line ecosystem (`futurerestore`, `idevicerestore`, `gaster`,
`ipwnder`, `tsschecker`, `ipsw`, and friends) and surfaces their workflows as
guided, log-streamed flows in a single window.

Supported on **macOS** (x64 + arm64) and **Linux** (x64 + arm64). Windows is not
currently a target.

## Features

- **Restore / downgrade** flows for supported devices (signed OTA, blob-based,
  powdersn0w-style, "no-blob" 32-bit)
- **SHSH blob management** — save via tsschecker, onboard dumps, Cydia server
  batches, and a library view
- **Jailbreak pipeline** including a DFU helper, gaster pwn/reset, and an
  untether step
- **SSH Ramdisk** workflow — build, boot via `kloader`, and run scripted post-boot
  actions
- **Apps & Data** — IPA installation, app dump tooling, idevicebackup2 backups
- **Utilities** — diagnostics export, NVRAM tools, syslog viewer, recovery /
  shutdown / pair quick-actions, TrollStore install
- **Updates** view for tracking bundled sidecar versions
- **Integrated terminal log deck** — every sidecar process streams into a
  persistent, searchable log
- **Workspace-based state** — all artifacts live under a user-chosen `saved/`
  directory, compatible with the legacy bash layout

## Install / Download

Prebuilt bundles are published on the
[GitHub Releases](https://github.com/Drakeym132/LegacyKit/releases) page.

| Platform | Bundles |
|---|---|
| macOS (x64, arm64) | `.dmg`, `.app` |
| Linux (x64, arm64) | `.deb`, `.rpm`, `.AppImage` |

> **Linux note:** USB device access (DFU/recovery/normal) requires `udev` rules
> for libimobiledevice and the bundled checkm8 tooling. See the install section
> of [`docs/USER-GUIDE.md`](docs/USER-GUIDE.md:1) for the exact rules and group
> membership requirements.

## Quickstart

1. Install the appropriate bundle from Releases and launch **LegacyKit**.
2. On first run, choose a **workspace directory** — this is where SHSH blobs,
   IPSWs, and backups will be stored.
3. Connect your iOS device over USB.
4. The device is auto-detected; the toolbar shows model, ECID, mode (Normal /
   Recovery / DFU), and iOS version.
5. Pick a flow from the sidebar (**Restore**, **Jailbreak**, **SHSH Blobs**,
   **Apps**, **Data**, **Utilities**, **SSH Ramdisk**, **Updates**).
6. Watch live logs in the terminal deck at the bottom of the window. All output
   is persisted into the workspace under `logs/`.

## Workspace layout

LegacyKit reuses the bash script's `saved/` conventions, so a single workspace
can be shared between both tools:

```
saved/
├── shsh/                       # SHSH blobs (tsschecker, Cydia, onboard dumps)
├── ipsw/                       # downloaded IPSWs
├── info/                       # device info / battery info exports
├── backups/                    # idevicebackup2 outputs (timestamped)
├── TrollStore.tar              # cached TrollStore release asset
├── PersistenceHelper_Embedded
└── TrollStore_version          # version stamp for cache invalidation
```

## Development

### Prerequisites

- **Node.js** 20+
- **pnpm** (this repo uses pnpm; do not commit `package-lock.json`)
- **Rust** 1.77+ (stable toolchain, `cargo`, `rustfmt`, `clippy`)
- Platform-specific Tauri prerequisites — follow the official guide:
  https://v2.tauri.app/start/prerequisites/

### Commands

```bash
pnpm install           # install JS dependencies
pnpm tauri dev         # run the app in development mode
pnpm tauri build       # produce a release bundle for the current platform
```

### Quality gates

Run before opening a PR:

```bash
pnpm svelte-check
pnpm tsc --noEmit -p tsconfig.app.json
cd src-tauri && cargo fmt && cargo clippy --all-targets
```


## Project structure

```
LegacyKit/
├── src/                        # Svelte 5 frontend
│   ├── App.svelte              # entry; mounts layout + views
│   ├── app.css
│   └── lib/
│       ├── api/                # typed wrappers around Tauri `invoke`
│       ├── components/
│       │   ├── actions/        # ActionPanel
│       │   ├── common/         # ConfirmDialog, ProgressBar, TerminalLog, Toaster
│       │   ├── device/         # DeviceCard, DeviceImage, DfuHelper, …
│       │   ├── layout/         # Sidebar, Toolbar, ContentArea, TerminalDeck
│       │   ├── onboarding/     # WorkspaceOnboarding
│       │   ├── restore/        # IpswDownloaderPanel
│       │   └── wizard/         # WizardSteps
│       ├── stores/             # Svelte 5 runes-based stores (.svelte.ts)
│       ├── utils/              # deviceModels, processorGen, workingState
│       └── views/              # Home, Restore, Jailbreak, Apps, Data,
│                               # SSHRamdisk, Utilities, Updates, …
├── src-tauri/                  # Rust / Tauri backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers (one file per domain)
│   │   ├── services/           # business logic (workspace, ipsw_prep,
│   │   │                       # bootchain, external_tools, device_parser, …)
│   │   ├── models/             # serde DTOs
│   │   ├── tools/              # runner.rs (sidecar process runner) + util
│   │   ├── platform.rs         # macOS/Linux + arch detection
│   │   ├── error.rs            # AppError type
│   │   └── lib.rs              # command registration
│   ├── binaries/               # bundled CLI sidecars (gitignored)
│   ├── icons/                  # bundle icons (gitignored)
│   └── Cargo.toml
├── resources/                  # runtime assets (payload, ssh_config, appdump,
│                               # sshrd patches/blobs)
├── scripts/                    # download-device-images.sh, extract-device-icons.sh
├── docs/
│   ├── USER-GUIDE.md
│   └── MIGRATION-FROM-BASH.md
├── public/                     # static frontend assets
├── .github/workflows/          # ci.yml (lint/check), release.yml (bundle build)
├── README.md
├── AGENTS.md
├── CONTRIBUTING.md
├── SECURITY.md
└── LICENSE
```

## CI

- [`ci.yml`](.github/workflows/ci.yml:1) runs `svelte-check`, `tsc`, `cargo fmt`,
  and `cargo clippy` on Ubuntu and macOS for every PR.
- `release.yml` builds the macOS and Linux bundles on tagged releases.


## Security

To report a vulnerability, please use GitHub's private advisory feature — see
[`SECURITY.md`](SECURITY.md:1) for the full policy and scope.

## For AI Agents

LegacyKit is set up for AI-assisted development. If you're an agent (Claude,
Copilot, Roo, etc.) entering this repo, start with
[`AGENTS.md`](AGENTS.md:1) for an annotated repo map, code conventions, and
"where to start" pointers. Roo Code users will also find
[`.roomodes`](.roomodes:1) at the repo root, which defines the orchestrator,
code, architect, debug, UI-developer, and security-reviewer modes used in this
project.

## License

LegacyKit is licensed under **GPL-3.0-or-later**. See [`LICENSE`](LICENSE:1) for
the full text.
