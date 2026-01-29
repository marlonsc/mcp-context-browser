#!/bin/bash

# MCP Context Browser - Diagram Generation Script
# Generates PlantUML diagrams for architecture documentation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIAGRAMS_DIR="$PROJECT_ROOT/docs/architecture/diagrams"
OUTPUT_DIR="$PROJECT_ROOT/docs/architecture/diagrams/generated"

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

# Check if plantuml is installed
check_plantuml() {
    if ! command -v plantuml &> /dev/null; then
        log_error "PlantUML is not installed. Please install it first:"
        echo "  - Ubuntu/Debian: sudo apt-get install plantuml"
        echo "  - macOS: brew install plantuml"
        echo "  - Or download from: https://plantuml.com/download"
        exit 1
    fi

    log_success "PlantUML found: $(plantuml -version 2>&1 | head -1)"
}

# Create output directory
setup_output_dir() {
    mkdir -p "$OUTPUT_DIR"
    log_info "Output directory: $OUTPUT_DIR"
}

# Generate diagrams from PlantUML files
generate_diagrams() {
    local format="$1"
    local input_files=("$DIAGRAMS_DIR"/*.puml)

    if [ ${#input_files[@]} -eq 0 ] || [ ! -f "${input_files[0]}" ]; then
        log_warning "No PlantUML files found in $DIAGRAMS_DIR"
        return 0
    fi

    log_info "Generating $format diagrams..."

    for input_file in "${input_files[@]}"; do
        if [ -f "$input_file" ]; then
            local filename output_file
            filename=$(basename "$input_file" .puml)
            output_file="$OUTPUT_DIR/$filename.$format"

            log_info "Processing: $filename.puml -> $filename.$format"

            if plantuml -t"$format" "$input_file" -o "$OUTPUT_DIR"; then
                log_success "Generated: $output_file"
            else
                log_error "Failed to generate: $output_file"
            fi
        fi
    done
}

# Generate Mermaid diagrams (alternative to PlantUML)
generate_mermaid_diagrams() {
    log_info "Mermaid diagram generation requires manual processing"
    log_info "Use tools like:"
    echo "  - mmdc (Mermaid CLI): npm install -g @mermaid-js/mermaid-cli"
    echo "  - Online editor: https://mermaid.live/"
    echo "  - VS Code extension: Markdown Preview Mermaid Support"
}

# Validate PlantUML syntax
validate_diagrams() {
    log_info "Validating PlantUML syntax..."

    local input_files=("$DIAGRAMS_DIR"/*.puml)
    local errors=0

    for input_file in "${input_files[@]}"; do
        if [ -f "$input_file" ]; then
            local filename
            filename=$(basename "$input_file")

            if plantuml -checkonly "$input_file" 2>/dev/null; then
                log_success "Valid: $filename"
            else
                log_error "Invalid syntax: $filename"
                ((errors++))
            fi
        fi
    done

    if [ $errors -gt 0 ]; then
        log_error "Found $errors diagram(s) with syntax errors"
        return 1
    else
        log_success "All diagrams are syntactically valid"
    fi
}

# Generate documentation index
generate_index() {
    log_info "Generating diagram index..."

    local index_file="$OUTPUT_DIR/index.html"

    cat > "$index_file" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MCP Context Browser - Architecture Diagrams</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }
        .diagram-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr)); gap: 20px; }
        .diagram-card { border: 1px solid #ddd; border-radius: 8px; padding: 16px; }
        .diagram-card img { max-width: 100%; height: auto; }
        .diagram-title { font-weight: bold; margin-bottom: 8px; }
        .diagram-desc { color: #666; font-size: 14px; }
    </style>
</head>
<body>
    <h1>MCP Context Browser - Architecture Diagrams</h1>
    <p>This page contains all architecture diagrams for the MCP Context Browser project.</p>

    <div class="diagram-grid">
EOF

    # Add diagram entries
    for png_file in "$OUTPUT_DIR"/*.png; do
        if [ -f "$png_file" ]; then
            local filename svg_file
            filename=$(basename "$png_file" .png)
            svg_file="$OUTPUT_DIR/$filename.svg"

            cat >> "$index_file" << EOF
        <div class="diagram-card">
            <div class="diagram-title">$filename</div>
            <div class="diagram-desc">Architecture diagram</div>
            <img src="$(basename "$png_file")" alt="$filename" loading="lazy">
EOF

            if [ -f "$svg_file" ]; then
                echo "            <a href=\"$(basename "$svg_file")\">Download SVG</a>" >> "$index_file"
            fi

            echo "        </div>" >> "$index_file"
        fi
    done

    cat >> "$index_file" << 'EOF'
    </div>

    <footer style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #eee; color: #666;">
        <p>Generated on: <span id="timestamp"></span></p>
        <script>document.getElementById('timestamp').textContent = new Date().toLocaleString();</script>
    </footer>
</body>
</html>
EOF

    log_success "Generated diagram index: $index_file"
}

# Clean generated files
clean_generated() {
    if [ -d "${OUTPUT_DIR:?}" ]; then
        log_info "Cleaning generated diagrams..."
        rm -rf "${OUTPUT_DIR:?}"/*
        log_success "Cleaned: $OUTPUT_DIR"
    else
        log_info "No generated files to clean"
    fi
}

# Show usage information
show_usage() {
    cat << EOF
MCP Context Browser - Diagram Generation Tool

USAGE:
    $0 [COMMAND] [OPTIONS]

COMMANDS:
    all             Generate all diagram formats (PNG, SVG) and index
    png             Generate PNG diagrams only
    svg             Generate SVG diagrams only
    validate        Validate PlantUML syntax without generating
    index           Generate HTML index page only
    clean           Remove all generated files
    help            Show this help message

OPTIONS:
    --mermaid       Include Mermaid diagram generation info

EXAMPLES:
    $0 all          # Generate everything
    $0 validate     # Check syntax only
    $0 clean        # Clean generated files

PLANTUML FILES:
    Place .puml files in: $DIAGRAMS_DIR/

OUTPUT DIRECTORY:
    Generated files: $OUTPUT_DIR/

EOF
}

# Main execution
main() {
    local command="${1:-all}"
    local mermaid=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --mermaid)
                mermaid=true
                shift
                ;;
            *)
                command="$1"
                shift
                ;;
        esac
    done

    log_info "MCP Context Browser - Diagram Generator"
    log_info "Command: $command"

    case $command in
        all)
            check_plantuml
            setup_output_dir
            validate_diagrams
            generate_diagrams "png"
            generate_diagrams "svg"
            generate_index
            if [ "$mermaid" = true ]; then
                generate_mermaid_diagrams
            fi
            log_success "All diagrams generated successfully"
            ;;

        png)
            check_plantuml
            setup_output_dir
            generate_diagrams "png"
            ;;

        svg)
            check_plantuml
            setup_output_dir
            generate_diagrams "svg"
            ;;

        validate)
            check_plantuml
            validate_diagrams
            ;;

        index)
            setup_output_dir
            generate_index
            ;;

        clean)
            clean_generated
            ;;

        help|--help|-h)
            show_usage
            ;;

        *)
            log_error "Unknown command: $command"
            echo
            show_usage
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"