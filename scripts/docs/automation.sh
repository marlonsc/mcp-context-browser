#!/bin/bash

# =============================================================================
# Documentation Automation Script - v0.0.4 Documentation Excellence
# =============================================================================
#
# This script orchestrates existing documentation tools for automated
# documentation generation and quality assurance.
#
# Tools used:
# - adrs: Professional ADR management
# - cargo-modules: Module analysis and dependency graphs
# - rust-code-analysis: Advanced code analysis (optional)
# - cargo-spellcheck: Spelling validation
# - cargo-deadlinks: Link validation
# - mdbook: Interactive documentation platform
#
# Usage:
#   ./scripts/docs/automation.sh <command> [options]
#
# Commands:
#   generate     Generate automated documentation
#   validate     Validate documentation quality and ADR compliance
#   quality      Run quality checks on documentation
#   adr-check    Check ADR compliance
#   mdbook       Manage mdbook interactive documentation (init|build|serve|clean)
#   setup        Install and configure all tools
#
# =============================================================================

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DOCS_DIR="${PROJECT_ROOT}/docs"
MODULES_DIR="${DOCS_DIR}/modules"
ADR_DIR="${DOCS_DIR}/adr"

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if command exists (including in cargo bin)
command_exists() {
    command -v "$1" >/dev/null 2>&1 || [ -x "$HOME/.cargo/bin/$1" ]
}

# Ensure mdbook is available
ensure_mdbook() {
    if ! command_exists mdbook; then
        log_error "mdbook not found. Run: ./scripts/docs/automation.sh setup"
        exit 1
    fi
}

# Run mdbook commands
mdbook_command() {
    local subcommand="$1"
    shift

    case "$subcommand" in
        "init")
            log_info "Initializing mdbook documentation..."
            mkdir -p "${PROJECT_ROOT}/docs/book"
            cd "${PROJECT_ROOT}/docs/book"
            mdbook init --title "MCP Context Browser - Documentation Excellence v0.0.4"
            log_success "mdbook initialized"
            ;;
        "build")
            log_info "Building mdbook documentation..."
            cd "${PROJECT_ROOT}/docs/book"
            mdbook build
            log_success "mdbook built successfully"
            ;;
        "serve")
            log_info "Serving mdbook documentation on http://localhost:3000"
            cd "${PROJECT_ROOT}/docs/book"
            mdbook serve --open
            ;;
        "clean")
            log_info "Cleaning mdbook build artifacts..."
            cd "${PROJECT_ROOT}/docs/book"
            rm -rf book/
            log_success "mdbook cleaned"
            ;;
        *)
            echo "Usage: $0 mdbook <init|build|serve|clean>"
            exit 1
            ;;
    esac
}

# Ensure tools are installed
ensure_tools() {
    local missing_tools=()

    if ! command_exists adrs && [ ! -x "$HOME/.cargo/bin/adrs" ]; then
        missing_tools+=("adrs")
    fi

    if ! command_exists cargo-modules && [ ! -x "$HOME/.cargo/bin/cargo-modules" ]; then
        missing_tools+=("cargo-modules")
    fi

    if ! command_exists cargo-spellcheck && [ ! -x "$HOME/.cargo/bin/cargo-spellcheck" ]; then
        missing_tools+=("cargo-spellcheck")
    fi

    if ! command_exists cargo-deadlinks && [ ! -x "$HOME/.cargo/bin/cargo-deadlinks" ]; then
        missing_tools+=("cargo-deadlinks")
    fi

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Run: ./scripts/docs/automation.sh setup"
        exit 1
    fi
}

# Generate automated documentation
generate_docs() {
    log_info "Generating automated documentation..."

    # Create output directories
    mkdir -p "${MODULES_DIR}"

    # Generate module dependencies using cargo-modules
    log_info "Generating module dependencies..."
    local cargo_modules_cmd="cargo-modules"
    if [ -x "$HOME/.cargo/bin/cargo-modules" ]; then
        cargo_modules_cmd="$HOME/.cargo/bin/cargo-modules"
    fi
    if $cargo_modules_cmd dependencies --package mcp-context-browser > "${MODULES_DIR}/dependencies.dot" 2>/dev/null; then
        # Convert DOT to markdown
        cat > "${MODULES_DIR}/dependencies.md" << EOF
# Module Dependencies

This document shows the internal module dependencies of the MCP Context Browser.

## Dependencies Graph

\`\`\`dot
$(cat "${MODULES_DIR}/dependencies.dot")
\`\`\`

## Analysis

The dependency graph above shows how modules are interconnected within the codebase.
Strong dependencies indicate tight coupling that may need refactoring.

*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*
EOF
        log_success "Module dependencies generated"
    else
        log_warning "cargo-modules not available or failed"
    fi

    # Generate module structure
    log_info "Generating module structure..."
    if $cargo_modules_cmd structure --package mcp-context-browser > "${MODULES_DIR}/module-structure.md" 2>/dev/null; then
        # Add header to the file
        sed -i '1i# Module Structure\n\nThis document shows the hierarchical structure of modules in the MCP Context Browser.\n\n## Module Tree\n\n```' "${MODULES_DIR}/module-structure.md"
        echo '```' >> "${MODULES_DIR}/module-structure.md"
        echo -e "\n## Structure Analysis\n\nThe module tree above shows the organization of code into logical units.\n\n*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*" >> "${MODULES_DIR}/module-structure.md"
        log_success "Module structure generated"
    else
        log_warning "Failed to generate module structure"
    fi

    # Generate API surface documentation
    log_info "Generating API surface documentation..."
    cat > "${MODULES_DIR}/api-surface.md" << EOF
# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Public Modules

TODO: List all public modules and their exports

## Public Functions

TODO: List all public functions with signatures

## Public Types

TODO: List all public types and traits

## API Stability

TODO: API stability analysis

*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*
EOF
    log_success "API surface documentation generated"

    # Generate code analysis if available
    log_info "Checking for advanced code analysis..."
    if command_exists rust-code-analysis; then
        log_info "Generating advanced code analysis..."
        # TODO: Implement rust-code-analysis integration
        log_warning "rust-code-analysis integration not yet implemented"
    else
        log_info "rust-code-analysis not available, skipping advanced analysis"
    fi

    # Build interactive documentation if mdbook is available
    build_mdbook

    log_success "Documentation generation completed"
}

# Validate documentation and ADR compliance
validate_docs() {
    log_info "Validating documentation quality and ADR compliance..."

    local has_errors=false

    # Validate ADR compliance
    if [ -f "${PROJECT_ROOT}/.adr-dir" ]; then
        log_info "Validating ADR compliance..."
        local adrs_cmd="adrs"
        if [ -x "$HOME/.cargo/bin/adrs" ]; then
            adrs_cmd="$HOME/.cargo/bin/adrs"
        fi
        if $adrs_cmd list >/dev/null 2>&1; then
            local adr_count=$($adrs_cmd list | wc -l)
            log_success "ADR system functional: ${adr_count} ADRs found"

            # Check ADR files exist and have content
            while IFS= read -r adr_file; do
                if [ -n "$adr_file" ] && [ ! -f "$adr_file" ]; then
                    log_error "ADR file does not exist: $adr_file"
                    has_errors=true
                elif [ -n "$adr_file" ] && [ ! -s "$adr_file" ]; then
                    log_error "ADR file is empty: $adr_file"
                    has_errors=true
                fi
            done < <($adrs_cmd list)
        else
            log_error "ADR system validation failed"
            has_errors=true
        fi
    else
        log_error "ADR configuration not found (.adr-dir)"
        has_errors=true
    fi

    # Validate documentation structure
    log_info "Validating documentation structure..."
    local required_files=("${DOCS_DIR}/README.md" "${ADR_DIR}/README.md")
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_warning "Required documentation file missing: $file"
        fi
    done

    if [ "$has_errors" = true ]; then
        log_error "Validation failed with errors"
        exit 1
    else
        log_success "Validation completed successfully"
    fi
}

# Run quality checks
quality_checks() {
    log_info "Running documentation quality checks..."

    local has_warnings=false

    # Spelling check
    log_info "Checking spelling..."
    local spellcheck_cmd="cargo-spellcheck"
    if [ -x "$HOME/.cargo/bin/cargo-spellcheck" ]; then
        spellcheck_cmd="$HOME/.cargo/bin/cargo-spellcheck"
    fi
    if $spellcheck_cmd --code 0 >/dev/null 2>&1; then
        log_success "No spelling errors found"
    else
        log_warning "Spelling issues found (check output above)"
        has_warnings=true
    fi

    # Link validation
    log_info "Checking dead links..."
    local deadlinks_cmd="cargo-deadlinks"
    if [ -x "$HOME/.cargo/bin/cargo-deadlinks" ]; then
        deadlinks_cmd="$HOME/.cargo/bin/cargo-deadlinks"
    fi
    if $deadlinks_cmd --check-http >/dev/null 2>&1; then
        log_success "No dead links found"
    else
        log_warning "Dead links found (check output above)"
        has_warnings=true
    fi

    # Markdown formatting checks
    log_info "Checking markdown formatting..."
    if [ -d "$DOCS_DIR" ]; then
        # Check for excessive heading levels
        if find "$DOCS_DIR" -name "*.md" -exec grep -l "^####" {} \; | grep -q .; then
            log_warning "Found files with excessive heading levels (####+)"
            has_warnings=true
        fi

        # Check for TODO markers
        if find "$DOCS_DIR" -name "*.md" -exec grep -l "TODO\|FIXME\|XXX" {} \; | grep -q .; then
            log_info "Found files with TODO markers (this is normal)"
        fi
    fi

    if [ "$has_warnings" = true ]; then
        log_warning "Quality checks completed with warnings"
    else
        log_success "Quality checks completed successfully"
    fi
}

# ADR compliance check
adr_check() {
    log_info "Checking ADR compliance..."

    if [ ! -f "${PROJECT_ROOT}/.adr-dir" ]; then
        log_error "ADR configuration not found (.adr-dir)"
        exit 1
    fi

    local adrs_cmd="adrs"
    if [ -x "$HOME/.cargo/bin/adrs" ]; then
        adrs_cmd="$HOME/.cargo/bin/adrs"
    fi

    if ! $adrs_cmd list >/dev/null 2>&1; then
        log_error "ADR system not functional"
        exit 1
    fi

    local adr_count=$($adrs_cmd list | wc -l)
    log_success "ADR system functional: ${adr_count} ADRs found"

    # Generate ADR summary
    log_info "Generating ADR summary..."
    $adrs_cmd generate toc > "${ADR_DIR}/README.md" 2>/dev/null || true

    log_success "ADR compliance check completed"
}

# Build interactive documentation with mdbook
build_mdbook() {
    log_info "Building interactive documentation with mdbook..."

    local mdbook_cmd="mdbook"
    if [ -x "$HOME/.cargo/bin/mdbook" ]; then
        mdbook_cmd="$HOME/.cargo/bin/mdbook"
    fi

    if ! command_exists mdbook && [ ! -x "$HOME/.cargo/bin/mdbook" ]; then
        log_warning "mdbook not found, skipping interactive docs build"
        return
    fi

    # Update mdbook content
    "${SCRIPT_DIR}/generate-mdbook.sh" update

    # Build mdbook
    cd "${PROJECT_ROOT}/book"
    if $mdbook_cmd build; then
        log_success "Interactive documentation built successfully"
        log_info "Documentation available at: ${PROJECT_ROOT}/book/docs/index.html"
    else
        log_warning "Failed to build interactive documentation"
    fi
}

# Setup and install tools
setup_tools() {
    log_info "Setting up documentation automation tools..."

    # Install ADR tools
    log_info "Installing ADR tools..."
    if cargo install adrs; then
        log_success "adrs installed successfully"
    else
        log_error "Failed to install adrs"
        exit 1
    fi

    # Install cargo-modules
    log_info "Installing cargo-modules..."
    if cargo install cargo-modules; then
        log_success "cargo-modules installed successfully"
    else
        log_error "Failed to install cargo-modules"
        exit 1
    fi

    # Install cargo-spellcheck
    log_info "Installing cargo-spellcheck..."
    if cargo install cargo-spellcheck; then
        log_success "cargo-spellcheck installed successfully"
    else
        log_error "Failed to install cargo-spellcheck"
        exit 1
    fi

    # Install cargo-deadlinks
    log_info "Installing cargo-deadlinks..."
    if cargo install cargo-deadlinks; then
        log_success "cargo-deadlinks installed successfully"
    else
        log_error "Failed to install cargo-deadlinks"
        exit 1
    fi

    # Install mdbook
    log_info "Installing mdbook..."
    if cargo install mdbook; then
        log_success "mdbook installed successfully"
    else
        log_error "Failed to install mdbook"
        exit 1
    fi

    # Install mdbook
    log_info "Installing mdbook..."
    if cargo install mdbook; then
        log_success "mdbook installed successfully"
    else
        log_error "Failed to install mdbook"
        exit 1
    fi

    # Check for optional tools
    if command_exists rust-code-analysis; then
        log_success "rust-code-analysis is available"
    else
        log_info "rust-code-analysis not found (optional advanced analysis)"
    fi

    # Initialize ADR system if needed
    if [ ! -f "${PROJECT_ROOT}/.adr-dir" ]; then
        log_info "Initializing ADR system..."
        cd "${PROJECT_ROOT}"
        local adrs_cmd="adrs"
        if [ -x "$HOME/.cargo/bin/adrs" ]; then
            adrs_cmd="$HOME/.cargo/bin/adrs"
        fi
        if $adrs_cmd init >/dev/null 2>&1; then
            echo "docs/adr" > .adr-dir
            log_success "ADR system initialized"
        else
            log_warning "ADR system initialization failed"
        fi
    fi

    log_success "Documentation tools setup completed"
    log_info "Available commands:"
    log_info "  ./scripts/docs/automation.sh generate    - Generate documentation"
    log_info "  ./scripts/docs/automation.sh validate    - Validate docs and ADRs"
    log_info "  ./scripts/docs/automation.sh quality     - Run quality checks"
    log_info "  ./scripts/docs/automation.sh adr-check   - Check ADR compliance"
}

# Main command dispatcher
main() {
    local command="$1"
    shift

    case "$command" in
        "generate")
            ensure_tools
            generate_docs "$@"
            ;;
        "validate")
            ensure_tools
            validate_docs "$@"
            ;;
        "quality")
            ensure_tools
            quality_checks "$@"
            ;;
        "adr-check")
            ensure_tools
            adr_check "$@"
            ;;
        "mdbook")
            ensure_mdbook
            mdbook_command "$@"
            ;;
        "setup")
            setup_tools "$@"
            ;;
        *)
            echo "Usage: $0 <command> [options]"
            echo ""
            echo "Commands:"
            echo "  generate     Generate automated documentation"
            echo "  validate     Validate documentation quality and ADR compliance"
            echo "  quality      Run quality checks on documentation"
            echo "  adr-check    Check ADR compliance"
            echo "  setup        Install and configure all tools"
            echo ""
            echo "Examples:"
            echo "  $0 setup"
            echo "  $0 generate"
            echo "  $0 validate"
            echo "  $0 quality"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"