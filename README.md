# MCP Context Browser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-orange)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05-blue)](https://modelcontextprotocol.io/)
[![Version](https://img.shields.io/badge/version-0.1.4-blue)](https://github.com/marlonsc/mcb/releases/tag/v0.1.4)

**High-performance MCP server for semantic code search using vector embeddings**

## Overview

MCP Context Browser is a Model Context Protocol (MCP) server that provides semantic code search capabilities using vector embeddings. Transform natural language queries into code search across indexed codebases, enabling intelligent code discovery and analysis. Built with Clean Architecture principles in Rust with comprehensive provider support.

**Current Version**: 0.1.4

See [`CLAUDE.md`](./CLAUDE.md) for development guide and [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md) for complete architecture documentation.

## Installation

### From source (recommended)

Prerequisites: Rust toolchain (1.89+), `make`, and a POSIX shell.

```bash
# Build release binary
make build-release

# Install as a user systemd service (installs to ~/.claude/servers/claude-context-mcp)
make install
```

For a faster dev install, use `make install-debug`. If you prefer to run without systemd, build with `make build-release` and run `target/release/mcb` directly.

### Main Features

-   **Semantic Code Search**: Natural language queries → code discovery using vector embeddings
-   **Clean Architecture**: 8 crates (domain, application, infrastructure, providers, server, validate) per Clean Architecture layers
-   **Provider Ecosystem**: 6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null), 5 vector stores (In-Memory, Encrypted, Filesystem, Milvus, EdgeVec, Null)
-   **Multi-Language Support**: AST-based parsing for 14 languages (Rust, Python, JS/TS, Go, Java, C/C++/C#, Ruby, PHP, Swift, Kotlin)
-   **Architecture Validation**: mcb-validate crate, Phases 1–7 (CA001–CA009, metrics, duplication); 1636+ tests project-wide
-   **Linkme Provider Registration**: Compile-time provider discovery (zero runtime overhead)

## Architecture

MCP Context Browser follows **Clean Architecture** with strict layer separation across 8 Cargo workspace crates:

```
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services orchestration
├── mcb-providers/       # Layer 3: Provider implementations (embedding, vector stores)
├── mcb-infrastructure/  # Layer 4: DI, config, cache, crypto, health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation (Phases 1–7)
```

**Dependency Direction** (inward only):

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓
              mcb-providers
```

### Key Architectural Decisions

-   **ADR-001**: Modular Crates Architecture – 8 crates, separation of concerns
-   **ADR-002**: Async-First Architecture – Tokio throughout
-   **ADR-029**: Hexagonal Architecture with dill – DI, handles, linkme registry (replaces Shaku)
-   **ADR-013**: Clean Architecture Crate Separation – Port/Adapter pattern
-   **ADR-023**: Inventory to Linkme Migration – Compile-time provider registration

See [`docs/adr/`](./docs/adr/) for complete Architecture Decision Records and [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md) for detailed architecture documentation.

## Usage

### Requirements

-   Rust 1.89+ (edition 2024)
-   For embedding providers: API keys (OpenAI, VoyageAI, Gemini) or local Ollama instance
-   For vector stores: Milvus/Qdrant instance (or use in-memory for development)

### Build and Run

```bash
# Build
make build-release

# Run tests
make test

# Validate architecture
make validate
```

### MCP Tools

The server exposes 4 MCP tools for semantic code search:

| Tool | Purpose |
|------|---------|
| `index_codebase` | Index a codebase directory with semantic embeddings |
| `search_code` | Search indexed code using natural language queries |
| `get_indexing_status` | Check indexing status and collection stats |
| `clear_index` | Clear a collection's indexed data |

### Configuration

Configure via environment variables (see [`CLAUDE.md`](./CLAUDE.md) for details):

```bash
# Embedding provider (openai, voyageai, ollama, gemini, fastembed)
export EMBEDDING_PROVIDER=ollama
export OLLAMA_MODEL=nomic-embed-text

# Vector store (in-memory, encrypted, null)
export VECTOR_STORE_PROVIDER=in-memory
```

See [`docs/CONFIGURATION.md`](./docs/CONFIGURATION.md) for complete configuration guide.

## Development

### Commands

Always use `make` commands (see [`CLAUDE.md`](./CLAUDE.md)):

```bash
make build          # Debug build
make build-release  # Release build
make test           # All tests (950+)
make quality        # Full check: fmt + lint + test
make validate       # Architecture validation
```

### Quality Gates

-   All tests pass (`make test`)
-   Clean Rust lint (`make lint`); clean Markdown lint (`make docs-lint`)
-   Zero architecture violations (`make validate`)
-   No new `unwrap/expect` in code

See [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md) for contribution guidelines.

## Testing

1636+ tests covering all layers:

```bash
make test           # All tests
make test-unit      # Unit tests only
cargo test test_name -- --nocapture  # Single test
```

Test organization:

-   **Domain layer**: Entity and value object tests
-   **Application layer**: Service and use case tests
-   **Infrastructure layer**: DI, config, cache tests
-   **Providers**: Embedding and vector store provider tests
-   **mcb-validate**: Architecture validation (Phases 1–7, 1636+ tests)

See [`docs/INTEGRATION_TESTS.md`](./docs/INTEGRATION_TESTS.md) for testing documentation.

## Documentation

-   **Quick Start**: [`docs/user-guide/QUICKSTART.md`](./docs/user-guide/QUICKSTART.md)
-   **Architecture**: [`docs/architecture/ARCHITECTURE.md`](./docs/architecture/ARCHITECTURE.md)
-   **Development**: [`CLAUDE.md`](./CLAUDE.md) and [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md)
-   **Roadmap**: [`docs/developer/ROADMAP.md`](./docs/developer/ROADMAP.md)
-   **Changelog**: [`docs/operations/CHANGELOG.md`](./docs/operations/CHANGELOG.md)
-   **ADRs**: [`docs/adr/`](./docs/adr/) - Architecture Decision Records
-   **Migration**: [`docs/migration/FROM_CLAUDE_CONTEXT.md`](./docs/migration/FROM_CLAUDE_CONTEXT.md)
-   **API (docs.rs)**: [mcb](https://docs.rs/mcb) (when published)

## Contributing

Contributions welcome! See [`docs/developer/CONTRIBUTING.md`](./docs/developer/CONTRIBUTING.md) for guidelines.

Quality requirements:

-   Follow Clean Architecture principles
-   Add tests for new features
-   Update ADRs for architectural changes
-   Run `make quality` before committing

## License

MIT Licensed - Open source and free for commercial and personal use.

---

**Last Updated**: 2026-01-28
