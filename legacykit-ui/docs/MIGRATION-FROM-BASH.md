# Migrating from `restore.sh` to LegacyKit (UI)

If you've been using the original Legacy iOS Kit bash script (`restore.sh`), this document maps
the most common bash menus to their LegacyKit UI equivalents and explains the shared on-disk
state.

## Shared on-disk layout

LegacyKit reuses the bash script's `saved/` directory conventions:

```
saved/
├── shsh/                    # SHSH blobs (tsschecker, Cydia, onboard dumps)
├── ipsw/                    # downloaded IPSWs
├── info/                    # device info / battery info exports
├── backups/                 # idevicebackup2 outputs (timestamped subdirs)
├── TrollStore.tar           # cached TrollStore release asset
├── PersistenceHelper_Embedded
└── TrollStore_version       # version stamp for cache invalidation
```

You can point both tools at the same `saved/` directory — the UI does not move or rename anything
the bash script writes.

## Menu mapping

### Main menu

| `restore.sh` | LegacyKit view |
|---|---|
| 1) Restore/Downgrade | **Restore** |
| 2) Save SHSH Blobs | **SHSH Blobs** |
| 3) Jailbreak Device | **Jailbreak** |
| 4) Other Utilities | **Utilities** |
| 5) Useful Utilities | **Utilities** |
| 6) Install IPA | **Apps** |
| 7) Install TrollStore | **Utilities → TrollStore** |
| 8) (Re-)Install Dependencies | _Not yet ported — handled at install time on each platform_ |

### Misc / Useful Utilities

| `menu_miscutilities` / `menu_usefulutilities` | LegacyKit |
|---|---|
| Export Device Info | **Utilities → Diagnostics export → ideviceinfo** |
| Export Battery Info | **Utilities → Diagnostics export → Battery (AppleSmartBattery)** |
| Shutdown / Restart Device | **Utilities → Quick actions → Shutdown / Restart** |
| Enter Recovery Mode | **Utilities → Quick actions → Enter Recovery** |
| Attempt Activation | **Utilities → Quick actions → Attempt activation** |
| Hacktivate Device | **Restore** (with Hacktivate flag) — UI Hacktivation as a standalone action is on the deferred list |
| Revert Hacktivation | _Deferred — run via SSH from a ramdisk session for now_ |
| Enter pwnDFU Mode | **Jailbreak → gaster pwn** |
| Send Pwned iBSS | **SSH Ramdisk** (build pipeline → `kloader` boot) |
| Clear NVRAM | **Utilities → irecovery / NVRAM → Clear NVRAM** |
| DFU Mode Helper | **Jailbreak → DFU Helper** (generation-aware countdown) |
| Run uicache | _Run via SSH from a ramdisk session_ |
| Console (`idevicesyslog`) | **Utilities → Syslog** (start/stop with rolling viewer) |
| Update DateTime | _Deferred — run via SSH ramdisk session_ |
| Just Boot | **SSH Ramdisk** (build the bundle, boot via `kloader`) |
| FourThree Utility | _Deferred_ |
| Pair Device | **Utilities → Quick actions → Pair (Trust)** |

### Jailbreak / SSH Ramdisk

| Bash function | LegacyKit |
|---|---|
| `device_ramdisk` / `device_ramdisk64` | **SSH Ramdisk** (manual step pipeline) |
| `device_dfuhelper` | **Jailbreak → DFU Helper** |
| `gaster` checkm8 / pwnDFU | **Jailbreak → gaster pwn / reset** |
| `g1lbertJB`, `evasi0n` | **Jailbreak → Untether** |
| `kloader` | Used inside SSH Ramdisk view |

### SHSH

| Bash menu | LegacyKit |
|---|---|
| Save Blobs | **SHSH Blobs → tsschecker** |
| Onboard Blobs | **SHSH Blobs → Onboard dump** (raw `.shsh` conversion via `img4tool`) |
| Cydia Server Blobs | **SHSH Blobs → Cydia servers** (batched per-build) |
| List saved blobs | **SHSH Blobs → Library** |

### Apps / Data

| Bash menu | LegacyKit |
|---|---|
| Install IPA | **Apps → Install IPA(s)** (multi-line paths) |
| List apps | **Apps → Installed apps** (User / System / All scope) |
| Uninstall app | Per-row in the Installed apps list |
| `device_backup_create` | **Data → Backup** |
| `device_backup_restore` | **Data → Restore** |
| Backup encryption (`-i on/off/changepw`) | **Data → Encryption** |
| Erase all content and settings | **Data → Erase** (typed confirmation gate) |

### TrollStore

| Bash function | LegacyKit |
|---|---|
| Download latest TrollStore release | **Utilities → TrollStore → Asset preparation** |
| iOS 14/15 install via SSH ramdisk | UI guides through the steps; actual install is still manual SSH (live SSH session orchestration is on the roadmap) |
| iOS 16+ install via TrollRestore | UI surfaces the path; install runs via the same Python venv the bash script creates (`saved/TrollRestore_venv/bin/python3 saved/TrollRestore/trollstore.py`) |

## Behavioral differences

- **No interactive `select_yesno` / `pause` prompts.** The UI replaces them with explicit confirm
  dialogs and toast notifications. Long-running actions stream their output to the terminal
  panel.
- **State is held in Svelte stores, not env vars.** Settings persist across restarts via
  `localStorage`. Tool paths are resolved via the bundled sidecar layout instead of `set_tool_paths`.
- **No `(Re-)Install Dependencies`.** Tool binaries ship with the bundle (per-platform). You no
  longer need to install GTK/WebKit dev libs at runtime.
- **`debug_mode` flag.** Equivalent: open the Terminal panel — every backend command logs there.

## What's _not_ in the UI yet

These features are deferred to follow-up phases:

- One-click ramdisk orchestration (BuildManifest auto-discovery; the UI exposes each step manually).
- Live SSH session manager (currently each SSH-based action is a manual `ssh -p 2222 root@…` from
  your terminal).
- Filesystem mount via `sshfs`.
- App dumping via Clutch / ipainstaller (requires the SSH session manager above).
- iOS 7/8 erase flow that the bash script invokes from inside the SSH ramdisk session.

For these, run `restore.sh` directly. The two tools coexist in the same `saved/` directory.
