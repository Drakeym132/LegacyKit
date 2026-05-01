# LegacyKit User Guide

LegacyKit is a desktop app for managing legacy iOS devices. It is a re-implementation of the
[Legacy iOS Kit](https://github.com/LukeZGD/Legacy-iOS-Kit) bash script as a Tauri v2 + Svelte 5
application, targeting macOS and Linux (x86_64 and arm64).

This guide covers the UI workflows. For protocol-level detail, refer to the upstream
[Legacy iOS Kit wiki](https://github.com/LukeZGD/Legacy-iOS-Kit/wiki).

---

## Installing

Download the matching bundle from the [Releases](https://github.com/Drakeym132/LegacyKit/releases) page:

| Host | Bundle |
|------|--------|
| macOS Apple Silicon (M1/M2/M3) | `LegacyKit_<ver>_aarch64.dmg` |
| macOS Intel | `LegacyKit_<ver>_x64.dmg` |
| Linux x86_64 | `legacy-kit_<ver>_amd64.AppImage` (or `.deb`) |
| Linux arm64 | `legacy-kit_<ver>_arm64.AppImage` (or `.deb`) |

On Linux you also need the upstream `udev` rules for USB device access — install via the bash
script's `install_udev_rules`, or copy `resources/udev/*.rules` into `/etc/udev/rules.d/`.

---

## Connecting a device

1. Plug in the device with a USB cable.
2. On first connect, tap **Trust** on the device.
3. The header device card auto-populates with model, iOS version, ECID, UDID, and current mode
   (`Normal`, `Recovery`, `DFU`, `kDFU`, or `pwnDFU`).

If nothing shows up:

- Open **Settings** and confirm "Auto-detect device" is on (poll interval defaults to 2000 ms).
- On Linux, verify USB permissions (`lsusb` should list the Apple device, and your user must be
  in the `plugdev` group).
- macOS may require a "Trust this computer" prompt to be confirmed.

---

## View-by-view tour

### Home
A welcome screen with a device summary card and quick-jump links to the most common workflows.

### Restore
Step-by-step downgrade / restore flow:

1. Pick a restore option (the UI surfaces only the methods compatible with your device + target
   iOS — IPSW restore, futurerestore, powdersn0w, etc.).
2. Download the IPSW (via `aria2c`, with resumable, multi-connection support).
3. Verify SHA-1.
4. Optionally run the powdersn0w preparation pipeline.
5. Preview the exact restore command the app will run.
6. Launch the restore (futurerestore + pwnDFU for tethered, idevicerestore otherwise).

### Jailbreak
- DFU helper with generation-aware button instructions and a guided countdown.
- `gaster` pwn / reset for checkm8-vulnerable devices.
- `g1lbertJB` and `evasi0n` untether wrappers (eligibility checked against device + iOS).

### SSH Ramdisk
Step-driven build pipeline:
extract IPSW components → patch iBoot/kernel → grow ramdisk → inject SSH binaries → repack as
IMG3/IMG4 → boot via `kloader`.

The view exposes each step manually so you can debug or substitute pieces. Once the build
finishes, you boot the ramdisk and SSH in from your host (`iproxy 2222 22` then
`ssh -p 2222 root@127.0.0.1`, default password `alpine`).

### SHSH Blobs
Four tabs:
- **tsschecker** — save blobs for a specific iOS version + build (with optional generator,
  APNonce, board, BuildManifest).
- **Cydia servers** — batch fetch blobs from `cydia.saurik.com` for one or more build IDs.
- **Onboard dump** — convert a previously dumped raw blob (from an SSH ramdisk session) into a
  `.shsh2` file using `img4tool --convert`.
- **Library** — browse saved blobs, parsed into device + iOS + build columns.

### Apps
- Multi-IPA install via `ideviceinstaller` (one path per line).
- Scope-filtered (User / System / All) install list with per-row uninstall.
- App dump (Clutch / ipainstaller) requires an SSH ramdisk session — flagged as a future
  follow-up, run via SSH for now.

### Data
Four tabs:
- **Backup** — `idevicebackup2` to a timestamped subdirectory under your chosen root.
- **Restore** — radio-pick a backup and toggle `--system`, `--settings`, `--reboot`.
- **Encryption** — turn backup encryption on/off or change the password.
- **Erase** — typed-confirmation gated full Erase All Content and Settings.

### Utilities
Five tabs:
- **Quick actions** — Enter/Exit Recovery, Shutdown, Restart, Sleep, Pair, Validate pairing,
  Unpair, Activate, Deactivate, show activation state.
- **irecovery / NVRAM** — run a list of `irecovery -c` commands (optionally followed by
  `irecovery -n` for reboot), or use the dedicated **Clear NVRAM** action.
- **Syslog** — start/stop a streaming `idevicesyslog -q` session (rolling 500 lines in view, full
  log captured in the global terminal).
- **Diagnostics export** — write `ideviceinfo`, AppleSmartBattery, or Diagnostics All output to a
  timestamped `.txt` in the directory of your choice.
- **TrollStore** — eligibility check (iOS 14/15 → SSH ramdisk path; iOS 16+ → TrollRestore;
  pre-14 → incompatible) and asset preparation (downloads + caches the latest TrollStore.tar
  and PersistenceHelper_Embedded from `opa334/TrollStore`).

### Settings
- Theme: System / Light / Dark.
- Terminal panel visibility + height.
- Auto-detect device + poll interval.
- **Check for updates** — compares the bundled version to the latest GitHub release.

---

## Tips

- The terminal panel at the bottom captures every backend command's stdout/stderr. Toggle it from
  **Settings → Terminal**.
- Toasts give you the success/error of each background action; the terminal has the full log.
- All long-running actions are streamed — you can watch progress as it happens rather than waiting
  for completion.
- LegacyKit never overwrites the bash script's `saved/` directory layout. Reusing the same
  directory across both tools is safe.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| "Device not found" with device plugged in | USB perms (Linux) or pairing (macOS) | Install udev rules; tap Trust on device |
| `irecovery` fails to find device | Device is in Normal mode | Use **Utilities → Enter Recovery** first |
| TrollStore install never finishes (iOS 14/15) | SSH ramdisk session not running | Boot the SSH ramdisk first via the SSH Ramdisk view |
| Update check returns a parse error | GitHub API rate-limited | Wait or set up `GITHUB_TOKEN` (anonymous limit is 60 req/h) |
| Frontend renders but no devices ever detected | Polling disabled | Settings → Device Detection → toggle Auto-Detect on |

For deeper protocol-level issues, the upstream
[Legacy iOS Kit Troubleshooting wiki](https://github.com/LukeZGD/Legacy-iOS-Kit/wiki/Troubleshooting)
is still the source of truth.
