#!/bin/bash

# MCP Context Browser to MCB Rename Script
# This script handles renaming the project from mcb to mcb
# Supports dry-run mode for validation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DRY_RUN=false
VERBOSE=false
BACKUP=true

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Change patterns - ordered by specificity (most specific first)
declare -A CHANGE_PATTERNS=(
    # Repository URLs (most specific)
    ["https://github.com/marlonsc/mcb"]="https://github.com/marlonsc/mcb"

    # Data and config directories
    ["$HOME/.local/share/mcb"]="$HOME/.local/share/mcb"
    ["$HOME/.config/mcb"]="$HOME/.config/mcb"

    # Crate and binary names
    ["mcb"]="mcb"
)

# Files to exclude from processing (contain references that shouldn't change)
EXCLUDE_PATTERNS=(
    # Docker compose files have "mcp-" prefixed services that are different
    "docker-compose*.yml"
    "docker-compose*.yaml"
    # These files have different contexts
    "CODE_REVIEW_REPORT.md"
    "CLAUDE.md"
    "REFACTORING_BASELINE.md"
)

# Files that require special handling
SPECIAL_FILES=(
    "Cargo.toml"
    "src/main.rs"
    "systemd/mcb.service"
    "config/default.toml"
)

show_help() {
    cat << EOF
MCP Context Browser to MCB Rename Script

This script renames the project from 'mcb' to 'mcb' across all files.

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --dry-run           Show what would be changed without making changes
    --apply             Apply the changes (default behavior when no flags given)
    --verbose, -v       Show detailed output
    --no-backup         Don't create backup files
    --help, -h          Show this help message

EXAMPLES:
    # Dry run to see what would change
    $0 --dry-run

    # Apply changes with verbose output
    $0 --apply --verbose

    # Apply changes without backups (dangerous!)
    $0 --apply --no-backup

DESCRIPTION:
    This script handles multiple types of renames:
    - Repository URLs: marlonsc/mcb → marlonsc/mcb
    - Data directories: ~/.local/share/mcb → ~/.local/share/mcb
    - Config directories: ~/.config/mcb → ~/.config/mcb
    - Crate/binary names: mcb → mcb

    The script excludes certain files that contain 'mcp-' references in different contexts.
EOF
}

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

should_exclude_file() {
    local file="$1"
    for pattern in "${EXCLUDE_PATTERNS[@]}"; do
        if [[ "$file" == "$pattern" ]]; then
            return 0
        fi
    done
    return 1
}

backup_file() {
    local file="$1"
    local backup
    backup="${file}.backup.$(date +%Y%m%d_%H%M%S)"
    if $VERBOSE; then
        log_info "Creating backup: $backup"
    fi
    cp "$file" "$backup"
}

validate_change() {
    local file="$1"
    local _old_content="$2"
    local new_content="$3"

    # Basic validation - ensure we haven't broken anything obvious (_old_content available for diff)
    if [[ "$file" == "Cargo.toml" ]]; then
        # Check that name field is valid
        if ! grep -q "^name = \"mcb\"$" <<< "$new_content"; then
            log_error "Cargo.toml name field validation failed"
            return 1
        fi
    fi

    if [[ "$file" == "src/main.rs" ]]; then
        # Check that command name is valid
        if ! grep -q "#\[command(name = \"mcb\"\)\]" <<< "$new_content"; then
            log_error "src/main.rs command name validation failed"
            return 1
        fi
    fi

    return 0
}

apply_changes_to_file() {
    local file="$1"
    local changes_made=0
    local original_content
    local new_content

    if [[ ! -f "$file" ]]; then
        if $VERBOSE; then
            log_warn "File not found: $file"
        fi
        return 0
    fi

    original_content=$(cat "$file")

    # Apply all change patterns
    new_content="$original_content"
    for old_pattern in "${!CHANGE_PATTERNS[@]}"; do
        local new_pattern="${CHANGE_PATTERNS[$old_pattern]}"

        # Check if this pattern exists in the file
        if grep -q "$old_pattern" "$file"; then
            if $DRY_RUN; then
                log_info "Would change in $file: '$old_pattern' → '$new_pattern'"
            else
                # Use sed to replace all occurrences (dynamic pattern - cannot use ${var//search/replace})
                # shellcheck disable=SC2001
                new_content=$(sed "s|$old_pattern|$new_pattern|g" <<< "$new_content")
                ((changes_made++))
            fi
        fi
    done

    if [[ $changes_made -gt 0 ]]; then
        # Only validate in apply mode since dry-run mode doesn't modify content
        if ! $DRY_RUN; then
            if ! validate_change "$file" "$original_content" "$new_content"; then
                log_error "Validation failed for $file"
                return 1
            fi
        fi

        if $BACKUP && ! $DRY_RUN; then
            backup_file "$file"
        fi

        if ! $DRY_RUN; then
            echo "$new_content" > "$file"
            log_success "Updated $file ($changes_made changes)"
        fi
        return 0
    fi

    return 0
}

handle_special_files() {
    local file="$1"

    case "$file" in
        "systemd/mcb.service")
            # This file needs to be renamed to systemd/mcb.service
            local new_name="systemd/mcb.service"
            if $DRY_RUN; then
                log_info "Would rename: $file → $new_name"
            else
                if $BACKUP; then
                    backup_file "$file"
                fi
                mv "$file" "$new_name"
                log_success "Renamed $file → $new_name"

                # Now apply changes to the renamed file
                apply_changes_to_file "$new_name"
            fi
            ;;
        *)
            apply_changes_to_file "$file"
            ;;
    esac
}

process_files() {
    if $VERBOSE; then
        log_info "Starting file processing..."
    fi
    local total_files=0
    local changed_files=0

    # Process regular files using a temporary file to avoid process substitution issues
    local temp_file
    temp_file=$(mktemp)
    if $VERBOSE; then
        log_info "Finding files to process..."
    fi
    find "$PROJECT_ROOT" -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yml" -o -name "*.yaml" -o -name "*.sh" -o -name "*.service" -o -name "*.mk" -o -name "Makefile" -o -name "*.json" \) -print0 > "$temp_file"
    if $VERBOSE; then
        local file_count
        file_count=$(tr -cd '\0' < "$temp_file" | wc -c)
        log_info "Found $file_count files to process"
    fi

    while IFS= read -r -d '' file; do
        ((total_files++))
        if should_exclude_file "$file"; then
            if $VERBOSE; then
                log_info "Skipping excluded file: $file"
            fi
            continue
        fi

        if [[ " ${SPECIAL_FILES[*]} " == *" $file "* ]]; then
            if handle_special_files "$file"; then
                ((changed_files++))
            fi
        else
            if apply_changes_to_file "$file"; then
                ((changed_files++))
            fi
        fi
    done < "$temp_file"

    # Clean up temp file
    rm -f "$temp_file"

    # Special handling for book.toml
    if [[ -f "book.toml" ]]; then
        ((total_files++))
        if apply_changes_to_file "book.toml"; then
            ((changed_files++))
        fi
    fi

    if $VERBOSE; then
        log_info "Processed $total_files files, $changed_files would be changed"
    fi
}

validate_changes() {
    log_info "Validating changes..."

    # Check that critical files have been updated correctly
    local checks_passed=0
    local total_checks=0

    ((total_checks++))
    if [[ -f "Cargo.toml" ]] && grep -q "^name = \"mcb\"$" Cargo.toml; then
        log_success "Cargo.toml name field updated correctly"
        ((checks_passed++))
    else
        log_error "Cargo.toml name field not updated correctly"
    fi

    ((total_checks++))
    if [[ -f "src/main.rs" ]] && grep -q "#\[command(name = \"mcb\"\)\]" src/main.rs; then
        log_success "src/main.rs command name updated correctly"
        ((checks_passed++))
    else
        log_error "src/main.rs command name not updated correctly"
    fi

    ((total_checks++))
    if [[ ! -f "systemd/mcb.service" ]] && [[ -f "systemd/mcb.service" ]]; then
        log_success "Systemd service file renamed correctly"
        ((checks_passed++))
    else
        log_error "Systemd service file not renamed correctly"
    fi

    log_info "Validation: $checks_passed/$total_checks checks passed"

    if [[ $checks_passed -eq $total_checks ]]; then
        log_success "All validation checks passed!"
        return 0
    else
        log_error "Some validation checks failed!"
        return 1
    fi
}

main() {
    # Track if a mode was explicitly specified
    local mode_specified=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                mode_specified=true
                shift
                ;;
            --apply)
                DRY_RUN=false
                mode_specified=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --no-backup)
                BACKUP=false
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # Default to apply mode if no mode was specified
    if [[ $mode_specified == false ]]; then
        DRY_RUN=false
    fi

    # Change to project root
    cd "$PROJECT_ROOT"

    log_info "MCP Context Browser → MCB Rename Script"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Mode: $(if $DRY_RUN; then echo 'DRY RUN'; else echo 'APPLY CHANGES'; fi)"
    log_info "Backup: $(if $BACKUP; then echo 'enabled'; else echo 'disabled'; fi)"

    if $DRY_RUN; then
        log_warn "DRY RUN MODE - No changes will be made"
    else
        log_warn "APPLY MODE - Changes will be made to files"
        if ! $BACKUP; then
            log_error "BACKUP DISABLED - This could be dangerous!"
            read -p "Are you sure you want to continue without backups? (y/N): " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                log_info "Operation cancelled"
                exit 0
            fi
        fi
    fi

    echo
    log_info "Change patterns:"
    for old_pattern in "${!CHANGE_PATTERNS[@]}"; do
        echo "  '$old_pattern' → '${CHANGE_PATTERNS[$old_pattern]}'"
    done

    echo
    log_info "Excluded patterns:"
    for pattern in "${EXCLUDE_PATTERNS[@]}"; do
        echo "  $pattern"
    done

    echo
    process_files

    if ! $DRY_RUN; then
        echo
        validate_changes
    fi

    echo
    if $DRY_RUN; then
        log_info "Dry run completed. Run with --apply to make actual changes."
    else
        log_success "Rename operation completed!"
        log_info "Next steps:"
        echo "  1. Review the changes"
        echo "  2. Run 'cargo clean && cargo build' to rebuild"
        echo "  3. Test the application"
        echo "  4. Update any external references (GitHub repo, etc.)"
    fi
}

# Run main function
main "$@"