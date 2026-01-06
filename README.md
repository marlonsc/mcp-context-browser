# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.0.2-blue)](https://github.com/marlonsc/mcp-context-browser/releases)
[![CI](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml)

**Model Context Protocol Server** - Provides semantic code search and analysis capabilities to AI assistants through a standardized MCP interface.


## 🎯 Current Capabilities (v0.0.3)

### 🏆 Features


-   **🧠 Semantic Code Search**: Hybrid BM25 + vector search using natural language queries
-   **🔄 Incremental Sync**: Automatic background synchronization with change detection
-   **💾 Persistent State**: Professional snapshot management with Keyv storage
-   **🎯 Advanced Indexing**: AST-based code chunking with custom extensions and ignore patterns
-   **🔒 Concurrency Control**: p-queue coordination with async-Mutex and file locks
-   **🔧 Multi-Provider Support**: OpenAI, VoyageAI, Gemini, Ollama embeddings + Milvus vector storage
-   **⚙️ Advanced Configuration**: convict.js schema validation with environment variables
-   **📊 Professional Monitoring**: Comprehensive status tracking and error recovery

### Core MCP Tools

-   **`index_codebase`**: Index entire codebases with AST chunking and custom configurations
-   **`search_code`**: Natural language semantic search with extension filtering
-   **`get_indexing_status`**: Real-time status monitoring with change detection
-   **`clear_index`**: Professional index management and cleanup

### Architecture

-   **🏗️ Enterprise Architecture**: SOLID principles with dependency injection
-   **🔌 Provider Pattern**: Extensible system for embeddings and vector storage
-   **⚡ Async-First Design**: Tokio runtime with streams and concurrent processing
-   **🛡️ Robust Error Handling**: Custom error types with detailed diagnostics
-   **🔄 Background Services**: Cron-based incremental updates and synchronization
-   **💾 Persistent Storage**: Keyv-based state management with automatic recovery

## 📋 Documentation

-   [**CLAUDE.md**](CLAUDE.md) - Development guide and project rules
-   [**ARCHITECTURE.md**](ARCHITECTURE.md) - Technical architecture and design
-   [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
-   [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
-   [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

## 🚀 Quick Start

```bash
# Install Rust and Node.js, then clone
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install Node.js from https://nodejs.org/
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Setup all dependencies (MANDATORY)
make setup

# Verify dependencies
make check-deps

# Run development
make dev
```

## 🧪 Testing & Quality

The project follows TDD (Test-Driven Development) principles with comprehensive test coverage and strict quality gates:

```bash
# Complete quality assurance
make quality        # fmt + lint + lint-md + test + audit + validate

# Individual checks
make test           # Run all tests (60 tests, 100% pass rate)
make lint           # Rust code linting (clippy)
make lint-md        # Markdown linting (markdownlint-cli required)
make validate       # Documentation validation
make audit          # Security audit

# Auto-fix issues
make fix            # Auto-fix formatting and markdown issues
```

### Test Structure

-   **Core Types**: Data structure validation and serialization (18 tests)
-   **Services**: Business logic testing (Context, Indexing, Search) (16 tests)
-   **MCP Protocol**: Protocol compliance and message handling (15 tests)
-   **Integration**: End-to-end functionality testing (11 tests)

### Claude Context Compatibility ✅

**v0.0.3 implements all core Claude Context features:**

| Feature | Status | Implementation |
|---------|--------|----------------|
| **index_codebase** | ✅ Complete | AST chunking, custom extensions, ignore patterns |
| **search_code** | ✅ Complete | Hybrid BM25 + vector search, extension filtering |
| **clear_index** | ✅ Complete | Professional cleanup and state management |
| **get_indexing_status** | ✅ Complete | Real-time status with change detection |
| **Incremental Sync** | ✅ Complete | Background cron jobs, change detection |
| **Multi-Provider Support** | ✅ Complete | OpenAI, VoyageAI, Gemini, Ollama, Milvus |
| **Configuration System** | ✅ Complete | convict.js validation, environment variables |
| **Snapshot Management** | ✅ Complete | Keyv persistence, state recovery |
| **Concurrency Control** | ✅ Complete | p-queue, async-Mutex, file locks |

### CI/CD

GitHub Actions automatically runs:

-   **Tests**: Multiple Rust versions (stable, beta, MSRV)
-   **Linting**: Code formatting and clippy checks
-   **Security**: Dependency vulnerability scanning
-   **Coverage**: Code coverage reporting
-   **Build**: Cross-platform binary builds

## 🤝 Contributing

See [**CONTRIBUTING.md**](CONTRIBUTING.md) for detailed contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
