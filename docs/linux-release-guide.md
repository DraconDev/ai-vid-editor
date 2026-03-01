# Linux Release Guide

This guide outlines the steps to prepare, package, and publish AI Video Editor for Linux users. It assumes you only target Linux distributions and want a frictionless installer or downloadable bundle.

## 1. Prerequisites
- Ensure all code changes are merged and tests pass:
  ```bash
  cargo test --all-features
  cargo clippy --all-features
  ```
- Update the version in `Cargo.toml` and record release notes in `CHANGELOG.md` or your release tracker.

## 2. Build the Release Binary
```bash
cargo build --release
```  
The resulting binary lives at `target/release/ai-vid-editor`.

## 3. Choose a Packaging Format
Select one or more options below based on your distribution channel:

| Format    | Description                                                                        |
|-----------|------------------------------------------------------------------------------------|
| AppImage  | Single executable bundle; runs without installation on most distros.                |
| Tarball   | Compressed archive (`.tar.gz`) containing binary, assets, install script, and docs.|
| DEB/RPM   | Native Debian/RPM package for system package managers.                             |

### 3.1 AppImage (Recommended)
1. Install or configure an AppImage builder (e.g. `appimage-builder`).
2. Create a recipe referencing:
   - `target/release/ai-vid-editor`
   - `assets/ai-vid-editor.desktop`
   - `assets/icon.svg`
3. Run the builder to produce `ai-vid-editor-x86_64.AppImage`.

### 3.2 Tarball + Installer Script
```bash
./scripts/release.sh <version>
```  
This script packages:
- `target/release/ai-vid-editor`
- `install.sh` (installer)
- `assets/` (desktop entry & icon)
- `docs/` (user guides)
and creates `release/ai-vid-editor-<version>.tar.gz` plus a `.sha256` checksum.

### 3.3 DEB/RPM
- Use `cargo deb` or `cargo rpm`, or a generic tool like `fpm`, to generate native packages:
  ```bash
  cargo deb --no-build --version <version>
  ```

## 4. Publish Artifacts
1. **GitHub Releases**: upload chosen artifacts (`.AppImage`, `.tar.gz`, `.deb`, etc.) and checksum files. Use the GitHub CLI:
   ```bash
   gh release create v<version> release/*<version>* --title "v<version>" --notes-file CHANGELOG.md
   ```
2. **Website/Download Page**: mirror or link to the GitHub release assets. Display checksums and quick install instructions.
3. **Package Repositories**: push the `.deb` to a PPA or update AUR/Flathub manifest if maintained.

## 5. Update Documentation
- Ensure `README.md` and `docs/customer-facing.md` reflect your chosen packaging format:
  - How to run the AppImage
  - How to extract and run the tarball (`./install.sh --user`)
  - How to install the DEB/RPM
- Update `docs/release-locations.md` with final download URLs.

## 6. Announce & Support
- Announce the release via email, blog, or social channels with links to the artifacts and basic install commands.
- Update support playbook to reference:
  - Installation troubleshooting (missing `ffmpeg`, permission issues)
  - Log locations (`~/.config/ai-vid-editor`, `~/.cache/ai-vid-editor`)

---
Following this guide ensures a repeatable, automated Linux release process and a smooth install experience for end users.
