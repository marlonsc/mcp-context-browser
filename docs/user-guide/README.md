# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.1.1-blue)](https://github.com/marlonsc/mcp-context-browser/releases)

**Model Context Protocol Server**- Provides semantic code search and analysis capabilities to AI assistants through a standardized MCP interface.

## 🎯 Current Capabilities (v0.1.1)

### Core Features

\1-  **🔍 Vector-Based Search**: Semantic similarity search using embeddings
\1-  **💾 In-Memory Storage**: Fast vector storage for development and testing
\1-  **🎭 Mock Embeddings**: Fixed-dimension embedding generation for testing
\1-  **🔧 MCP Protocol**: Basic MCP server implementation with stdio transport
\1-  **📁 File Processing**: Simple text-based code file reading and chunking

### Architecture

\1-  **🏗️ Modular Design**: Clean separation with core, providers, services, and server layers
\1-  **🔌 Provider Pattern**: Extensible system for embeddings and vector storage
\1-  **⚡ Async Processing**: Tokio-based asynchronous operations
\1-  **🛡️ Error Handling**: Comprehensive error types with detailed diagnostics

## 📋 Documentation

\1-   [**ARCHITECTURE.md**](../architecture/ARCHITECTURE.md) - Technical architecture and design
\1-   [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
\1-   [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
\1-   [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

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

\1-  **Core Types**: Data structure validation and serialization
\1-  **Services**: Business logic testing (Context, Indexing, Search)
\1-  **MCP Protocol**: Protocol compliance and message handling
\1-  **Integration**: End-to-end functionality testing

### CI/CD

GitHub Actions automatically runs:

\1-  **Tests**: Multiple Rust versions (stable, beta, MSRV)
\1-  **Linting**: Code formatting and clippy checks
\1-  **Security**: Dependency vulnerability scanning
\1-  **Coverage**: Code coverage reporting
\1-  **Build**: Cross-platform binary builds

[![CI](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml)

## 🤝 Contributing

See [**CONTRIBUTING.md**](CONTRIBUTING.md) for detailed contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
