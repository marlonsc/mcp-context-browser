#!/bin/bash
# MCP Context Browser - Config Migration Script
#
# Migrates old configuration format to new provider structure.
# This script converts direct provider sections to named provider sections.
#
# Usage: ./scripts/migrate-config.sh [config_file]
#
# If config_file is not provided, uses ~/.config/mcb/config.toml

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration file to migrate
CONFIG_FILE="${1:-$HOME/.config/mcb/config.toml}"

echo -e "${GREEN}MCP Context Browser - Config Migration${NC}"
echo "Migrating configuration file: $CONFIG_FILE"
echo ""

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Config file does not exist: $CONFIG_FILE${NC}"
    echo "Nothing to migrate."
    exit 0
fi

# Create backup
BACKUP_FILE="${CONFIG_FILE}.backup.$(date +%Y%m%d_%H%M%S)"
echo "Creating backup: $BACKUP_FILE"
cp "$CONFIG_FILE" "$BACKUP_FILE"

# Function to migrate a provider section
migrate_provider() {
    local provider_type="$1"
    local section_name="$2"

    # Check if old format exists
    if grep -q "^\[$section_name\]$" "$CONFIG_FILE"; then
        echo "Found old format for $provider_type provider, migrating..."

        # Extract the provider configuration
        local temp_file=$(mktemp)
        local in_section=false
        local next_section=false

        while IFS= read -r line; do
            if [[ "$line" =~ ^\[$section_name\] ]]; then
                in_section=true
                echo "[$section_name.default]" >> "$temp_file"
                continue
            fi

            if $in_section && [[ "$line" =~ ^\[.*\] ]] && [[ ! "$line" =~ ^\[$section_name ]]; then
                next_section=true
                in_section=false
            fi

            if $in_section && ! $next_section; then
                # Convert 'host' to 'base_url' for compatibility
                if [[ "$line" =~ ^[[:space:]]*host[[:space:]]*=[[:space:]]* ]]; then
                    line=$(echo "$line" | sed 's/host/base_url/')
                    echo "$line" >> "$temp_file"
                # Skip empty lines and comments at the beginning
                elif [[ -n "$line" ]] && [[ ! "$line" =~ ^[[:space:]]*# ]]; then
                    echo "$line" >> "$temp_file"
                elif [[ "$line" =~ ^[[:space:]]*# ]]; then
                    echo "$line" >> "$temp_file"
                fi
            elif ! $in_section; then
                echo "$line" >> "$temp_file"
            fi
        done < "$CONFIG_FILE"

        # Replace original file
        mv "$temp_file" "$CONFIG_FILE"

        echo -e "${GREEN}✓ Migrated $provider_type provider configuration${NC}"
    else
        echo "No migration needed for $provider_type provider"
    fi
}

# Migrate embedding provider
migrate_provider "embedding" "providers.embedding"

# Migrate vector store provider
migrate_provider "vector_store" "providers.vector_store"

echo ""
echo -e "${GREEN}Migration complete!${NC}"
echo "Backup saved to: $BACKUP_FILE"
echo ""
echo "Please review the migrated configuration and adjust as needed."
echo "You may need to restart the service: systemctl --user restart mcb"