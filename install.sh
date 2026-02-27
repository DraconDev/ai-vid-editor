#!/usr/bin/env bash
# =============================================================================
# AI Video Editor - Install Script
# =============================================================================
#
# Usage:
#   ./install.sh              # Install to /usr/local/bin (requires sudo)
#   ./install.sh --user       # Install to ~/.local/bin (no sudo)
#   ./install.sh --uninstall  # Remove installation
#
# Requirements:
#   - Rust toolchain (cargo)
#   - ffmpeg (for video processing)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
PREFIX="/usr/local"
USER_INSTALL=false
UNINSTALL=false
BIN_NAME="ai-vid-editor"
CONFIG_DIR="$HOME/.config/ai-vid-editor"
SERVICE_FILE="/etc/systemd/system/ai-vid-editor.service"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            USER_INSTALL=true
            PREFIX="$HOME/.local"
            shift
            ;;
        --uninstall)
            UNINSTALL=true
            shift
            ;;
        --help|-h)
            echo "AI Video Editor - Install Script"
            echo ""
            echo "Usage:"
            echo "  ./install.sh              Install to /usr/local/bin (requires sudo)"
            echo "  ./install.sh --user       Install to ~/.local/bin (no sudo)"
            echo "  ./install.sh --uninstall  Remove installation"
            echo ""
            echo "Options:"
            echo "  --user       Install to user directory (no sudo required)"
            echo "  --uninstall  Remove the installation"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# -----------------------------------------------------------------------------
# Uninstall
# -----------------------------------------------------------------------------
if [ "$UNINSTALL" = true ]; then
    echo -e "${BLUE}Uninstalling AI Video Editor...${NC}"
    
    # Remove binary
    if [ -f "$PREFIX/bin/$BIN_NAME" ]; then
        rm -f "$PREFIX/bin/$BIN_NAME"
        echo -e "${GREEN}✓ Removed binary from $PREFIX/bin/${NC}"
    else
        echo -e "${YELLOW}Binary not found at $PREFIX/bin/$BIN_NAME${NC}"
    fi
    
    # Remove icon
    for icon_path in "$HOME/.local/share/icons/hicolor/scalable/apps/$BIN_NAME.svg" \
                     "/usr/share/icons/hicolor/scalable/apps/$BIN_NAME.svg"; do
        if [ -f "$icon_path" ]; then
            rm -f "$icon_path"
            echo -e "${GREEN}✓ Removed icon from $icon_path${NC}"
        fi
    done
    
    # Remove desktop entry
    for desktop_path in "$HOME/.local/share/applications/$BIN_NAME.desktop" \
                        "/usr/share/applications/$BIN_NAME.desktop"; do
        if [ -f "$desktop_path" ]; then
            rm -f "$desktop_path"
            echo -e "${GREEN}✓ Removed desktop entry from $desktop_path${NC}"
        fi
    done
    
    # Remove systemd service (if exists and running as root)
    if [ -f "$SERVICE_FILE" ]; then
        if [ "$EUID" -eq 0 ]; then
            systemctl stop ai-vid-editor 2>/dev/null || true
            systemctl disable ai-vid-editor 2>/dev/null || true
            rm -f "$SERVICE_FILE"
            systemctl daemon-reload
            echo -e "${GREEN}✓ Removed systemd service${NC}"
        else
            echo -e "${YELLOW}Note: Systemd service exists but requires root to remove${NC}"
        fi
    fi
    
    # Ask about config directory
    if [ -d "$CONFIG_DIR" ]; then
        read -p "Remove config directory $CONFIG_DIR? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIG_DIR"
            echo -e "${GREEN}✓ Removed config directory${NC}"
        fi
    fi
    
    echo -e "${GREEN}Uninstall complete!${NC}"
    exit 0
fi

# -----------------------------------------------------------------------------
# Install
# -----------------------------------------------------------------------------
echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║           AI Video Editor - Installer                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust first:${NC}"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo -e "${GREEN}✓ Found cargo: $(cargo --version)${NC}"

# Check for ffmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${YELLOW}Warning: ffmpeg not found. Video processing will not work.${NC}"
    echo "  Install with: sudo apt install ffmpeg  (Debian/Ubuntu)"
    echo "  Or:           brew install ffmpeg      (macOS)"
else
    echo -e "${GREEN}✓ Found ffmpeg: $(ffmpeg -version | head -1)${NC}"
fi

# Create directories
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p "$PREFIX/bin"
mkdir -p "$CONFIG_DIR"

# Build release binary
echo -e "${BLUE}Building release binary...${NC}"
cargo build --release
echo -e "${GREEN}✓ Build complete${NC}"

# Install binary
BINARY_PATH="target/release/$BIN_NAME"
if [ -f "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "$PREFIX/bin/"
    chmod +x "$PREFIX/bin/$BIN_NAME"
    echo -e "${GREEN}✓ Installed binary to $PREFIX/bin/$BIN_NAME${NC}"
else
    echo -e "${RED}Error: Binary not found at $BINARY_PATH${NC}"
    exit 1
fi

# Install icon and desktop entry (for GUI)
if [ -d "assets" ]; then
    ICON_DIR=""
    DESKTOP_DIR=""
    
    if [ "$USER_INSTALL" = true ]; then
        ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
        DESKTOP_DIR="$HOME/.local/share/applications"
    else
        ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
        DESKTOP_DIR="/usr/share/applications"
    fi
    
    # Install icon
    if [ -f "assets/icon.svg" ]; then
        mkdir -p "$ICON_DIR"
        cp "assets/icon.svg" "$ICON_DIR/$BIN_NAME.svg"
        echo -e "${GREEN}✓ Installed icon to $ICON_DIR/$BIN_NAME.svg${NC}"
        
        # Update icon cache
        if command -v gtk-update-icon-cache &> /dev/null; then
            gtk-update-icon-cache -q "$(dirname "$ICON_DIR")" 2>/dev/null || true
        fi
    fi
    
    # Install desktop entry
    if [ -f "assets/$BIN_NAME.desktop" ]; then
        mkdir -p "$DESKTOP_DIR"
        cp "assets/$BIN_NAME.desktop" "$DESKTOP_DIR/"
        echo -e "${GREEN}✓ Installed desktop entry to $DESKTOP_DIR/$BIN_NAME.desktop${NC}"
        
        # Update desktop database
        if command -v update-desktop-database &> /dev/null; then
            update-desktop-database -q "$DESKTOP_DIR" 2>/dev/null || true
        fi
    fi
fi

# Install example config if not exists
CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Config file already exists at $CONFIG_FILE${NC}"
    echo -e "${YELLOW}Skipping config installation (backup your config and re-run to update)${NC}"
else
    if [ -f "ai-vid-editor.example.toml" ]; then
        cp "ai-vid-editor.example.toml" "$CONFIG_FILE"
        echo -e "${GREEN}✓ Installed config to $CONFIG_FILE${NC}"
    else
        # Generate default config
        "$PREFIX/bin/$BIN_NAME" --generate-config > "$CONFIG_FILE"
        echo -e "${GREEN}✓ Generated default config at $CONFIG_FILE${NC}"
    fi
fi

# Add to PATH if needed (for --user install)
if [ "$USER_INSTALL" = true ]; then
    if [[ ":$PATH:" != *":$PREFIX/bin:"* ]]; then
        echo -e "${YELLOW}Note: $PREFIX/bin is not in your PATH${NC}"
        echo -e "${YELLOW}Add this to your ~/.bashrc or ~/.zshrc:${NC}"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
fi

# Ask about systemd service (only for system-wide install)
if [ "$USER_INSTALL" = false ] && [ "$EUID" -eq 0 ]; then
    read -p "Install systemd service for daemon mode? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cat > "$SERVICE_FILE" << 'EOF'
[Unit]
Description=AI Video Editor Daemon
After=network.target

[Service]
Type=simple
User=%USER%
WorkingDirectory=%HOME%
ExecStart=/usr/local/bin/ai-vid-editor --config %HOME%/.config/ai-vid-editor/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
        # Replace placeholders
        sed -i "s/%USER%/$SUDO_USER/g" "$SERVICE_FILE"
        sed -i "s|%HOME%|$HOME|g" "$SERVICE_FILE"
        
        systemctl daemon-reload
        echo -e "${GREEN}✓ Installed systemd service${NC}"
        echo -e "${BLUE}To enable and start:${NC}"
        echo "  sudo systemctl enable ai-vid-editor"
        echo "  sudo systemctl start ai-vid-editor"
    fi
fi

# Summary
echo ""
echo -e "${GREEN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                  Installation Complete!                    ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""
echo "Binary:     $PREFIX/bin/$BIN_NAME"
echo "Config:     $CONFIG_FILE"
echo ""
echo "Quick Start:"
echo "  1. Edit config:  nano $CONFIG_FILE"
echo "  2. Set up project directories (watch/, output/, music/)"
echo "  3. Run:          $BIN_NAME --config $CONFIG_FILE"
echo ""
echo "For daemon mode (background):"
if [ "$USER_INSTALL" = true ]; then
    echo "  nohup $BIN_NAME --config $CONFIG_FILE > /tmp/ai-vid-editor.log 2>&1 &"
else
    echo "  sudo systemctl enable --now ai-vid-editor"
fi
echo ""