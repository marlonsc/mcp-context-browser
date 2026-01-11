#!/bin/bash

# MCP Context Browser - Initialize mdbook Documentation
# Creates basic mdbook structure for v0.1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create mdbook structure
create_mdbook_structure() {
    local book_dir="$PROJECT_ROOT/docs/book"

    log_info "Creating mdbook structure in $book_dir"

    # Create directories
    mkdir -p "$book_dir/src"

    # Create book.toml
    cat > "$book_dir/book.toml" << 'EOF'
[book]
title = "MCP Context Browser - v0.1.0"
author = "Marlon Carvalho"
description = "Interactive documentation for the MCP Context Browser project"

[build]
build-dir = "../generated/mdbook"

[output.html]
additional-css = ["custom.css"]
git-repository-url = "https://github.com/marlonsc/mcp-context-browser"
edit-url-template = "https://github.com/marlonsc/mcp-context-browser/edit/main/{path}"

[output.html.search]
enable = true
limit-results = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1

[output.html.fold]
enable = true
level = 1
EOF

    # Create SUMMARY.md
    cat > "$book_dir/src/SUMMARY.md" << 'EOF'
# Summary

# 📖 MCP Context Browser v0.1.0

- [🏠 Introduction](introduction.md)
- [🚀 Getting Started](getting-started.md)

---

# 🏗️ Architecture & Design

- [Architecture Decision Records](adr/index.md)
  - [ADR 001: Provider Pattern Architecture](adr/001-provider-pattern-architecture.md)
  - [ADR 002: Async-First Architecture](adr/002-async-first-architecture.md)
  - [ADR 003: C4 Model Documentation](adr/003-c4-model-documentation.md)
  - [ADR 005: Documentation Excellence](adr/005-documentation-excellence.md)
- [📊 System Context](architecture/system-context.md)
- [🏛️ Container Architecture](architecture/container-architecture.md)

---

# 🔧 Implementation

- [📚 API Reference](api/index.md)
- [📁 Modules](modules/index.md)
- [📊 Metrics & Monitoring](metrics/index.md)

---

# 🚀 Deployment & Operations

- [🏭 Production Deployment](deployment/production.md)
- [🐳 Docker Configuration](deployment/docker.md)

---

# 🧪 Development

- [🏗️ Development Setup](development/setup.md)
- [🧪 Testing](development/testing.md)
- [📝 Contributing](development/contributing.md)

---

# 📋 Project Management

- [🎯 Roadmap](roadmap.md)
- [📈 Implementation Status](status.md)
EOF

    # Create introduction.md
    cat > "$book_dir/src/introduction.md" << 'EOF'
# MCP Context Browser - v0.1.0

Welcome to the **MCP Context Browser** project! This is a comprehensive Model Context Protocol (MCP) server implementation that provides semantic code analysis using advanced vector embeddings and intelligent chunking.

## 🎯 What is MCP Context Browser v0.1.0?

This version establishes the project as a **reference implementation for automated, self-documenting systems**.

### Key Features

- **Self-Documenting Codebase**: Documentation generated automatically from source code
- **ADR-Driven Development**: Architectural decisions captured and validated
- **Quality Assurance Gates**: Automated validation preventing documentation drift
- **Interactive Experience**: Modern documentation with search and cross-references

## 📖 Documentation Features

### 🤖 Automated Generation
- Module structure analysis
- API surface extraction
- Dependency graph visualization
- Code metrics and complexity analysis

### 📋 ADR Management
- Professional ADR tooling with `adrs`
- Automated compliance validation
- Status tracking and lifecycle management

### 🔍 Quality Assurance
- Spelling validation with `cargo-spellcheck`
- Link validation with `cargo-deadlinks`
- Automated quality gates in CI/CD

## 🚀 Quick Start

```bash
# Install documentation tools
./scripts/docs/automation.sh setup

# Generate documentation
./scripts/docs/automation.sh generate

# Serve interactive documentation
./scripts/docs/automation.sh mdbook serve
```

---

*This documentation is automatically generated and always up-to-date.*
EOF

    # Create getting-started.md
    cat > "$book_dir/src/getting-started.md" << 'EOF'
# Getting Started

## Prerequisites

- Rust 1.70+
- Docker (for testing)
- Git

## Installation

```bash
# Clone the repository
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Build the project
cargo build --release
```

## Quick Test

```bash
# Run tests
cargo test

# Start the MCP server
cargo run --bin mcp-context-browser
```

## Documentation

```bash
# Install documentation tools
./scripts/docs/automation.sh setup

# Generate all documentation
./scripts/docs/automation.sh generate

# View interactive documentation
./scripts/docs/automation.sh mdbook serve
```
EOF

    log_success "mdbook structure created"
}

# Create custom CSS
create_custom_css() {
    local book_dir="$PROJECT_ROOT/docs/book"

    cat > "$book_dir/src/custom.css" << 'EOF'
/* Custom styles for MCP Context Browser documentation */

:root {
    --sidebar-width: 300px;
    --sidebar-non-existant: calc(-1 * var(--sidebar-width));
}

.sidebar {
    width: var(--sidebar-width);
}

.sidebar-hidden .sidebar {
    transform: translateX(var(--sidebar-non-existant));
}

.content {
    margin-left: var(--sidebar-width);
}

.sidebar-hidden .content {
    margin-left: 0;
}

/* Code highlighting improvements */
.code-attribute,
.code-builtin,
.code-comment,
.code-constant,
.code-function,
.code-keyword,
.code-literal,
.code-macro,
.code-meta,
.code-string,
.code-type {
    font-family: 'Fira Code', 'Source Code Pro', monospace;
}

/* ADR specific styling */
.adr-status {
    padding: 0.2em 0.5em;
    border-radius: 0.2em;
    font-size: 0.8em;
    font-weight: bold;
}

.adr-accepted {
    background-color: #d4edda;
    color: #155724;
}

.adr-proposed {
    background-color: #fff3cd;
    color: #856404;
}

.adr-rejected {
    background-color: #f8d7da;
    color: #721c24;
}

/* Metric badges */
.metric-badge {
    display: inline-block;
    padding: 0.2em 0.5em;
    margin: 0.1em;
    border-radius: 0.3em;
    font-size: 0.8em;
    font-weight: bold;
    background-color: #007acc;
    color: white;
}
EOF
}

# Main execution
main() {
    log_info "MCP Context Browser - mdbook Initialization"
    echo "============================================="

    create_mdbook_structure
    create_custom_css

    log_success "mdbook initialization completed"
    log_info "Run: ./scripts/docs/automation.sh mdbook serve"
    log_info "Or:  ./scripts/docs/automation.sh mdbook build"
}

main "$@"