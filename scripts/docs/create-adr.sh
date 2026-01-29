#!/bin/bash
# =============================================================================
# MCP Context Browser - ADR Creation Script
# =============================================================================
# Creates new Architecture Decision Records interactively or in batch mode
# =============================================================================

set -e

# Source shared library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Get next ADR number
get_next_adr_number() {
    local last_file
    last_file=$(find "$ADR_DIR" -maxdepth 1 -name '[0-9][0-9][0-9]-*.md' 2>/dev/null | sort -V | tail -1)
    local existing_adrs=0
    [ -n "$last_file" ] && existing_adrs=$(basename "$last_file" | grep -oE '^[0-9]+' | sed 's/^0*//')
    [ -z "$existing_adrs" ] && existing_adrs=0
    echo $((existing_adrs + 1))
}

# Create ADR filename from title
create_filename() {
    local title="$1"
    # Convert to lowercase, replace spaces/special chars with hyphens
    echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-\|-$//g'
}

# Interactive ADR creation
create_adr_interactive() {
    local template_file="$PROJECT_ROOT/docs/templates/adr-template.md"

    # Check if template exists
    if ! check_file "$template_file" "ADR template"; then
        exit 1
    fi

    # Get ADR details
    echo "Creating new Architecture Decision Record"
    echo "========================================"
    echo

    read -r -p "ADR Title: " adr_title
    if [[ -z "$adr_title" ]]; then
        log_error "ADR title cannot be empty"
        exit 1
    fi

    read -r -p "Status (Proposed/Accepted/Rejected/Deprecated/Superseded by ADR-xxx) [Proposed]: " adr_status
    adr_status=${adr_status:-Proposed}

    create_adr_file "$adr_title" "$adr_status"
}

# Create the ADR file
create_adr_file() {
    local adr_title="$1"
    local adr_status="${2:-Proposed}"
    local template_file="$PROJECT_ROOT/docs/templates/adr-template.md"

    # Generate filename
    local adr_num
    adr_num=$(printf "%03d" "$(get_next_adr_number)")
    local filename_slug
    filename_slug=$(create_filename "$adr_title")
    local adr_filename="${adr_num}-${filename_slug}.md"
    local adr_filepath="$ADR_DIR/$adr_filename"

    # Check if file already exists
    if [[ -f "$adr_filepath" ]]; then
        log_error "ADR file already exists: $adr_filepath"
        exit 1
    fi

    log_info "Creating ADR: $adr_filename"
    log_info "Title: $adr_title"
    log_info "Status: $adr_status"
    echo

    # Copy template and update
    run_or_echo cp "$template_file" "$adr_filepath"

    if ! is_dry_run; then
        # Update ADR number and title
        sed -i "s/{number}/$adr_num/g" "$adr_filepath"
        sed -i "s/{title}/$adr_title/g" "$adr_filepath"
        sed -i "s/{Proposed | Accepted | Rejected | Deprecated | Superseded by ADR-xxx}/$adr_status/g" "$adr_filepath"
    fi

    log_success "ADR created: $adr_filepath"
    echo
    log_info "Next steps:"
    echo "1. Edit the ADR file to add context, decision, and consequences"
    echo "2. Run 'make adr-check' to validate the ADR format"
    echo "3. Add the ADR to the architecture documentation if applicable"
}

# Batch ADR creation (for automation)
create_adr_batch() {
    local title="$1"
    local status="${2:-Proposed}"

    if [[ -z "$title" ]]; then
        log_error "ADR title is required for batch creation"
        echo "Usage: $0 batch \"ADR Title\" [status]"
        exit 1
    fi

    create_adr_file "$title" "$status"
}

# Show usage
show_usage() {
    cat << EOF
MCP Context Browser - ADR Creation Tool

USAGE:
    $0                           # Interactive mode
    $0 batch "title" [status]    # Batch mode
    $0 --dry-run batch "title"   # Dry-run mode

ARGUMENTS:
    title     ADR title (required in batch mode)
    status    ADR status: Proposed, Accepted, Rejected, Deprecated, or "Superseded by ADR-xxx"
             (default: Proposed)

EXAMPLES:
    $0                                        # Interactive creation
    $0 batch "New Feature Decision"           # Create with default status
    $0 batch "Security Enhancement" Accepted  # Create with specific status

EOF
}

# Main execution
main() {
    # Check for dry-run flag (exported for use in create_adr_file / create_adr_batch)
    if [[ "${1:-}" == "--dry-run" ]]; then
        export DRY_RUN=true
        shift
    fi

    local command="${1:-interactive}"

    log_info "MCP Context Browser - ADR Creation Tool"

    case "$command" in
        "batch")
            shift
            create_adr_batch "$@"
            ;;
        "help"|"--help"|"-h")
            show_usage
            ;;
        "interactive")
            create_adr_interactive
            ;;
        *)
            show_usage
            exit 1
            ;;
    esac
}

main "$@"
