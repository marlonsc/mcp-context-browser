# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.0.3-blue)](https://github.com/marlonsc/mcp-context-browser/releases)
[![Claude Context Compatible](https://img.shields.io/badge/Claude%20Context-Compatible-green)](IMPLEMENTATION_GUIDE_v0.0.3.md)
[![CI](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml/badge.svg)](https://github.com/marlonsc/mcp-context-browser/actions/workflows/ci.yml)

**Model Context Protocol Server** - Provides semantic code search and analysis capabilities to AI assistants through a standardized MCP interface.

**🎯 v0.0.3: Full Claude Context Compatibility** - Implements all core functionality from the official Claude Context MCP server, including professional indexing, incremental sync, and multi-provider support.

## 🎯 Current Capabilities (v0.0.3)

### 🏆 Enterprise-Grade Features (v0.0.3)
- **🧠 Semantic Code Search**: Hybrid BM25 + vector search using natural language queries
- **🔄 Incremental Sync**: Automatic background synchronization with change detection
- **💾 Persistent State**: Professional snapshot management with Keyv storage
- **🎯 Advanced Indexing**: AST-based code chunking with custom extensions and ignore patterns
- **🔒 Concurrency Control**: p-queue coordination with async-mutex and file locks
- **🔧 Multi-Provider Support**: OpenAI, VoyageAI, Gemini, Ollama embeddings + Milvus vector storage
- **⚙️ Advanced Configuration**: convict.js schema validation with environment variables
- **📊 Professional Monitoring**: Comprehensive status tracking and error recovery

### Core MCP Tools
- **`index_codebase`**: Index entire codebases with AST chunking and custom configurations
- **`search_code`**: Natural language semantic search with extension filtering
- **`get_indexing_status`**: Real-time status monitoring with change detection
- **`clear_index`**: Professional index management and cleanup

### Architecture
- **🏗️ Enterprise Architecture**: SOLID principles with dependency injection
- **🔌 Provider Pattern**: Extensible system for embeddings and vector storage
- **⚡ Async-First Design**: Tokio runtime with streams and concurrent processing
- **🛡️ Robust Error Handling**: Custom error types with detailed diagnostics
- **🔄 Background Services**: Cron-based incremental updates and synchronization
- **💾 Persistent Storage**: Keyv-based state management with automatic recovery

## 📋 Documentation

- [**IMPLEMENTATION_GUIDE_v0.0.3.md**](IMPLEMENTATION_GUIDE_v0.0.3.md) - Complete v0.0.3 implementation guide
- [**ARCHITECTURE.md**](ARCHITECTURE.md) - Technical architecture and design
- [**ROADMAP.md**](ROADMAP.md) - Development roadmap and milestones
- [**DEPLOYMENT.md**](DEPLOYMENT.md) - Deployment guides and configurations
- [**CONTRIBUTING.md**](CONTRIBUTING.md) - Contribution guidelines

## 🚀 Quick Start

```bash
# Install Rust and clone
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/marlonsc/mcp-context-browser.git
cd mcp-context-browser

# Run development setup
make setup
make dev
```

## 🧪 Testing & Quality

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
- **Core Types**: Data structure validation and serialization (18 tests)
- **Services**: Business logic testing (Context, Indexing, Search) (16 tests)
- **MCP Protocol**: Protocol compliance and message handling (15 tests)
- **Integration**: End-to-end functionality testing (11 tests)

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
| **Concurrency Control** | ✅ Complete | p-queue, async-mutex, file locks |

### CI/CD
GitHub Actions automatically runs:
- **Tests**: Multiple Rust versions (stable, beta, MSRV)
- **Linting**: Code formatting and clippy checks
- **Security**: Dependency vulnerability scanning
- **Coverage**: Code coverage reporting
- **Build**: Cross-platform binary builds

## 🤝 Contributing

See [**CONTRIBUTING.md**](CONTRIBUTING.md) for detailed contribution guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.