#!/bin/bash

# =============================================================================
# MdBook Sync Script - Syncs docs/ content to book/src/ for mdbook
# =============================================================================
# This script creates symlinks or copies documentation files from docs/
# to the mdbook source directory, keeping them synchronized.
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DOCS_DIR="${PROJECT_ROOT}/docs"
BOOK_SRC="${PROJECT_ROOT}/book/src"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# Create required directories
create_directories() {
    log_info "Creating mdbook directory structure..."
    mkdir -p "${BOOK_SRC}/adr"
    mkdir -p "${BOOK_SRC}/architecture"
    mkdir -p "${BOOK_SRC}/api"
    mkdir -p "${BOOK_SRC}/modules"
    mkdir -p "${BOOK_SRC}/operations"
    mkdir -p "${BOOK_SRC}/developer"
    mkdir -p "${BOOK_SRC}/user-guide"
}

# Sync ADRs
sync_adrs() {
    log_info "Syncing ADRs..."

    # Copy ADR files (not symlink to avoid mdbook issues)
    for adr in "${DOCS_DIR}/adr"/*.md; do
        if [ -f "$adr" ]; then
            cp "$adr" "${BOOK_SRC}/adr/"
        fi
    done

    log_success "ADRs synced"
}

# Sync architecture docs
sync_architecture() {
    log_info "Syncing architecture docs..."

    if [ -f "${DOCS_DIR}/architecture/ARCHITECTURE.md" ]; then
        cp "${DOCS_DIR}/architecture/ARCHITECTURE.md" "${BOOK_SRC}/architecture/"
    fi

    log_success "Architecture docs synced"
}

# Sync operations docs
sync_operations() {
    log_info "Syncing operations docs..."

    for doc in DEPLOYMENT.md CHANGELOG.md MONITORING.md; do
        if [ -f "${DOCS_DIR}/operations/${doc}" ]; then
            cp "${DOCS_DIR}/operations/${doc}" "${BOOK_SRC}/operations/"
        fi
    done

    log_success "Operations docs synced"
}

# Sync developer docs
sync_developer() {
    log_info "Syncing developer docs..."

    for doc in CONTRIBUTING.md ROADMAP.md TESTING.md; do
        if [ -f "${DOCS_DIR}/developer/${doc}" ]; then
            cp "${DOCS_DIR}/developer/${doc}" "${BOOK_SRC}/developer/"
        fi
    done

    log_success "Developer docs synced"
}

# Sync user guide
sync_user_guide() {
    log_info "Syncing user guide..."

    if [ -d "${DOCS_DIR}/user-guide" ]; then
        cp "${DOCS_DIR}/user-guide"/*.md "${BOOK_SRC}/user-guide/" 2>/dev/null || true
    fi

    log_success "User guide synced"
}

# Sync modules documentation
sync_modules() {
    log_info "Syncing module docs..."

    if [ -d "${DOCS_DIR}/modules" ]; then
        cp "${DOCS_DIR}/modules"/*.md "${BOOK_SRC}/modules/" 2>/dev/null || true
    fi

    log_success "Module docs synced"
}

# Create README.md if it doesn't exist
create_readme() {
    if [ ! -f "${BOOK_SRC}/README.md" ]; then
        log_info "Creating README.md..."

        cat > "${BOOK_SRC}/README.md" << 'EOF'
# MCP Context Browser

<span class="version-badge">v0.1.1</span>

**Semantic code search using vector embeddings** - A Model Context Protocol (MCP) server that provides intelligent code understanding through AI-powered embeddings.

## Key Features

- **12 Programming Languages** - AST-based parsing for accurate code understanding
- **6 Embedding Providers** - OpenAI, Ollama, Gemini, VoyageAI, FastEmbed, Null
- **6 Vector Stores** - Milvus, In-Memory, Filesystem, Encrypted, EdgeVec, Null
- **Hybrid Search** - Combines BM25 text search with semantic similarity
- **Enterprise Ready** - JWT auth, rate limiting, metrics, circuit breakers

## Quick Start

```bash
# Build and run
cargo build --release
./target/release/mcb

# Or with cargo
cargo run --release
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `index_codebase` | Index a directory for semantic search |
| `search_code` | Search indexed code by meaning |
| `get_indexing_status` | Check indexing progress |
| `clear_index` | Clear indexed data |

## Architecture

Built on a clean hexagonal architecture with:

- **Domain Layer** - Core types, validation, error handling
- **Application Layer** - Business services (Context, Indexing, Search)
- **Adapters Layer** - Provider implementations, repositories
- **Infrastructure Layer** - Config, metrics, caching, DI

See [Architecture Overview](architecture/ARCHITECTURE.md) for details.

---

*Documentation generated with mdbook*
EOF

        log_success "README.md created"
    fi
}

# Create placeholder files for missing docs
create_placeholders() {
    log_info "Creating placeholder files..."

    # API docs
    for doc in core-types providers services; do
        if [ ! -f "${BOOK_SRC}/api/${doc}.md" ]; then
            echo "# ${doc^}" > "${BOOK_SRC}/api/${doc}.md"
            echo "" >> "${BOOK_SRC}/api/${doc}.md"
            echo "*Documentation pending*" >> "${BOOK_SRC}/api/${doc}.md"
        fi
    done

    # User guide docs
    for doc in installation configuration; do
        if [ ! -f "${BOOK_SRC}/user-guide/${doc}.md" ]; then
            echo "# ${doc^}" > "${BOOK_SRC}/user-guide/${doc}.md"
            echo "" >> "${BOOK_SRC}/user-guide/${doc}.md"
            echo "*Documentation pending*" >> "${BOOK_SRC}/user-guide/${doc}.md"
        fi
    done

    # VERSION_HISTORY.md
    if [ -f "${DOCS_DIR}/VERSION_HISTORY.md" ]; then
        cp "${DOCS_DIR}/VERSION_HISTORY.md" "${BOOK_SRC}/"
    elif [ ! -f "${BOOK_SRC}/VERSION_HISTORY.md" ]; then
        echo "# Version History" > "${BOOK_SRC}/VERSION_HISTORY.md"
        echo "" >> "${BOOK_SRC}/VERSION_HISTORY.md"
        echo "See [CHANGELOG](operations/CHANGELOG.md)" >> "${BOOK_SRC}/VERSION_HISTORY.md"
    fi

    log_success "Placeholders created"
}

# Main
main() {
    log_info "MdBook Documentation Sync"
    echo "========================="

    create_directories
    sync_adrs
    sync_architecture
    sync_operations
    sync_developer
    sync_user_guide
    sync_modules
    create_readme
    create_placeholders

    log_success "Documentation sync complete!"
    log_info "Run: mdbook build book/ or mdbook serve book/"
}

main "$@"
