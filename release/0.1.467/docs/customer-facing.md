# Making AI Video Editor customer-ready

This document captures what we already ship (desktop install script, GUI launcher, first-run wizard) and what steps are still needed before AI Video Editor can be handed to non-technical customers.

## Current strengths
1. **Single installer** (`./install.sh --user`) builds the current release, installs `ai-vid-editor --gui`, installs the icon and desktop entry, and optionally wires a system-wide daemon.
2. **Desktop integration** now overwrites the desktop entry on every install so clicking the dash launches the GUI, and the icon is refreshed via `assets/icon.svg` (purple gradient film + AI badge).
3. **First-run wizard** introduces the product to non-technical users: welcome screen, folder picker, content preset, and audio/mute toggles that automatically save into the config and start watch mode.
4. **CLI & GUI parity** is complete – the same binary contains both modes, both read `[[paths.watch_folders]]` from the same config file (`~/.config/ai-vid-editor/config.toml`, auto-loaded), and the README documents GUI/CLI usage and the install script.

## Remaining work to go fully customer-facing
| Area | Work | Why it matters |
| --- | --- | --- |
| UX polish | Run the GUI through a design review (tutorial overlay, localized strings, accessibility, onboarding videos). Document keyboard shortcuts and keyboard focus flow. | Customers need a polished, confident experience. UX polish reduces support load. |
| Packaging & distribution | Produce versioned binaries for macOS, Windows, Linux (AppImage, DMG, MSI). Build signed installers or notarized packages, add auto-update channel (e.g., GitHub Releases + Sparkle or rustup-updater). | Non‑technical customers expect “Download → Run” without Rust/tooling. |
| Documentation | Expand README into a user manual (Getting Started, Config reference, Watch folders, Troubleshooting, FAQ). Include a quick tutorial GIF/video, sample project, and localized versions. | Customers expect guides; support teams rely on docs to triage. |
| Support & telemetry | Integrate optional crash reporting/usage telemetry (with opt-in). Add in-app logging viewer, export logs, easy way to copy/paste error output. | Helps diagnose issues quickly, especially for background processing. |
| Testing & QA | Add automated GUI smoke tests (egui screenshot tests or runheadless). Maintain regression suites for macOS/Windows builds and nightly builds. Document how to run `cargo test`/`cargo clippy` so release engineers can repeat. | Ensures stability across platforms before shipping to customers. |
| Operational readiness | Define release cadence, changelog process, versioning policy. Publish packaged binaries/updates and announce via website/social. Provide install/uninstall support. | Customers need confidence that updates are safe and supported; operations need a repeatable workflow. |
| Security & compliance | Offer arm64/linux builds, document dependencies (FFmpeg, Rust versions), perform dependency CVE scans, sign bundles if required. | Enterprises look for compliance assurance. |

## Next steps for a publication-ready release
1. **Curate a release checklist** that includes UI regressions, documentation proofing, QA on each platform, and verifying the install script with `--user` and `sudo`. Keep a `RELEASE.md` that references the checklist plus `cargo` commands used to build artifacts.
2. **Build installers** using GitHub Actions + `cargo` cross, releasing artifacts for Linux, macOS, Windows. Couple each release with an FAQ (common errors, how to open logs, where to find config). 
3. **Ship a companion website** (README/Docs + support portal) that explains the onboarding wizard, CLI commands, converting to vertical, watch folders, and manual troubleshooting steps (missing `ffmpeg`, permissions, etc.).
4. **Train a support playbook**: how to collect logs from `~/.cache/ai-vid-editor`/`~/.config/ai-vid-editor`, comprehending notify-daemon output, recreating issues with `ai-vid-editor --dry-run`, and how to reinstall via `./install.sh --user`.

Keeping this document alongside your release plan makes it easier to spot gaps early and tick off requirements as they are delivered.
