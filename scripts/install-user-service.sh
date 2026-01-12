#!/bin/bash
# MCP Context Browser - User Service Installation Script
#
# Installs the MCP Context Browser as a systemd user service.
# The service runs under your user account and persists across logins.
#
# Usage: ./scripts/install-user-service.sh

set -e

# Colors for output
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
BINARY_SOURCE="$PROJECT_ROOT/target/release/mcp-context-browser"
if [ ! -f "$BINARY_SOURCE" ]; then
    echo -e "${YELLOW}Release binary not found. Building...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release
fi

# Create required directories
echo "Creating directories..."
mkdir -p ~/.local/bin
mkdir -p ~/.config/mcp-context-browser
mkdir -p ~/.local/share/mcp-context-browser
mkdir -p ~/.config/systemd/user

# Copy binary
echo "Installing binary to ~/.local/bin/"
cp "$BINARY_SOURCE" ~/.local/bin/mcp-context-browser
chmod 755 ~/.local/bin/mcp-context-browser

# Copy config if it doesn't exist
CONFIG_DEST="$HOME/.config/mcp-context-browser/config.toml"
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
    echo -e "${YELLOW}Config already exists at $CONFIG_DEST (not overwritten)${NC}"
fi

# Install systemd service
echo "Installing systemd user service..."
cp "$PROJECT_ROOT/systemd/mcp-context-browser.service" ~/.config/systemd/user/

# Enable lingering (keeps user services running after logout)
echo "Enabling user lingering..."
loginctl enable-linger "$USER" 2>/dev/null || true

# Reload systemd and enable service
echo "Reloading systemd and enabling service..."
systemctl --user daemon-reload
systemctl --user enable mcp-context-browser

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}Installation complete!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo "Service locations:"
echo "  Binary:  ~/.local/bin/mcp-context-browser"
echo "  Config:  ~/.config/mcp-context-browser/config.toml"
echo "  Data:    ~/.local/share/mcp-context-browser/"
echo "  Service: ~/.config/systemd/user/mcp-context-browser.service"
echo ""
echo "Commands:"
echo "  Start:   systemctl --user start mcp-context-browser"
echo "  Stop:    systemctl --user stop mcp-context-browser"
echo "  Status:  systemctl --user status mcp-context-browser"
echo "  Logs:    journalctl --user -u mcp-context-browser -f"
echo "  Reload:  systemctl --user reload mcp-context-browser"
echo "  Restart: systemctl --user restart mcp-context-browser"
echo ""
echo "The service will auto-start on login (lingering enabled)."
echo ""
echo -e "${YELLOW}To start now:${NC} systemctl --user start mcp-context-browser"
