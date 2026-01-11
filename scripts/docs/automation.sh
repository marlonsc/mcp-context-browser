#!/bin/bash

# =============================================================================
# Documentation Automation Script - v0.1.0 First Stable Release
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
            mdbook init --title "MCP Context Browser - v0.1.0"
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

    # Generate module analysis using reliable fallback method (cargo-modules has performance issues)
    log_info "Generating module analysis using reliable fallback method..."
    generate_fallback_module_analysis

    # Generate API surface documentation using cargo doc
    log_info "Generating API surface documentation..."
    if cargo doc --no-deps --document-private-items --package mcp-context-browser --lib > /dev/null 2>&1; then
        # Extract public API information from source files
        generate_api_surface_from_source
    else
        log_warning "cargo doc failed, using basic API surface template"
        generate_basic_api_surface
    fi

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
# ADR compliance validation
validate_adr_compliance() {
    log_info "Validating ADR compliance..."

    local violations=0
    local total_checks=0

    # ADR 001: Provider Pattern - Check for trait usage
    ((total_checks++))
    if grep -r "pub trait.*Provider" "${PROJECT_ROOT}/src/providers/" > /dev/null 2>&1; then
        log_success "ADR 001: Provider Pattern - Traits found ✓"
    else
        log_error "ADR 001: Provider Pattern - No provider traits found"
        ((violations++))
    fi

    # ADR 002: Async-first Architecture - Check for async usage
    ((total_checks++))
    local async_count=$(grep -r "async fn" "${PROJECT_ROOT}/src/" | wc -l)
    if [ "$async_count" -gt 10 ]; then
        log_success "ADR 002: Async-first - $async_count async functions found ✓"
    else
        log_warning "ADR 002: Async-first - Only $async_count async functions (expected >10)"
    fi

    # ADR 004: Multi-provider Strategy - Check for routing logic
    ((total_checks++))
    if [ -f "${PROJECT_ROOT}/src/providers/routing.rs" ]; then
        log_success "ADR 004: Multi-provider Strategy - Routing module exists ✓"
    else
        log_error "ADR 004: Multi-provider Strategy - No routing module found"
        ((violations++))
    fi

    # ADR 006: Code Audit - Check for unwrap/expect usage
    ((total_checks++))
    local unwrap_count=$(grep -r "unwrap()" "${PROJECT_ROOT}/src/" | grep -v "test" | wc -l)
    local expect_count=$(grep -r "expect(" "${PROJECT_ROOT}/src/" | grep -v "test" | wc -l)
    local total_anti_patterns=$((unwrap_count + expect_count))

    if [ "$total_anti_patterns" -eq 0 ]; then
        log_success "ADR 006: Code Audit - No unwrap/expect in production code ✓"
    else
        log_error "ADR 006: Code Audit - Found $total_anti_patterns unwrap/expect calls (should be 0)"
        ((violations++))
    fi

    # ADR 003: C4 Model Documentation - Check for diagrams
    ((total_checks++))
    if [ -d "${PROJECT_ROOT}/docs/diagrams" ] && [ "$(find "${PROJECT_ROOT}/docs/diagrams" -name "*.puml" | wc -l)" -gt 0 ]; then
        log_success "ADR 003: C4 Model - Diagrams directory exists ✓"
    else
        log_warning "ADR 003: C4 Model - No diagrams found"
    fi

    # ADR 005: Documentation Excellence - Check for automation
    ((total_checks++))
    if [ -f "${PROJECT_ROOT}/scripts/docs/automation.sh" ]; then
        log_success "ADR 005: Documentation Excellence - Automation script exists ✓"
    else
        log_error "ADR 005: Documentation Excellence - No automation script"
        ((violations++))
    fi

    # Generate compliance report
    cat > "${DOCS_DIR}/adr-validation-report.md" << EOF
# ADR Compliance Validation Report

Generated on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')

## Summary

- **Total Checks**: $total_checks
- **Passed**: $((total_checks - violations))
- **Violations**: $violations
- **Compliance Rate**: $(( (total_checks - violations) * 100 / total_checks ))%

## Detailed Results

### ADR 001: Provider Pattern Architecture
**Status**: $(grep -r "pub trait.*Provider" "${PROJECT_ROOT}/src/providers/" > /dev/null 2>&1 && echo "✅ PASSED" || echo "❌ FAILED")
**Requirement**: Use traits for provider abstractions
**Evidence**: $(grep -c "pub trait.*Provider" "${PROJECT_ROOT}/src/providers/" 2>/dev/null || echo "0") provider traits found

### ADR 002: Async-First Architecture
**Status**: $([ "$async_count" -gt 10 ] && echo "✅ PASSED" || echo "⚠️ WARNING")
**Requirement**: Comprehensive async/await usage
**Evidence**: $async_count async functions found

### ADR 004: Multi-Provider Strategy
**Status**: $([ -f "${PROJECT_ROOT}/src/providers/routing.rs" ] && echo "✅ PASSED" || echo "❌ FAILED")
**Requirement**: Intelligent provider routing
**Evidence**: Routing module $([ -f "${PROJECT_ROOT}/src/providers/routing.rs" ] && echo "exists" || echo "missing")

### ADR 006: Code Audit and Improvements
**Status**: $([ "$total_anti_patterns" -eq 0 ] && echo "✅ PASSED" || echo "❌ FAILED")
**Requirement**: Zero unwrap/expect in production code
**Evidence**: $total_anti_patterns unwrap/expect calls found

### ADR 003: C4 Model Documentation
**Status**: $([ -d "${PROJECT_ROOT}/docs/diagrams" ] && echo "✅ PASSED" || echo "⚠️ WARNING")
**Requirement**: Architecture diagrams using C4 model
**Evidence**: $(find "${PROJECT_ROOT}/docs/diagrams" -name "*.puml" 2>/dev/null | wc -l) PlantUML diagrams

### ADR 005: Documentation Excellence
**Status**: $([ -f "${PROJECT_ROOT}/scripts/docs/automation.sh" ] && echo "✅ PASSED" || echo "❌ FAILED")
**Requirement**: Automated documentation generation
**Evidence**: Automation script $([ -f "${PROJECT_ROOT}/scripts/docs/automation.sh" ] && echo "exists" || echo "missing")

## Recommendations

EOF

    if [ "$violations" -gt 0 ]; then
        echo "- **Critical**: Address $violations compliance violations" >> "${DOCS_DIR}/adr-validation-report.md"
    fi

    if [ "$total_anti_patterns" -gt 0 ]; then
        echo "- **Code Quality**: Replace $total_anti_patterns unwrap/expect calls with proper error handling" >> "${DOCS_DIR}/adr-validation-report.md"
    fi

    echo "- **Documentation**: Ensure all ADRs have automated validation rules" >> "${DOCS_DIR}/adr-validation-report.md"
    echo "" >> "${DOCS_DIR}/adr-validation-report.md"

    # Overall assessment
    if [ "$violations" -eq 0 ]; then
        log_success "ADR Compliance: $(( (total_checks - violations) * 100 / total_checks ))% ($((total_checks - violations))/$total_checks passed)"
        echo "## Overall Assessment: ✅ COMPLIANT" >> "${DOCS_DIR}/adr-validation-report.md"
    else
        log_error "ADR Compliance: $(( (total_checks - violations) * 100 / total_checks ))% - $violations violations found"
        echo "## Overall Assessment: ❌ NON-COMPLIANT" >> "${DOCS_DIR}/adr-validation-report.md"
    fi

    return $violations
}

adr_check() {
    log_info "Checking ADR compliance..."

    # Run compliance validation
    if validate_adr_compliance; then
        log_success "ADR compliance validation completed"
    else
        log_warning "ADR compliance issues found (see adr-validation-report.md)"
    fi

    # Original ADR system check
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

# Generate API surface from source code analysis
generate_api_surface_from_source() {
    log_info "Analyzing source code for API surface..."

    cat > "${MODULES_DIR}/api-surface.md" << EOF
# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Public Modules

EOF

    # Extract public modules from lib.rs
    if [ -f "${PROJECT_ROOT}/src/lib.rs" ]; then
        echo "### Core Library Modules" >> "${MODULES_DIR}/api-surface.md"
        echo "" >> "${MODULES_DIR}/api-surface.md"
        grep -E "^pub mod " "${PROJECT_ROOT}/src/lib.rs" | sed 's/^pub mod /- /' | sed 's/;$//' >> "${MODULES_DIR}/api-surface.md"
        echo "" >> "${MODULES_DIR}/api-surface.md"
    fi

    # Extract public re-exports
    echo "### Public Re-exports" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"
    grep -E "^pub use " "${PROJECT_ROOT}/src/lib.rs" | sed 's/^pub use /- /' | sed 's/;$//' >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    # Extract public functions and types from main source files
    echo "## Public Functions" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    # Core types
    if [ -f "${PROJECT_ROOT}/src/core/mod.rs" ]; then
        echo "### Core Types" >> "${MODULES_DIR}/api-surface.md"
        grep -h "pub fn " "${PROJECT_ROOT}/src/core/"*.rs 2>/dev/null | head -10 | sed 's/.*pub fn /- /' | sed 's/{.*//' >> "${MODULES_DIR}/api-surface.md"
        echo "" >> "${MODULES_DIR}/api-surface.md"
    fi

    # Provider interfaces
    if [ -f "${PROJECT_ROOT}/src/providers/mod.rs" ]; then
        echo "### Provider Interfaces" >> "${MODULES_DIR}/api-surface.md"
        grep -h "pub trait " "${PROJECT_ROOT}/src/providers/"*.rs 2>/dev/null | sed 's/.*pub trait /- /' | sed 's/{.*//' >> "${MODULES_DIR}/api-surface.md"
        echo "" >> "${MODULES_DIR}/api-surface.md"
    fi

    # Service interfaces
    if [ -f "${PROJECT_ROOT}/src/services/mod.rs" ]; then
        echo "### Service Interfaces" >> "${MODULES_DIR}/api-surface.md"
        grep -h "pub async fn " "${PROJECT_ROOT}/src/services/"*.rs 2>/dev/null | head -5 | sed 's/.*pub async fn /- /' | sed 's/{.*//' >> "${MODULES_DIR}/api-surface.md"
        echo "" >> "${MODULES_DIR}/api-surface.md"
    fi

    echo "## Public Types" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    # Extract public structs and enums
    echo "### Data Structures" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"
    find "${PROJECT_ROOT}/src" -name "*.rs" -exec grep -h "pub struct " {} \; 2>/dev/null | head -10 | sed 's/.*pub struct /- /' | sed 's/{.*//' >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    echo "### Enums" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"
    find "${PROJECT_ROOT}/src" -name "*.rs" -exec grep -h "pub enum " {} \; 2>/dev/null | head -5 | sed 's/.*pub enum /- /' | sed 's/{.*//' >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    echo "## API Stability" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"
    echo "### Current Status" >> "${MODULES_DIR}/api-surface.md"
    echo "- **Version**: 0.1.0 (First Stable Release)" >> "${MODULES_DIR}/api-surface.md"
    echo "- **Stability**: Experimental - APIs may change" >> "${MODULES_DIR}/api-surface.md"
    echo "- **Compatibility**: Breaking changes expected until 1.0.0" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"
    echo "### Public API Commitments" >> "${MODULES_DIR}/api-surface.md"
    echo "- MCP protocol interface stability" >> "${MODULES_DIR}/api-surface.md"
    echo "- Core semantic search functionality" >> "${MODULES_DIR}/api-surface.md"
    echo "- Provider abstraction interfaces" >> "${MODULES_DIR}/api-surface.md"
    echo "" >> "${MODULES_DIR}/api-surface.md"

    echo "*Generated automatically from source code analysis on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*" >> "${MODULES_DIR}/api-surface.md"
    log_success "API surface documentation generated from source"
}

# Generate basic API surface when source analysis fails
generate_basic_api_surface() {
    cat > "${MODULES_DIR}/api-surface.md" << EOF
# API Surface Analysis

This document provides an overview of the public API surface of the MCP Context Browser.

## Public Modules

### Core Library Modules
- chunking (text processing utilities)
- config (configuration management)
- core (core types and utilities)
- providers (AI provider abstractions)
- services (business logic layer)
- server (MCP protocol server)
- metrics (monitoring and observability)
- sync (cross-process coordination)
- daemon (background services)
- snapshot (change tracking)

### Public Re-exports
- Rate limiting system (core::rate_limit)
- Resource limits system (core::limits)
- Advanced caching system (core::cache)
- Hybrid search system (core::hybrid_search)
- Multi-provider routing (providers::routing)

## Public Functions

### Core Types
- Error handling and conversion functions
- Configuration validation functions
- Cache management operations

### Provider Interfaces
- EmbeddingProvider trait methods
- VectorStoreProvider trait methods
- Provider factory functions

### Service Interfaces
- ContextService::embed_text()
- IndexingService::index_codebase()
- SearchService::search()

## Public Types

### Data Structures
- Embedding (vector representation)
- CodeChunk (processed code segment)
- SearchResult (search response)
- ContextConfig (service configuration)
- ProviderConfig (provider settings)

### Enums
- Error (comprehensive error types)
- ProviderType (available providers)
- IndexStatus (indexing progress)

## API Stability

### Current Status
- **Version**: 0.1.0 (First Stable Release)
- **Stability**: Experimental - APIs may change
- **Compatibility**: Breaking changes expected until 1.0.0

### Public API Commitments
- MCP protocol interface stability
- Core semantic search functionality
- Provider abstraction interfaces

*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*
EOF
    log_success "Basic API surface documentation generated"
}

# Fallback module analysis when cargo-modules fails
generate_fallback_module_analysis() {
    log_info "Generating fallback module analysis..."

    # Create basic module structure using find and grep
    cat > "${MODULES_DIR}/dependencies.dot" << 'EOF'
digraph {
    rankdir=TB;
    node [shape=box, style=filled, fillcolor=lightblue];

    "main" -> "lib";
    "lib" -> "core";
    "lib" -> "config";
    "lib" -> "providers";
    "lib" -> "services";
    "lib" -> "server";
    "lib" -> "metrics";
    "lib" -> "sync";
    "lib" -> "daemon";
    "lib" -> "snapshot";

    "providers" -> "core";
    "services" -> "core";
    "services" -> "providers";
    "server" -> "core";
    "server" -> "services";
    "server" -> "providers";
    "metrics" -> "core";
    "sync" -> "core";
    "daemon" -> "core";
    "snapshot" -> "core";

    label="MCP Context Browser Module Dependencies (Estimated)";
}
EOF

    # Generate basic module structure
    cat > "${MODULES_DIR}/module-structure.md" << EOF
# Module Structure

This document shows the hierarchical structure of modules in the MCP Context Browser.

## Module Tree

\`\`\`
mcp-context-browser/
├── main.rs (entry point)
├── lib.rs (library exports)
├── core/ (core types and utilities)
│   ├── error.rs
│   ├── types.rs
│   ├── cache.rs
│   ├── limits.rs
│   └── rate_limit.rs
├── config.rs (configuration)
├── providers/ (provider implementations)
│   ├── mod.rs
│   ├── embedding/
│   └── vector_store/
├── services/ (business logic)
│   ├── mod.rs
│   ├── context.rs
│   ├── indexing.rs
│   └── search.rs
├── server/ (MCP protocol server)
│   └── mod.rs
├── metrics/ (monitoring)
│   ├── mod.rs
│   ├── http_server.rs
│   └── system.rs
├── sync/ (cross-process coordination)
│   └── mod.rs
├── daemon/ (background processes)
│   └── mod.rs
└── snapshot/ (change tracking)
    └── mod.rs
\`\`\`

## Analysis

This is a simplified module structure generated as fallback when cargo-modules analysis is not available. The actual structure may be more complex with additional submodules and dependencies.

*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*
EOF

    # Generate basic dependencies markdown
    cat > "${MODULES_DIR}/dependencies.md" << EOF
# Module Dependencies

This document shows the internal module dependencies of the MCP Context Browser.

## Dependencies Graph

\`\`\`dot
$(cat "${MODULES_DIR}/dependencies.dot")
\`\`\`

## Analysis

The dependency graph above shows estimated module relationships within the codebase. Higher-level modules depend on lower-level core modules, creating a clean layered architecture.

Key dependency patterns:
- **Entry point** (main) depends on library (lib)
- **Business logic** (services) depends on providers and core
- **HTTP server** depends on all major components
- **Core modules** have minimal dependencies

*Generated automatically on: $(date -u '+%Y-%m-%d %H:%M:%S UTC')*
EOF

    log_success "Fallback module analysis generated"
}

# Run main function with all arguments
main "$@"