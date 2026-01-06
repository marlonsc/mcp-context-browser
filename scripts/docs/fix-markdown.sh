#!/bin/bash

# MCP Context Browser - Markdown Auto-Fix Script
# Automatically fixes common markdown linting issues

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Counters
fixed=0

# Fix trailing whitespace
fix_trailing_whitespace() {
    log_info "Fixing trailing whitespace..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        if grep -q '[[:space:]]$' "$file"; then
            log_info "Fixing trailing whitespace in: $(basename "$file")"
            sed -i 's/[[:space:]]*$//' "$file"
            ((fixed++))
        fi
    done

    log_success "Trailing whitespace fixed"
}

# Fix multiple consecutive blank lines
fix_multiple_blank_lines() {
    log_info "Fixing multiple consecutive blank lines..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        if grep -q '\n\n\n' "$file"; then
            log_info "Fixing multiple blank lines in: $(basename "$file")"
            # Replace 3+ consecutive newlines with 2 newlines
            awk 'BEGIN{RS=""} {gsub(/\n\n+/,"\n\n"); print $0 "\n"}' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
            ((fixed++))
        fi
    done

    log_success "Multiple blank lines fixed"
}

# Fix unordered list consistency (use dashes)
fix_unordered_lists() {
    log_info "Fixing unordered list consistency..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Replace asterisks with dashes for consistency
        if grep -q '^[[:space:]]*\*[[:space:]]' "$file"; then
            log_info "Converting asterisks to dashes in: $(basename "$file")"
            sed -i 's/^[[:space:]]*\*[[:space:]]/  - /g' "$file"
            ((fixed++))
        fi
    done

    log_success "Unordered list consistency fixed"
}

# Fix code block language tags (add missing ones)
fix_code_blocks() {
    log_info "Checking code block language tags..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Find fenced code blocks without language tags
        if grep -q '^```$' "$file"; then
            log_warning "Found code blocks without language tags in: $(basename "$file")"
            log_warning "Manual review recommended for proper language tagging"
        fi
    done

    log_success "Code block check completed"
}

# Fix header spacing
fix_header_spacing() {
    log_info "Fixing header spacing..."

    local files=$(find "$PROJECT_ROOT/docs" -name "*.md" -type f)

    for file in $files; do
        # Ensure there's a blank line after headers
        if grep -q '^#.*' "$file"; then
            # This is complex to do with sed, so we'll just check and warn
            log_info "Header spacing check for: $(basename "$file") (manual review recommended)"
        fi
    done

    log_success "Header spacing check completed"
}

# Main execution
main() {
    log_info "MCP Context Browser - Markdown Auto-Fix"
    log_info "======================================="

    fix_trailing_whitespace
    fix_multiple_blank_lines
    fix_unordered_lists
    fix_code_blocks
    fix_header_spacing

    echo
    log_info "Auto-fix Summary:"
    echo "  Issues fixed: $fixed"

    if [ $fixed -gt 0 ]; then
        log_success "Auto-fix completed. Run 'make lint-md' to verify."
    else
        log_success "No auto-fixable issues found."
    fi
}

# Run main function
main "$@"