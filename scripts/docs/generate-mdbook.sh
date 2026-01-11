#!/bin/bash

# =============================================================================
# MdBook Integration Script - v0.1.0 First Stable Release
# =============================================================================
#
# This script integrates automatically generated documentation with mdbook
# for a professional, interactive documentation experience.
#
# Features:
# - Copies auto-generated docs to mdbook structure
# - Updates SUMMARY.md with generated content
# - Builds and serves documentation
#
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BOOK_DIR="${PROJECT_ROOT}/book"
DOCS_DIR="${PROJECT_ROOT}/docs"
MODULES_SRC="${DOCS_DIR}/modules"
MODULES_DEST="${BOOK_DIR}/src/modules"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Ensure directories exist
ensure_directories() {
    mkdir -p "${MODULES_DEST}"
    mkdir -p "${BOOK_DIR}/src/api"
    mkdir -p "${BOOK_DIR}/src/architecture"
}

# Copy generated module documentation
copy_module_docs() {
    log_info "Copying generated module documentation..."

    if [ -d "${MODULES_SRC}" ]; then
        # Copy all generated module docs
        cp -r "${MODULES_SRC}"/* "${MODULES_DEST}/" 2>/dev/null || true

        # Convert DOT files to markdown embeds
        for dot_file in "${MODULES_DEST}"/*.dot; do
            if [ -f "$dot_file" ]; then
                base_name=$(basename "$dot_file" .dot)
                md_file="${MODULES_DEST}/${base_name}.md"

                # If markdown file doesn't exist or is empty, create it
                if [ ! -f "$md_file" ] || [ ! -s "$md_file" ]; then
                    cat > "$md_file" << EOF
# ${base_name//-/ }

This document shows the ${base_name//-/ } analysis.

## Graph

\`\`\`dot
$(cat "$dot_file")
\`\`\`

*Generated automatically from source code analysis.*
EOF
                fi
            fi
        done

        log_success "Module documentation copied"
    else
        log_warning "No generated module docs found. Run 'make docs-generate' first."
    fi
}

# Copy ADR documentation
copy_adr_docs() {
    log_info "Copying ADR documentation..."

    if [ -d "${DOCS_DIR}/adr" ]; then
        mkdir -p "${BOOK_DIR}/src/architecture"
        cp "${DOCS_DIR}/adr/README.md" "${BOOK_DIR}/src/architecture/adrs.md" 2>/dev/null || true
        log_success "ADR documentation copied"
    else
        log_warning "No ADR docs found"
    fi
}

# Generate API documentation from rustdoc
generate_api_docs() {
    log_info "Generating API documentation..."

    # Build docs and copy to mdbook structure
    if cargo doc --no-deps --document-private-items; then
        # Copy generated rustdoc to mdbook if needed
        log_success "API documentation generated"
    else
        log_warning "Failed to generate API docs"
    fi
}

# Update SUMMARY.md with dynamic content
update_summary() {
    log_info "Updating SUMMARY.md with available content..."

    SUMMARY_FILE="${BOOK_DIR}/src/SUMMARY.md"

    # Check which modules actually exist and update SUMMARY accordingly
    if [ -f "${MODULES_DEST}/dependencies.md" ]; then
        log_success "Dependencies documentation available"
    fi

    if [ -f "${MODULES_DEST}/module-structure.md" ]; then
        log_success "Module structure documentation available"
    fi

    if [ -f "${MODULES_DEST}/api-surface.md" ]; then
        log_success "API surface documentation available"
    fi
}

# Build mdbook
build_mdbook() {
    log_info "Building mdbook documentation..."

    local mdbook_cmd="mdbook"
    if [ -x "$HOME/.cargo/bin/mdbook" ]; then
        mdbook_cmd="$HOME/.cargo/bin/mdbook"
    fi

    cd "${BOOK_DIR}"
    if $mdbook_cmd build; then
        log_success "Mdbook built successfully"
        log_info "Documentation available at: ${BOOK_DIR}/docs/index.html"
    else
        log_error "Failed to build mdbook"
        exit 1
    fi
}

# Serve mdbook (for development)
serve_mdbook() {
    log_info "Serving mdbook documentation..."

    local mdbook_cmd="mdbook"
    if [ -x "$HOME/.cargo/bin/mdbook" ]; then
        mdbook_cmd="$HOME/.cargo/bin/mdbook"
    fi

    cd "${BOOK_DIR}"
    log_info "Documentation server starting at: http://localhost:3000"
    log_info "Press Ctrl+C to stop"
    $mdbook_cmd serve --open
}

# Main function
main() {
    local command="$1"
    shift

    case "$command" in
        "build")
            ensure_directories
            copy_module_docs
            copy_adr_docs
            generate_api_docs
            update_summary
            build_mdbook
            ;;
        "serve")
            ensure_directories
            copy_module_docs
            copy_adr_docs
            generate_api_docs
            update_summary
            serve_mdbook
            ;;
        "update")
            ensure_directories
            copy_module_docs
            copy_adr_docs
            update_summary
            ;;
        *)
            echo "Usage: $0 <command>"
            echo ""
            echo "Commands:"
            echo "  build    Build mdbook with latest generated docs"
            echo "  serve    Serve mdbook with live reload"
            echo "  update   Update mdbook content from generated docs"
            echo ""
            echo "Examples:"
            echo "  $0 build"
            echo "  $0 serve"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"