#!/bin/bash
# MCP Context Browser - User Service Installation Script
#
# Installs the MCP Context Browser as a systemd user service.
# The service runs under your user account and persists across logins.
#
# Usage: ./scripts/install-user-service.sh

set -e

# Colors for output (RED reserved for error messages)
# shellcheck disable=SC2034
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Installing MCP Context Browser as user service...${NC}"
echo ""

# Determine script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Check if binary exists
BINARY_SOURCE="$PROJECT_ROOT/target/release/mcb"
if [ ! -f "$BINARY_SOURCE" ]; then
    echo -e "${YELLOW}Release binary not found. Building with make...${NC}"
    cd "$PROJECT_ROOT"
    make build-release
fi

# Create required directories
echo "Creating directories..."
mkdir -p ~/.local/bin
mkdir -p ~/.config/mcb
mkdir -p ~/.local/share/mcb
mkdir -p ~/.config/systemd/user

# Copy binary
echo "Installing binary to ~/.local/bin/"
cp "$BINARY_SOURCE" ~/.local/bin/mcb
chmod 755 ~/.local/bin/mcb

# Copy config if it doesn't exist
CONFIG_DEST="$HOME/.config/mcb/config.toml"
if [ ! -f "$CONFIG_DEST" ]; then
    if [ -f "$PROJECT_ROOT/config.example.toml" ]; then
        echo "Creating default configuration..."
        cp "$PROJECT_ROOT/config.example.toml" "$CONFIG_DEST"
    else
        echo "Creating minimal configuration..."
        cat > "$CONFIG_DEST" << 'EOF'
# MCP Context Browser Configuration
# See documentation for all options

[transport]
mode = "hybrid"

[transport.http]
bind_address = "127.0.0.1"

[metrics]
# Unified port for Admin + Metrics + MCP HTTP (default: 3001)
port = 3001
enabled = true
EOF
    fi
    echo -e "${GREEN}Created config at $CONFIG_DEST${NC}"
else
    echo -e "${YELLOW}Config already exists at $CONFIG_DEST, checking for migration...${NC}"
    # Run config migration if needed
    "$SCRIPT_DIR/migrate-config.sh" "$CONFIG_DEST"
fi

# Install systemd service
echo "Installing systemd user service..."
cp "$PROJECT_ROOT/systemd/mcb.service" ~/.config/systemd/user/

# Enable lingering (keeps user services running after logout)
echo "Enabling user lingering..."
loginctl enable-linger "$USER" 2>/dev/null || true

# Reload systemd and enable service
echo "Reloading systemd and enabling service..."
systemctl --user daemon-reload
systemctl --user enable mcb

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}Installation complete!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo "Service locations:"
echo "  Binary:  ~/.local/bin/mcb"
echo "  Config:  ~/.config/mcb/config.toml"
echo "  Data:    ~/.local/share/mcb/"
echo "  Service: ~/.config/systemd/user/mcb.service"
echo ""
echo "Commands:"
echo "  Start:   systemctl --user start mcb"
echo "  Stop:    systemctl --user stop mcb"
echo "  Status:  systemctl --user status mcb"
echo "  Logs:    journalctl --user -u mcb -f"
echo "  Reload:  systemctl --user reload mcb"
echo "  Restart: systemctl --user restart mcb"
echo ""
echo "The service will auto-start on login (lingering enabled)."
echo ""
echo -e "${YELLOW}To start now:${NC} systemctl --user start mcb"
