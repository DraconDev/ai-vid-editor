## Where to publish releases

This repo ships binaries and installers to end users. Make sure each release is published in all relevant channels so customers can download whichever package suits their platform.

1. **GitHub Releases**
   - Upload `release/ai-vid-editor-<version>.tar.gz` and the corresponding `.sha256` alongside release notes (highlight installer fixes + UI updates).
   - Pin the release and mark it as “Latest” until the next release is created.

2. **Website / Download page**
   - Mirror the GitHub artifact URL or host a static copy on your website.
   - Display checksums and quick instructions: `tar -xzf ai-vid-editor-<version>.tar.gz` and `./install.sh --user`.

3. **Package repositories**
   - Update any Linux package (e.g., Nixpkgs overlay, Arch AUR, Debian, Ubuntu PPA) with the same version/hash.
   - Publish AppImage/Deb packages that wrap the release binary + assets.

4. **Desktop stores** (optional)
   - Microsoft Store / Snap / Flatpak: rebuild around the release binary and include the same icon/desktop entry.

5. **Release notes & docs**
   - Update `README.md`, `docs/customer-facing.md`, and `docs/release-locations.md` with the final version number.
   - Document any breaking changes (new config keys, removed flags) so support teams can reference them.

6. **Support communications**
   - Email / Slack / forum posts: link to the release artifact, highlight install script improvements, and mention the onboarding wizard.

Keeping the instructions in this doc ensures every release reaches every channel and remains easy for customers to find and install.
