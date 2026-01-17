# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

MCP Context Browser is a high-performance MCP server for semantic code search using vector embeddings. Version 0.1.1 is production-ready.

## Commands

Always use `make` commands, never raw Cargo or git:

```bash
# Build
make build          # Debug build
make build-release  # Release build

# Test
make test           # All tests (790+)
make test-unit      # Unit tests only
make test-doc       # Doctests only

# Quality
make fmt            # Format (Rust + Markdown)
make lint           # Clippy + Markdown lint
make quality        # Full check: fmt + lint + test
make validate       # Architecture validation (mcb-validate)

# Git
make sync           # Add + commit + push (never use raw git commit)

# Single test
cargo test test_name -- --nocapture
```

## Architecture (7 Crates - Clean Architecture)

```
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services orchestration
├── mcb-providers/       # Layer 3: Provider implementations (embedding, vector stores)
├── mcb-infrastructure/  # Layer 4: DI, config, cache, crypto, health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation rules
```

**Dependency Direction** (inward only):

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓
              mcb-providers
```

### Key Crate Contents

**mcb-domain**: Port traits (`EmbeddingProvider`, `VectorStore`, `CacheProvider`), domain entities (`CodeChunk`, `Embedding`, `SearchResult`), domain errors with `thiserror`.

**mcb-application**: Services (`ContextService`, `SearchService`, `IndexingService`), `ChunkingOrchestrator` for batch processing.

**mcb-infrastructure**: DI container (`InfrastructureComponents`, `FullContainer`), cache providers (Moka, Redis, Null), config loading, AES-GCM crypto, health checks, structured logging.

**mcb-server**: MCP tool handlers (`index_codebase`, `search_code`, `get_indexing_status`, `clear_index`), stdio transport.

## Code Standards

1.  **No unwrap/expect** - Use `?` operator with proper error types
2.  **File size < 500 lines** - Split large files
3.  **Trait-based DI** - Use `Arc<dyn Trait>`, not `Arc<ConcreteType>`
4.  **Async-first** - All I/O operations async with Tokio
5.  **Error handling** - Custom types with `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Provider error: {message}")]
    Provider { message: String },
}
```

## DI Pattern

Manual builder pattern (not full Shaku macros):

```rust
pub struct InfrastructureComponents {
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
    pub health: HealthRegistry,
    pub config: AppConfig,
}

pub struct FullContainer {
    pub infrastructure: InfrastructureComponents,
    pub domain_services: DomainServicesContainer,
}
```

## Quality Gates

Before any commit:

-   `make test` - 0 failures
-   `make lint` - clean output
-   `make validate` - 0 architecture violations
-   No new `unwrap/expect`

## Supported Providers

**Embedding**: OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null

**Vector Store**: Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null

**Languages (AST)**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin

## Documentation

-   ADRs: `docs/adr/README.md` (13 architectural decisions)
-   Architecture: `docs/architecture/ARCHITECTURE.md`
-   Migration: `docs/migration/FROM_CLAUDE_CONTEXT.md`
