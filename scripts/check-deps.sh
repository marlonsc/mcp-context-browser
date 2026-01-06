#!/bin/bash

# MCP Context Browser - Dependency Check Script
# Ensures all required tools are available (no fallbacks)

set -e

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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Required tools check
check_tool() {
    local tool=$1
    local description=$2

    if command -v "$tool" >/dev/null 2>&1; then
        log_success "$tool found: $description"
        return 0
    else
        log_error "$tool not found: $description"
        return 1
    fi
}

# Main dependency check
main() {
    log_info "MCP Context Browser - Dependency Check"
    log_info "====================================="

    local missing_deps=0

    # Core Rust tools
    check_tool "cargo" "Rust package manager" || ((missing_deps++))
    check_tool "rustc" "Rust compiler" || ((missing_deps++))
    check_tool "rustfmt" "Rust code formatter" || ((missing_deps++))

    # Development tools
    check_tool "cargo-watch" "File watcher for development" || ((missing_deps++))
    check_tool "cargo-tarpaulin" "Code coverage tool" || ((missing_deps++))
    check_tool "cargo-audit" "Security audit tool" || ((missing_deps++))

    # Markdown linting (MANDATORY)
    check_tool "npm" "Node.js package manager (required for markdownlint)" || ((missing_deps++))
    if command -v npm >/dev/null 2>&1; then
        check_tool "markdownlint" "Markdown linter (MANDATORY - run 'make setup')" || ((missing_deps++))
    fi

    echo
    if [ $missing_deps -eq 0 ]; then
        log_success "All dependencies satisfied!"
        exit 0
    else
        log_error "Missing $missing_deps dependencies."
        echo
        log_info "To fix missing dependencies:"
        echo "  make setup    # Install development tools"
        echo "  make check-deps # Re-run this check"
        exit 1
    fi
}

# Run main function
main "$@"