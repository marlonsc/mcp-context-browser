#!/bin/bash
# MCP Context Browser - User Service Uninstallation Script
#
# Removes the MCP Context Browser systemd user service.
# Optionally removes data and configuration.
#
# Usage: ./scripts/uninstall-user-service.sh [--all]
#   --all: Also remove configuration and data

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

REMOVE_ALL=false
if [ "$1" = "--all" ]; then
    REMOVE_ALL=true
fi

echo -e "${YELLOW}Removing MCP Context Browser user service...${NC}"
echo ""

# Stop and disable service
echo "Stopping service..."
systemctl --user stop mcb 2>/dev/null || true
systemctl --user disable mcb 2>/dev/null || true

# Remove service file
echo "Removing service file..."
rm -f ~/.config/systemd/user/mcb.service

# Remove binary
echo "Removing binary..."
rm -f ~/.local/bin/mcb

# Reload systemd
systemctl --user daemon-reload

echo ""
echo -e "${GREEN}Service removed.${NC}"
echo ""

if [ "$REMOVE_ALL" = true ]; then
    echo -e "${YELLOW}Removing configuration and data...${NC}"
    rm -rf ~/.config/mcb
    rm -rf ~/.local/share/mcb
    echo -e "${GREEN}All data removed.${NC}"
else
    echo "Data preserved at:"
    echo "  Config: ~/.config/mcb/"
    echo "  Data:   ~/.local/share/mcb/"
    echo ""
    echo -e "${YELLOW}To remove all data:${NC} $0 --all"
fi

echo ""
echo -e "${GREEN}Uninstallation complete.${NC}"
