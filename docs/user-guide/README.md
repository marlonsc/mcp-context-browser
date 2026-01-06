# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.0.2--alpha-blue)](https://github.com/marlonsc/mcp-context-browser/releases)

**Model Context Protocol Server** - Provides semantic code search and analysis capabilities to AI assistants through a standardized MCP interface.

## 🎯 Current Capabilities (v0.0.2)

### Core Features

-   **🔍 Vector-Based Search**: Semantic similarity search using embeddings
-   **💾 In-Memory Storage**: Fast vector storage for development and testing
-   **🎭 Mock Embeddings**: Fixed-dimension embedding generation for testing
-   **🔧 MCP Protocol**: Basic MCP server implementation with stdio transport
-   **📁 File Processing**: Simple text-based code file reading and chunking

### Architecture

-   **🏗️ Modular Design**: Clean separation with core, providers, services, and server layers
-   **🔌 Provider Pattern**: Extensible system for embeddings and vector storage
-   **⚡ Async Processing**: Tokio-based asynchronous operations
-   **🛡️ Error Handling**: Comprehensive error types with detailed diagnostics

## 📋 Documentation

-   [**ARCHITECTURE.md**](../architecture/ARCHITECTURE.md) - Technical architecture and design
-   [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
-   [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
-   [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

## 📦 Quick Start

```bash
# Install Rust and clone
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Run development setup
make setup
make dev
```

## 🧪 Testing

The project follows TDD (Test-Driven Development) principles with comprehensive test coverage:

```bash
# Run all tests
make test

# Run tests with coverage
make coverage

# Run tests in watch mode
make test-watch

# Run all validation checks
make validate
```

### Test Structure

-   **Core Types**: Data structure validation and serialization
-   **Services**: Business logic testing (Context, Indexing, Search)
-   **MCP Protocol**: Protocol compliance and message handling
-   **Integration**: End-to-end functionality testing

### CI/CD

GitHub Actions automatically runs:

-   **Tests**: Multiple Rust versions (stable, beta, MSRV)
-   **Linting**: Code formatting and clippy checks
-   **Security**: Dependency vulnerability scanning
-   **Coverage**: Code coverage reporting
-   **Build**: Cross-platform binary builds

[![CI](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml)

## 🤝 Contributing

See [**CONTRIBUTING.md**](CONTRIBUTING.md) for detailed contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
