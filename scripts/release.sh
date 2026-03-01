#!/usr/bin/env bash
# Release helper that builds, packages, and snapshots artifacts for customers.
#
# Usage:
#   ./scripts/release.sh 0.1.423         # release with explicit version
#   ./scripts/release.sh                  # release with timestamped version

set -euo pipefail

cd "$(dirname "$0")/.."

VERSION=${1:-"0.1.$(date +%Y%m%d%H%M)"}
RELEASE_ROOT="release"
DIST_DIR="$RELEASE_ROOT/$VERSION"
ARCHIVE="$RELEASE_ROOT/ai-vid-editor-$VERSION.tar.gz"

info() {
  printf '\033[1;34m> %s\033[0m\n' "$1"
}

info "Running full test suite (cargo test + clippy)"
cargo test --all-features
cargo clippy --all-features

info "Building release binary"
cargo build --release

info "Preparing release directory"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"/{bin,assets,docs}

info "Copying binary + assets"
cp "target/release/ai-vid-editor" "$DIST_DIR/bin/"
cp assets/icon.svg assets/ai-vid-editor.desktop "$DIST_DIR/assets/"

info "Bundling docs"
cp README.md docs/customer-facing.md docs/release-locations.md "$DIST_DIR/docs/"
cp install.sh "$DIST_DIR/"

info "Creating archive $ARCHIVE"
rm -f "$ARCHIVE"
tar -czf "$ARCHIVE" -C "$DIST_DIR" .

info "Generating checksum"
sha256sum "$ARCHIVE" > "$RELEASE_ROOT/ai-vid-editor-$VERSION.sha256"

info "Release $VERSION ready in $RELEASE_ROOT/"
