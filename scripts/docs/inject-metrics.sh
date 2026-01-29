#!/bin/bash
# =============================================================================
# INJECT-METRICS.SH - Update documentation with extracted metrics
# =============================================================================
# Uses extract-metrics.sh as single source of truth to update all docs
# Usage: ./inject-metrics.sh [--dry-run]
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DRY_RUN=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run) DRY_RUN=true; shift ;;
        --verbose|-v) VERBOSE=true; shift ;;
        *) shift ;;
    esac
done

# Load metrics (dynamic source - extract-metrics.sh outputs env vars)
# shellcheck disable=SC1090
source <("$SCRIPT_DIR/extract-metrics.sh" --env)

log() {
    if $VERBOSE; then
        echo "[INFO] $*"
    fi
}

# -----------------------------------------------------------------------------
# Update function - replaces patterns in files
# -----------------------------------------------------------------------------
update_file() {
    local file="$1"
    local pattern="$2"
    local replacement="$3"
    local description="$4"

    if [[ ! -f "$file" ]]; then
        log "File not found: $file"
        return
    fi

    if grep -qE "$pattern" "$file" 2>/dev/null; then
        if $DRY_RUN; then
            echo "[DRY-RUN] Would update $file: $description"
            grep -nE "$pattern" "$file" | head -3
        else
            sed -i -E "s|$pattern|$replacement|g" "$file"
            echo "✅ Updated $file: $description"
        fi
    fi
}

# -----------------------------------------------------------------------------
# Update version badges
# -----------------------------------------------------------------------------
update_version_badges() {
    echo "📦 Updating version badges..."

    # README badges: version-X.X.X-blue
    update_file "$PROJECT_ROOT/docs/user-guide/README.md" \
        'version-[0-9]+\.[0-9]+\.[0-9]+-blue' \
        "version-${MCP_VERSION}-blue" \
        "version badge"

    update_file "$PROJECT_ROOT/README.md" \
        'version-[0-9]+\.[0-9]+\.[0-9]+-blue' \
        "version-${MCP_VERSION}-blue" \
        "version badge"
}

# -----------------------------------------------------------------------------
# Update capability sections
# -----------------------------------------------------------------------------
update_capabilities() {
    echo "🎯 Updating capability counts..."

    local files=(
        "$PROJECT_ROOT/README.md"
        "$PROJECT_ROOT/CLAUDE.md"
        "$PROJECT_ROOT/docs/user-guide/README.md"
        "$PROJECT_ROOT/docs/developer/ROADMAP.md"
        "$PROJECT_ROOT/docs/implementation-status.md"
        "$PROJECT_ROOT/docs/VERSION_HISTORY.md"
        "$PROJECT_ROOT/docs/operations/CHANGELOG.md"
        "$PROJECT_ROOT/docs/book/src/introduction.md"
    )

    for file in "${files[@]}"; do
        [[ -f "$file" ]] || continue

        # Update language count patterns
        update_file "$file" \
            '[0-9]+\+? (programming )?languages?' \
            "${MCP_LANGUAGE_COUNT} languages" \
            "language count"
        
        # Update "X programming languages" pattern
        update_file "$file" \
            '[0-9]+\+? programming languages' \
            "${MCP_LANGUAGE_COUNT} programming languages" \
            "programming languages count"

        update_file "$file" \
            '[0-9]+\+? language processors' \
            "${MCP_LANGUAGE_COUNT} language processors" \
            "language processor count"

        # Update embedding provider count
        update_file "$file" \
            '[0-9]+\+? embedding providers?' \
            "${MCP_EMBEDDING_COUNT} embedding providers" \
            "embedding provider count"

        # Update vector store count
        update_file "$file" \
            '[0-9]+\+? vector stores?' \
            "${MCP_VECTOR_STORE_COUNT} vector stores" \
            "vector store count"
        
        # Update "X+ vector stores" pattern
        update_file "$file" \
            '[0-9]+\+ vector stores' \
            "${MCP_VECTOR_STORE_COUNT} vector stores" \
            "vector stores with plus"

        # Update test count
        update_file "$file" \
            '[0-9]+\+ (comprehensive )?(business scenario )?tests' \
            "${MCP_TEST_COUNT}+ tests" \
            "test count"
    done
}

# -----------------------------------------------------------------------------
# Update ADR references
# -----------------------------------------------------------------------------
update_adr_counts() {
    echo "📋 Updating ADR counts..."

    local files=(
        "$PROJECT_ROOT/docs/adr/README.md"
        "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"
    )

    for file in "${files[@]}"; do
        [[ -f "$file" ]] || continue

        update_file "$file" \
            '[0-9]+ ADRs? (found|documented|available)' \
            "${MCP_ADR_COUNT} ADRs documented" \
            "ADR count"
    done
}

# -----------------------------------------------------------------------------
# Update version in Current Capabilities section headers
# -----------------------------------------------------------------------------
update_version_headers() {
    echo "📄 Updating version headers..."

    # Update "Current Capabilities (vX.X.X)" headers
    update_file "$PROJECT_ROOT/docs/user-guide/README.md" \
        'Current Capabilities \(v[0-9]+\.[0-9]+\.[0-9]+\)' \
        "Current Capabilities (v${MCP_VERSION})" \
        "capability header version"

    # Update "v0.0.X production-ready" in CLAUDE.md
    update_file "$PROJECT_ROOT/CLAUDE.md" \
        'v[0-9]+\.[0-9]+\.[0-9]+ production-ready' \
        "v${MCP_VERSION} production-ready" \
        "production ready version"

    # Update "Current Version: vX.X.X"
    update_file "$PROJECT_ROOT/CLAUDE.md" \
        'Current Version: v[0-9]+\.[0-9]+\.[0-9]+' \
        "Current Version: v${MCP_VERSION}" \
        "current version header"
}

# -----------------------------------------------------------------------------
# Generate metrics file for reference
# -----------------------------------------------------------------------------
generate_metrics_file() {
    echo "📊 Generating metrics reference file..."

    local metrics_file="$PROJECT_ROOT/docs/generated/METRICS.md"
    mkdir -p "$(dirname "$metrics_file")"

    if $DRY_RUN; then
        echo "[DRY-RUN] Would generate $metrics_file"
        return
    fi

    cat > "$metrics_file" <<EOF
# Project Metrics - Auto-Generated

> **Single Source of Truth**: This file is auto-generated by \`scripts/docs/inject-metrics.sh\`
> Do not edit manually. Run \`make docs-metrics\` to regenerate.

## Current Metrics (v${MCP_VERSION})

| Metric | Value |
|--------|-------|
| **Version** | ${MCP_VERSION} |
| **Languages** | ${MCP_LANGUAGE_COUNT} |
| **Embedding Providers** | ${MCP_EMBEDDING_COUNT} |
| **Vector Stores** | ${MCP_VECTOR_STORE_COUNT} |
| **ADRs** | ${MCP_ADR_COUNT} |
| **Tests** | ${MCP_TEST_COUNT}+ |
| **Source Files** | ${MCP_SOURCE_FILES} |
| **Source Lines** | ${MCP_SOURCE_LINES} |
| **Test Files** | ${MCP_TEST_FILES} |
| **Module Docs** | ${MCP_MODULE_DOCS} |

## Language Support

${MCP_LANGUAGE_LIST}

## Embedding Providers

${MCP_EMBEDDING_LIST}

## Vector Stores

${MCP_VECTOR_STORE_LIST}

---

*Generated: $(date '+%Y-%m-%d %H:%M:%S')*
*Source: \`scripts/docs/extract-metrics.sh\`*
EOF

    echo "✅ Generated $metrics_file"
}

# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------

echo "🔄 Injecting metrics into documentation..."
echo "   Version: ${MCP_VERSION}"
echo "   Languages: ${MCP_LANGUAGE_COUNT}"
echo "   Embeddings: ${MCP_EMBEDDING_COUNT}"
echo "   Vector Stores: ${MCP_VECTOR_STORE_COUNT}"
echo "   Tests: ${MCP_TEST_COUNT}+"
echo ""

if $DRY_RUN; then
    echo "🔍 DRY RUN MODE - No files will be modified"
    echo ""
fi

update_version_badges
update_capabilities
update_adr_counts
update_version_headers
generate_metrics_file

echo ""
echo "✅ Metrics injection complete"
