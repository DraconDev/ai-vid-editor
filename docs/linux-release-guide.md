# Linux Release Guide

This guide outlines the steps to prepare, package, and publish AI Video Editor for Linux users. It assumes you only target Linux distributions and want a frictionless installer or downloadable bundle.

## 1. Prerequisites
- Ensure all code changes are merged and tests pass:
  ```bash
  cargo test --all-features
  cargo clippy --all-features
  ```
- Update the version in `Cargo.toml` and record release notes in `CHANGELOG.md`.

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
| Tarball   | Compressed archive (`.tar.gz`) containing binary, assets, install script, and docs. |
| DEB/RPM   | Native Debian/RPM packages for Debian/Ubuntu and Fedora/RHEL.                    |
| Flatpak   | Flatpak bundle for Flathub or custom Flatpak repos (sandboxed).                  |
| Snap      | Snapcraft package for Snap Store or manual `snap install`.                        |
| Arch/AUR  | PKGBUILD for Arch/AUR users and derivatives.                                      |

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
- Use `cargo deb` or `cargo rpm`, or packaging helpers like `fpm`, to create `.deb`/`.rpm` artifacts:
  ```bash
  cargo deb --no-build --version <version>
  cargo rpm --no-build --release --target-dir release/rpm
  ```

### 3.4 Flatpak
1. Create a Flatpak manifest (JSON/YAML) that references `com.ai.videditor`, the Freedesktop runtime, and your repo. Example module snippet:
   ```json
   {
     "name": "ai-vid-editor",
     "buildsystem": "simple",
     "build-commands": ["cargo build --release"],
     "sources": [{"type": "git", "url": "https://github.com/DraconDev/ai-vid-editor", "branch": "master"}]
   }
   ```
2. Build and export:
   ```bash
   flatpak-builder --repo=repo build-dir manifest.json
   flatpak build-bundle repo ai-vid-editor.flatpak com.ai.videditor
   ```

### 3.5 Snap
1. Write `snap/snapcraft.yaml` describing the binary, icon, desktop entry, and confinement level.
2. Run `snapcraft` to build, then `snapcraft push ... --release stable` to publish on the Snap Store (or share the `.snap`).

### 3.6 Arch/AUR
1. Maintain a `PKGBUILD` (either inside this repo or separately) with the release tarball URL and checksum.
2. Users can install via `yay -S ai-vid-editor` or other AUR helpers. Include instructions for building from source if necessary.

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
