# AGENTS.md

Orientation for AI coding agents (Claude, Copilot, Roo Code, Cursor, etc.)
working in this repository. Humans contributing for the first time should read
[`CONTRIBUTING.md`](CONTRIBUTING.md:1) instead — but this file is a useful
high-density overview either way.

## Purpose

LegacyKit is a Tauri 2 + Svelte 5 desktop application that provides a graphical
front-end for restoring, downgrading, jailbreaking, and managing legacy iOS
devices. It orchestrates a curated set of bundled CLI sidecar tools
(futurerestore, idevicerestore, gaster, ipwnder, tsschecker, ipsw, kloader,
irecovery, img4tool, …) and surfaces their workflows as guided, log-streamed
flows. Targets: macOS (x64/arm64) and Linux (x64/arm64).

## Repo map

```
LegacyKit/
├── src/                        # Svelte 5 frontend (TypeScript, runes)
│   ├── App.svelte              # root component; layout + view switcher
│   ├── app.css                 # global styles
│   └── lib/
│       ├── api/                # typed wrappers over Tauri `invoke` — one file per backend domain
│       ├── components/         # reusable UI: layout/, device/, common/, actions/,
│       │                       # onboarding/, restore/, wizard/
│       ├── stores/             # Svelte 5 runes stores (.svelte.ts) — single source of truth
│       │                       # for device, navigation, logs, actions, settings, toasts
│       ├── utils/              # deviceModels, processorGen, workingState
│       └── views/              # one component per sidebar entry
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/           # #[tauri::command] handlers, one file per domain
│   │   │                       # (apps, data, device, firmware, jailbreak, just_boot,
│   │   │                       #  restore, settings, shsh, trollstore, updates, utilities)
│   │   ├── services/           # domain logic — workspace, ipsw_prep, bootchain,
│   │   │                       # external_tools, device_parser, device_meta,
│   │   │                       # app_settings, just_boot_store, shsh_store,
│   │   │                       # log_persist, restore_options, sha1
│   │   ├── models/             # serde DTOs mirroring command surface
│   │   ├── tools/              # runner.rs (sidecar runner + log streaming), util.rs
│   │   ├── platform.rs         # OS + arch detection; resolves sidecar paths
│   │   ├── error.rs            # AppError; commands return Result<T, AppError>
│   │   └── lib.rs              # builder + command registration
│   ├── binaries/               # bundled sidecars (GITIGNORED — never commit)
│   ├── icons/                  # bundle icons (gitignored, large)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── resources/                  # runtime assets shipped with the app:
│   ├── payload                 # ramdisk payload
│   ├── ssh_config              # ssh config used inside ramdisk sessions
│   ├── appdump/                # clutch / ipainstaller helpers
│   └── sshrd/                  # per-device SSH-ramdisk patches and blobs
├── scripts/                    # repo maintenance scripts (download/extract device images)
├── docs/                       # USER-GUIDE.md, MIGRATION-FROM-BASH.md
├── public/                     # frontend static assets
├── .github/workflows/          # ci.yml, release.yml
├── .roomodes                   # Roo Code mode definitions
├── README.md / AGENTS.md / CONTRIBUTING.md / SECURITY.md
└── LICENSE                     # GPL-3.0-or-later
```

## Tech stack + versions (pinned)

| Component | Version |
|---|---|
| Svelte | 5.55.4 |
| Vite | 8.0.10 |
| TypeScript | ~6.0.2 |
| `@tauri-apps/cli` | 2.10.1 |
| Tauri (Rust crate) | 2.10.3 |
| Rust edition | 2021 |
| Node | 20+ |
| Package manager | pnpm |

Bundle targets: `all` (produces `.dmg`/`.app` on macOS; `.deb`/`.rpm`/`.AppImage`
on Linux). Project version: **1.0.0**. License: **GPL-3.0-or-later**.

## Code conventions

### Frontend (Svelte 5 + TypeScript)

- **Runes only.** Use `$state`, `$derived`, `$effect`, `$props`. Do not use the
  legacy `writable` / `readable` / `$:` syntax.
- **Stores live in [`src/lib/stores/`](src/lib/stores/deviceStore.svelte.ts:1)**
  with the `.svelte.ts` extension and export rune-backed objects (typically a
  frozen object with getters or a class instance). Do not introduce
  `svelte/store`-based stores.
- **API layer is mandatory.** All Tauri calls go through
  [`src/lib/api/*.ts`](src/lib/api/settings.ts:1). These are thin, typed
  wrappers over `@tauri-apps/api/core` `invoke`. Components and stores must
  **never** call `invoke` directly.
- One view per sidebar entry, in [`src/lib/views/`](src/lib/views/HomeView.svelte:1).
- The terminal log deck is fed by [`logStore`](src/lib/stores/logStore.svelte.ts:1);
  do not write a competing log mechanism.

### Backend (Rust + Tauri 2)

- **Commands** are `#[tauri::command]` handlers in
  [`src-tauri/src/commands/<domain>.rs`](src-tauri/src/commands/settings.rs:1),
  registered in `commands/mod.rs` and `lib.rs`.
- Commands stay thin. **Long-running work delegates to
  [`services/`](src-tauri/src/services/mod.rs:1).**
- **Sidecar processes go through [`tools/runner.rs`](src-tauri/src/tools/runner.rs:1).**
  The runner handles spawning, log streaming into the terminal deck, and
  cancellation. Do not call `Command::new` directly from a command handler.
- **Errors:** commands return `Result<T, AppError>` from
  [`src-tauri/src/error.rs`](src-tauri/src/error.rs:1). Convert foreign errors
  via `From` impls; do not stringify errors at the boundary unless absolutely
  necessary.
- **DTOs** live in [`src-tauri/src/models/`](src-tauri/src/models/mod.rs:1) and
  derive `Serialize` / `Deserialize`. Their TypeScript shapes must stay in sync
  with the `api/` wrappers.

### Cross-platform

- macOS-only code: gate with `#[cfg(target_os = "macos")]`.
- Linux-only code: gate with `#[cfg(target_os = "linux")]`.
- Architecture-specific sidecar resolution lives in
  [`src-tauri/src/platform.rs`](src-tauri/src/platform.rs:1). Add new
  arch/OS combinations there, not ad-hoc.

## Key invariants

1. **`src-tauri/binaries/` is gitignored.** Never commit binaries. Sidecars are
   resolved at runtime via `platform.rs`. Full sidecar layout + tool inventory:
   [`docs/SIDECAR-BINARIES.md`](docs/SIDECAR-BINARIES.md:1).
2. **No hardcoded paths.** Do not bake `/usr/local/bin`, `/opt/homebrew`, or
   user-home paths into Rust commands. All sidecar paths flow from
   `platform.rs`.
3. **All child processes go through [`tools/runner.rs`](src-tauri/src/tools/runner.rs:1).**
   This is what makes logs visible in the terminal deck and what enables
   cancellation. Bypassing it produces silent failures from the user's POV.
4. **Workspace state is user-owned.** The `saved/` directory layout is shared
   with the legacy bash script — see
   [`docs/MIGRATION-FROM-BASH.md`](docs/MIGRATION-FROM-BASH.md:1). Do not
   rename or relocate existing entries.
5. **API wrapper layer is mandatory** on the frontend (see above).

## Build & verify

```bash
pnpm install
pnpm tauri dev                       # interactive run

# Quality gates (must be clean before PR):
pnpm svelte-check
pnpm tsc --noEmit -p tsconfig.app.json
cd src-tauri && cargo fmt && cargo clippy --all-targets

# Release bundle for the current platform:
pnpm tauri build
```

## Where to start

- **Add a new view (sidebar entry):** copy the pattern in
  [`src/lib/views/HomeView.svelte`](src/lib/views/HomeView.svelte:1), then
  register the route in
  [`src/lib/stores/navigationStore.svelte.ts`](src/lib/stores/navigationStore.svelte.ts:1)
  and surface it in [`src/lib/components/layout/Sidebar.svelte`](src/lib/components/layout/Sidebar.svelte:1).
- **Add a new Tauri command:** mirror
  [`src-tauri/src/commands/settings.rs`](src-tauri/src/commands/settings.rs:1).
  Register it in `commands/mod.rs` and inside `tauri::generate_handler!` in
  [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs:1). Add a typed wrapper in
  [`src/lib/api/`](src/lib/api/settings.ts:1).
- **Add a new sidecar tool:** declare it in
  [`src-tauri/src/platform.rs`](src-tauri/src/platform.rs:1) for every supported
  (os, arch) combination, then call it via the runner from a service.
- **Add a long-running flow with progress:** look at how
  [`services/ipsw_prep.rs`](src-tauri/src/services/mod.rs:1) and the restore
  command coordinate runner streams + progress events.
- **Persist settings or workspace state:** see
  [`services/app_settings.rs`](src-tauri/src/services/mod.rs:1) and
  [`services/workspace.rs`](src-tauri/src/services/mod.rs:1).

## What NOT to do

- **Do not** re-add the deleted bash legacy (`restore.sh`, root `bin/`, the
  legacy `plans/` directory, or the previous `build.yml` workflow). Phase 1
  removed these intentionally.
- **Do not** add files to `src-tauri/binaries/` in a PR. The directory is
  populated only at release time.
- **Do not** re-add `.vscode/`, `.roo/`, `.claude/`, or other editor/agent
  scratch directories to git — these are gitignored on purpose.
- **Do not** call `invoke` directly from a Svelte component. Add or extend a
  wrapper in `src/lib/api/`.
- **Do not** spawn child processes outside `tools/runner.rs`. Logs will not
  appear in the terminal deck and cancellation will not work.
- **Do not** stringify errors across the command boundary. Use `AppError`.
- **Do not** introduce `svelte/store` writables; use Svelte 5 runes.
- **Do not** add `package-lock.json`; this repo uses pnpm.

## Roo modes

Roo Code mode definitions live in [`.roomodes`](.roomodes:1). The configured
modes (orchestrator, code, architect, ask, debug, ui-developer, local-llm,
mode-writer, security-review, project-research, skill-writer,
google-genai-developer) are tuned for this codebase and should be preferred over
ad-hoc roles. The orchestrator mode is the recommended entry point for
multi-step changes.

## Cross-references

- User-facing walkthroughs: [`docs/USER-GUIDE.md`](docs/USER-GUIDE.md:1)
- Bash → UI mapping: [`docs/MIGRATION-FROM-BASH.md`](docs/MIGRATION-FROM-BASH.md:1)
- Contributor checklist: [`CONTRIBUTING.md`](CONTRIBUTING.md:1)
- Security policy: [`SECURITY.md`](SECURITY.md:1)
