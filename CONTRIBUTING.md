# Contributing to LegacyKit

Thanks for your interest in contributing. LegacyKit is a community project for
restoring, downgrading, and jailbreaking legacy iOS devices, and it survives on
contributions from people who care about keeping these devices usable.

## Scope

In-scope contributions include:

- Bug fixes and reliability improvements in the Tauri app
- New views, flows, or device-support coverage
- UX, accessibility, and i18n improvements
- Documentation fixes (`README.md`, `AGENTS.md`, `docs/*`)
- CI / build pipeline improvements
- Sidecar wiring updates as upstream tools evolve

Out of scope:

- Changes to upstream sidecar tool source (futurerestore, idevicerestore,
  gaster, ipwnder, etc.) — contribute those upstream
- Bundling or vendoring binaries into git
- Windows support (not a current target)


## Development setup

### Prerequisites

- **Node.js 20+**
- **pnpm** (install via Corepack: `corepack enable`)
- **Rust 1.77+** with `rustfmt` and `clippy` (`rustup component add rustfmt clippy`)
- **Tauri 2 platform prerequisites** — the canonical, up-to-date list is at
  https://v2.tauri.app/start/prerequisites/. Highlights:
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux (Debian/Ubuntu):** `libwebkit2gtk-4.1-dev`, `build-essential`,
    `curl`, `wget`, `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`,
    `librsvg2-dev`
  - **Linux (Fedora):** `webkit2gtk4.1-devel`, `openssl-devel`,
    `libappindicator-gtk3-devel`, `librsvg2-devel`, plus `@development-tools`
  - **Linux (Arch):** `webkit2gtk-4.1`, `base-devel`, `curl`, `wget`, `file`,
    `openssl`, `appmenu-gtk-module`, `libappindicator-gtk3`, `librsvg`
- **libusb / libimobiledevice headers** are required to build / run device
  features locally on Linux.

### First run

```bash
pnpm install
pnpm tauri dev
```

`pnpm tauri dev` rebuilds Rust on changes. The frontend hot-reloads.

### Sidecar binaries

The [`src-tauri/binaries/`](src-tauri/Cargo.toml:1) directory is **gitignored**
and is populated only at release time. To exercise device flows locally you'll
need to assemble it yourself (an archive will be attached to releases — TBD).
For purely UI-level work this isn't required; the app will start without it,
but device-side actions will fail.

## Branches and PRs

- Work in feature branches off `main` (e.g., `fix/restore-progress-stall`,
  `feat/data-import`).
- Use clear, conventional-ish commit messages: `feat: …`, `fix: …`,
  `docs: …`, `refactor: …`, `chore: …`. Imperative mood, present tense.
- Reference an issue when one exists: "Closes #123" or "Refs #123".
- Keep PRs focused. Multiple unrelated changes should be multiple PRs.
- Rebase on `main` before requesting review; avoid merge commits in feature
  branches.

## Pre-PR checklist

Run all of these locally and ensure they pass before opening a PR:

- [ ] `pnpm svelte-check` — clean
- [ ] `pnpm tsc --noEmit -p tsconfig.app.json` — clean
- [ ] `cd src-tauri && cargo fmt --check` — clean
- [ ] `cd src-tauri && cargo clippy --all-targets -- -D warnings` — clean
      (current policy: warnings are errors; if you need to relax this, justify
      it in the PR description)
- [ ] App builds: `pnpm tauri build` (or at minimum `pnpm tauri dev` starts
      cleanly)
- [ ] Manually tested on **at least one** of macOS or Linux. Note which in the
      PR description. Cross-platform changes should ideally be tested on both.
- [ ] Updated relevant docs ([`README.md`](README.md:1),
      [`AGENTS.md`](AGENTS.md:1), [`docs/USER-GUIDE.md`](docs/USER-GUIDE.md:1))
      if behaviour visible to users or agents changed.

CI ([`.github/workflows/ci.yml`](.github/workflows/ci.yml:1)) runs the same
gates on Ubuntu and macOS for every PR.

## Code conventions

The full set of conventions (frontend runes patterns, API wrapper rules,
backend command/service split, error handling, sidecar invariants) is
documented in [`AGENTS.md`](AGENTS.md:1). Read that file before adding a new
view, command, or sidecar.

## Adding a sidecar tool

If your change introduces a new third-party CLI dependency:

1. Add the tool to [`src-tauri/src/platform.rs`](src-tauri/src/platform.rs:1)
   for every supported (os, arch) combination — failures should be explicit
   when a platform isn't covered.
2. Invoke the tool via [`src-tauri/src/tools/runner.rs`](src-tauri/src/tools/runner.rs:1)
   so its output streams into the terminal deck.
3. **Do not** add the binary to git. Document where the binary comes from
   (upstream repo, build flags) in your PR description so it can be added to
   the release-time binary archive.
4. Update [`README.md`](README.md:1) and [`AGENTS.md`](AGENTS.md:1) acknowledgments
   if the tool isn't already mentioned.

## Reporting bugs / feature requests

Open an issue on
[GitHub Issues](https://github.com/Drakeym132/LegacyKit/issues). Helpful info:

- Your OS and architecture (e.g., macOS 14.5 arm64, Ubuntu 24.04 x64)
- LegacyKit version (about / settings or release tag)
- Device model, iOS version, and current mode (Normal / Recovery / DFU)
- Workspace `logs/<session>.log` excerpt around the failure (redact serials /
  ECID if you prefer)
- Steps to reproduce

## Security issues

**Do not** file security issues as public GitHub Issues. See
[`SECURITY.md`](SECURITY.md:1) for the disclosure policy.
