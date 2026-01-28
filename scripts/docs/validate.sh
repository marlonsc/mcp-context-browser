#!/bin/bash
# =============================================================================
# MCP Context Browser - Unified Documentation Validation
# =============================================================================
# Single entry point for all documentation validation
# Usage: ./validate.sh [all|adrs|structure|links|markdown]
# =============================================================================

set -e

# Source shared library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/lib/common.sh"

# =============================================================================
# ADR Validation Functions
# =============================================================================

validate_adr_format() {
    local adr_file="$1"
    local filename
    filename=$(basename "$adr_file")

    log_info "Validating ADR: $filename"

    # Check filename format (should be NNN-title.md where NNN is 3 digits)
    if ! is_adr_file "$adr_file"; then
        log_error "ADR filename format incorrect: $filename (should be NNN-title.md)"
        inc_errors
        return
    fi

    # Extract ADR number (remove leading zeros for comparison)
    local adr_num
    adr_num=$(get_adr_number "$adr_file" | sed 's/^0*//')

    # Check ADR number in title (supports both "ADR 001:" and "ADR 1:" formats)
    local first_line
    first_line=$(head -1 "$adr_file" | tr -d '\r')
    local adr_num_padded
    adr_num_padded=$(printf "%03d" "$adr_num")
    if ! echo "$first_line" | grep -qE "^# ADR (0*)?$adr_num:"; then
        log_error "ADR number mismatch in $filename"
        log_error "Expected ADR number: $adr_num or $adr_num_padded"
        log_error "Found: $first_line"
        inc_errors
    fi

    # Check required sections
    local required_sections=("## Status" "## Context" "## Decision")
    local has_errors=false

    for section in "${required_sections[@]}"; do
        if ! grep -q "^$section" "$adr_file"; then
            log_error "ADR $filename missing required section: $section"
            has_errors=true
        fi
    done

    # Check status values using library function
    local status_value
    status_value=$(get_adr_status "$adr_file")
    if [[ -n "$status_value" ]]; then
        if ! validate_adr_status "$status_value"; then
            log_error "ADR $filename has invalid status: '$status_value'"
            has_errors=true
        fi
    fi

    # Check for consequences section if status is Accepted
    if [[ "$status_value" == "Accepted"* ]]; then
        if ! grep -q "^## Consequences" "$adr_file"; then
            log_warning "ADR $filename (Accepted) missing Consequences section"
            inc_warnings
        fi
        if ! grep -q "^## Alternatives Considered" "$adr_file"; then
            log_warning "ADR $filename (Accepted) missing Alternatives Considered section"
            inc_warnings
        fi
    fi

    if [[ "$has_errors" == "false" ]]; then
        log_success "ADR $filename format is valid"
    else
        inc_errors
    fi
}

check_adr_numbering() {
    log_info "Checking ADR numbering consistency..."

    local adr_files
    adr_files=$(ls "$ADR_DIR" 2>/dev/null | grep -E '^[0-9]{3}-.*\.md$' | sort -V)
    local expected_num=1

    for adr_file in $adr_files; do
        local actual_num
        actual_num=$(echo "$adr_file" | grep -oE '^[0-9]+' | sed 's/^0*//')
        [[ -z "$actual_num" ]] && actual_num=0
        if [[ "$actual_num" -ne "$expected_num" ]]; then
            log_error "ADR numbering gap: expected $expected_num, found $actual_num"
            inc_errors
        fi
        ((expected_num++))
    done

    log_success "ADR numbering is consistent"
}

check_adr_references() {
    log_info "Checking ADR references in documentation..."

    local adr_files
    adr_files=$(ls "$ADR_DIR" 2>/dev/null | grep -E '^[0-9]{3}-.*\.md$')
    local arch_doc="$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md"

    if [[ -f "$arch_doc" ]]; then
        for adr_file in $adr_files; do
            local adr_num
            adr_num=$(echo "$adr_file" | grep -oE '^[0-9]+' | sed 's/^0*//')
            if [[ -n "$adr_num" ]] && ! grep -q "ADR.* $adr_num\|ADR-0*$adr_num\|ADR 0*$adr_num" "$arch_doc"; then
                log_warning "ADR $adr_num not referenced in architecture documentation"
                inc_warnings
            fi
        done
    fi

    log_success "ADR reference check completed"
}

# =============================================================================
# Link Validation Functions
# =============================================================================

extract_links() {
    local file="$1"
    grep -o '\[.*\](\([^)]*\))' "$file" 2>/dev/null | sed 's/.*(\([^)]*\))/\1/' | grep '^docs/' || true
}

validate_link() {
    local link="$1"
    local clean_link="${link%%#*}"

    if [[ -f "$PROJECT_ROOT/$clean_link" ]]; then
        return 0
    elif [[ -d "$PROJECT_ROOT/$clean_link" ]]; then
        if [[ -f "$PROJECT_ROOT/$clean_link/README.md" ]] || [[ -f "$PROJECT_ROOT/$clean_link/index.md" ]]; then
            return 0
        fi
    fi
    return 1
}

validate_doc_links() {
    log_info "Validating documentation links..."

    local doc_files
    doc_files=$(find_markdown_files "$DOCS_DIR")

    for doc_file in $doc_files; do
        local filename
        filename=$(basename "$doc_file")
        log_info "Checking links in: $filename"

        local links
        links=$(extract_links "$doc_file")

        for link in $links; do
            if ! validate_link "$link"; then
                log_error "Broken link in $filename: $link"
                inc_errors
            fi
        done
    done
}

validate_cross_references() {
    log_info "Validating cross-references..."

    local key_files=(
        "docs/README.md"
        "docs/user-guide/README.md"
        "docs/architecture/ARCHITECTURE.md"
        "docs/operations/DEPLOYMENT.md"
    )

    for file in "${key_files[@]}"; do
        if [[ -f "$PROJECT_ROOT/$file" ]]; then
            local filename
            filename=$(basename "$file")
            if ! grep -q "$filename" "$PROJECT_ROOT/docs/README.md" 2>/dev/null; then
                log_warning "Main docs index may not reference: $filename"
                inc_warnings
            fi
        fi
    done
}

validate_external_links() {
    log_info "Checking external links (basic validation)..."

    local doc_files
    doc_files=$(find_markdown_files "$DOCS_DIR")

    for doc_file in $doc_files; do
        local external_links
        external_links=$(grep -o 'https*://[^)]*' "$doc_file" 2>/dev/null || true)

        for link in $external_links; do
            if [[ "$link" =~ localhost ]] || [[ "$link" =~ 127\.0\.0\.1 ]] || [[ "$link" =~ example\.com ]]; then
                continue
            fi

            if check_executable curl; then
                local status
                status=$(curl -s -o /dev/null -w "%{http_code}" --max-time 5 "$link" 2>/dev/null || echo "000")
                if [[ "$status" =~ ^[45][0-9][0-9]$ ]]; then
                    log_warning "External link may be broken: $link (status: $status)"
                fi
            fi
        done
    done
}

# =============================================================================
# Structure Validation Functions
# =============================================================================

validate_doc_structure() {
    log_info "Validating documentation structure..."

    # Check main directories
    check_directory "$PROJECT_ROOT/docs" "Main documentation directory" || true
    check_directory "$PROJECT_ROOT/docs/user-guide" "User guide documentation" || true
    check_directory "$PROJECT_ROOT/docs/developer" "Developer documentation" || true
    check_directory "$PROJECT_ROOT/docs/architecture" "Architecture documentation" || true
    check_directory "$PROJECT_ROOT/docs/operations" "Operations documentation" || true
    check_directory "$PROJECT_ROOT/docs/templates" "Documentation templates" || true

    # Check key files
    check_file "$PROJECT_ROOT/docs/README.md" "Documentation index" || true
    check_file "$PROJECT_ROOT/docs/user-guide/README.md" "User guide" || true
    check_file "$PROJECT_ROOT/docs/developer/CONTRIBUTING.md" "Contributing guide" || true
    check_file "$PROJECT_ROOT/docs/developer/ROADMAP.md" "Development roadmap" || true
    check_file "$PROJECT_ROOT/docs/architecture/ARCHITECTURE.md" "Architecture overview" || true
    check_file "$PROJECT_ROOT/docs/operations/DEPLOYMENT.md" "Deployment guide" || true
    check_file "$PROJECT_ROOT/docs/operations/CHANGELOG.md" "Changelog" || true
    check_file "$PROJECT_ROOT/docs/templates/adr-template.md" "ADR template" || true

    # Check architecture subdirectories
    check_directory "$PROJECT_ROOT/docs/adr" "Architecture Decision Records" || true
    check_directory "$PROJECT_ROOT/docs/diagrams" "Architecture diagrams" || true

    # Check for orphaned files in root
    local orphaned_files=("ARCHITECTURE.md" "CONTRIBUTING.md" "ROADMAP.md" "DEPLOYMENT.md" "CHANGELOG.md")
    for file in "${orphaned_files[@]}"; do
        if [[ -f "$PROJECT_ROOT/$file" ]]; then
            log_error "Orphaned documentation file in root: $file (should be in docs/)"
            inc_errors
        fi
    done
}

validate_permissions() {
    log_info "Validating file permissions..."

    local scripts=("generate-diagrams.sh" "validate.sh" "create-adr.sh" "mdbook-sync.sh" "markdown.sh" "extract-metrics.sh" "inject-metrics.sh" "generate-module-docs.sh")

    for script in "${scripts[@]}"; do
        local script_path="$SCRIPT_DIR/$script"
        if [[ -f "$script_path" ]]; then
            if [[ ! -x "$script_path" ]]; then
                log_error "Script not executable: $script"
                inc_errors
            else
                log_success "Script executable: $script"
            fi
        else
            log_warning "Script missing: $script"
            inc_warnings
        fi
    done
}

# =============================================================================
# Validation Runners
# =============================================================================

run_adr_validation() {
    log_header "ADR Validation"
    log_info "MCP Context Browser - ADR Validation"
    log_info "===================================="

    if ! check_directory "$ADR_DIR" "ADR directory"; then
        return 1
    fi

    # Validate each ADR file
    for adr_file in "$ADR_DIR"/*.md; do
        local basename_file
        basename_file=$(basename "$adr_file")
        if [[ -f "$adr_file" ]] && \
           [[ "$basename_file" != "README.md" ]] && \
           [[ "$basename_file" != "TEMPLATE.md" ]] && \
           [[ "$basename_file" != "adr-graph.md" ]] && \
           [[ "$basename_file" != "CLAUDE.md" ]]; then
            validate_adr_format "$adr_file"
        fi
    done

    check_adr_numbering
    check_adr_references

    print_summary "ADR Validation"
}

run_structure_validation() {
    log_header "Structure Validation"
    log_info "MCP Context Browser - Documentation Structure Validation"
    log_info "======================================================="

    validate_doc_structure
    validate_permissions

    print_summary "Structure Validation"
}

run_link_validation() {
    log_header "Link Validation"
    log_info "MCP Context Browser - Documentation Link Validation"
    log_info "==================================================="

    validate_doc_links
    validate_cross_references

    if [[ -n "${QUICK:-}" ]] && [[ "${QUICK}" != "0" ]]; then
        log_info "QUICK=1: skipping external link validation"
    elif check_executable curl; then
        validate_external_links
    else
        log_warning "curl not available - skipping external link validation"
    fi

    print_summary "Link Validation"
}

run_markdown_validation() {
    log_header "Markdown Validation"
    "$SCRIPT_DIR/markdown.sh" lint || inc_errors
}

run_all_validations() {
    log_info "Running all documentation validations..."
    echo

    run_adr_validation
    echo
    run_structure_validation
    echo
    run_link_validation
    echo

    if check_executable markdownlint; then
        run_markdown_validation
        echo
    else
        log_warning "Skipping markdown lint (markdownlint not installed)"
    fi
}

# =============================================================================
# Main
# =============================================================================

show_usage() {
    cat << EOF
MCP Context Browser - Unified Documentation Validation

USAGE:
    $0 all               # Run all validations
    $0 adrs              # Validate ADR format and numbering
    $0 structure         # Validate documentation structure
    $0 links             # Validate internal/external links
    $0 markdown          # Lint markdown files

EXAMPLES:
    $0 all               # Full validation suite
    $0 adrs              # Quick ADR check
    $0 structure links   # Multiple specific checks

MAKE TARGETS:
    make docs-check      # Runs this script with 'all'
    make adr-check       # Runs this script with 'adrs'

EOF
}

main() {
    log_info "MCP Context Browser - Documentation Validation"
    log_info "=============================================="
    echo

    # No args = all validations
    if [[ $# -eq 0 ]]; then
        run_all_validations
        exit_with_summary "Overall Validation Summary"
    fi

    # Process each argument
    for target in "$@"; do
        case "$target" in
            all)
                run_all_validations
                ;;
            adrs|adr)
                run_adr_validation
                ;;
            structure)
                run_structure_validation
                ;;
            links)
                run_link_validation
                ;;
            markdown|md)
                run_markdown_validation
                ;;
            help|--help|-h)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown validation target: $target"
                show_usage
                exit 1
                ;;
        esac
        echo
    done

    exit_with_summary "Validation Summary"
}

main "$@"
