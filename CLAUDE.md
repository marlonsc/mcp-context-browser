# Claude.md

This file provides guidance to Claude Code (Claude.ai/code) when working with code in this repository.

## Project Overview

MCP Context Browser is a high-performance MCP server for semantic code search using vector embeddings. Version 0.1.4 released.

## Commands

Always use `make` commands, never raw Cargo or git:

```bash
# Build
make build          # Debug build
make build-release  # Release build

# Test
make test           # All tests (950+)
make test-unit      # Unit tests only
make test-doc       # Doctests only

# Quality
make fmt            # Format (Rust + Markdown)
make lint           # Rust: fmt check + clippy (FIX=1, CI_MODE=1)
make docs-lint      # Markdown lint (FIX=1 to auto-fix)
make quality        # Full check: fmt + lint + test
make validate       # Architecture validation (mcb-validate)

# Git
make sync           # Add + commit + push (never use raw git commit)

# Single test
cargo test test_name -- --nocapture
```

## Architecture (8 Crates - Clean Architecture)

```
crates/
├── mcb/                 # Facade crate (re-exports public API)
├── mcb-domain/          # Layer 1: Entities, ports (traits), errors
├── mcb-application/     # Layer 2: Use cases, services, registry (linkme slices)
├── mcb-providers/       # Layer 3: Provider implementations (auto-register via linkme)
├── mcb-infrastructure/  # Layer 4: DI handles, config (Figment), health, logging
├── mcb-server/          # Layer 5: MCP protocol, handlers, transport
└── mcb-validate/        # Dev tooling: architecture validation
```

**Dependency Direction**:

```
mcb-server → mcb-infrastructure → mcb-application → mcb-domain
                    ↓                    ↑
              mcb-providers ─────────────┘
```

Note: `mcb-providers` depends on both `mcb-domain` (port traits) and `mcb-application` (registry slices for auto-registration via linkme).

### Key Crate Contents

**mcb-domain**: Port traits (`EmbeddingProvider`, `VectorStoreProvider`, `CacheProvider`, `LanguageChunkingProvider`), domain entities (`CodeChunk`, `Embedding`, `SearchResult`), domain errors with `thiserror`.

**mcb-application**: Services (`ContextService`, `SearchService`, `IndexingService`), registry system (linkme distributed slices for provider auto-registration), admin ports (`IndexingOperationsInterface`, `PerformanceMetricsInterface`), infrastructure ports (`EventBusProvider`, `AuthServiceInterface`).

**mcb-providers**: Provider implementations. Auto-register via `#[linkme::distributed_slice]` into mcb-application registry.

**mcb-infrastructure**: Handle-based DI (ADR-024), Figment config loading (ADR-025), provider handles (`EmbeddingProviderHandle`, etc.), resolvers, admin services for runtime switching.

**mcb-server**: MCP tool handlers (`index_codebase`, `search_code`, `get_indexing_status`, `clear_index`), stdio/HTTP transport.

**mcb-validate**: Architecture validation tooling. AST parsers (Tree-sitter), rule engines.

## Code Standards

1.  **No unwrap/expect** - Use `?` operator with proper error types
2.  **No hardcoded fallbacks** - Require configuration, fail fast if missing
3.  **File size < 500 lines** - Split large files
4.  **Trait-based DI** - Use `Arc<dyn Trait>`, not `Arc<ConcreteType>`
5.  **Async-first** - All I/O operations async with Tokio
6.  **Error handling** - Custom types with `thiserror`:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Provider error: {message}")]
    Provider { message: String },
}
```

## DI Pattern (ADR-024 → ADR-029: Hexagonal Architecture with dill)

Handle-based pattern with dill IoC Container and linkme registry:

```rust
// dill Catalog - IoC Container (mcb-infrastructure/src/di/catalog.rs)
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    CatalogBuilder::new()
        .add_value(config)
        .add_value(embedding_provider)    // From linkme registry
        .add_value(embedding_handle)      // RwLock wrapper
        .add_value(embedding_admin)       // Runtime switching
        .build()
}

// Service retrieval
pub fn get_service<T: ?Sized + Send + Sync>(catalog: &Catalog) -> Result<Arc<T>> {
    catalog.get_one::<T>()
}

// Provider Handle - RwLock wrapper for runtime switching
pub struct EmbeddingProviderHandle {
    inner: RwLock<Arc<dyn EmbeddingProvider>>,
}

// AppContext - composition root (wraps Catalog)
pub struct AppContext {
    embedding_handle: Arc<EmbeddingProviderHandle>,
    embedding_admin: Arc<dyn EmbeddingAdminInterface>,
    // ... other handles and services via trait objects
}
```

**Key Architecture Rules** (enforced by mcb-validate CA007/CA008):

-   Ports defined in `mcb-domain/src/ports/providers/`
-   Application imports ports from mcb-domain (not duplicated)
-   Infrastructure uses `Arc<dyn Trait>` for admin services

## Provider Registration (ADR-023: linkme)

Providers auto-register via linkme distributed slices:

```rust
// In mcb-application: declare slice
#[linkme::distributed_slice]
pub static EMBEDDING_PROVIDERS: [EmbeddingProviderEntry] = [..];

// In mcb-providers: register provider
#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static OLLAMA_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "ollama",
    description: "Ollama local embedding provider",
    factory: ollama_factory,  // Function pointer, NOT closure
};
```

## Configuration (ADR-025: Figment)

Use Figment for configuration loading:

```rust
use figment::{Figment, providers::{Toml, Env}};

let figment = Figment::new()
    .merge(Toml::file("config/default.toml"))
    .merge(Toml::file(config_path))
    .merge(Env::prefixed("MCB_").split("_"));

let config: AppConfig = figment.extract()?;
```

## Quality Gates

Before any commit:

-   `make test` - 0 failures
-   `make lint` - Rust clean; `make docs-lint` - Markdown clean
-   `make validate` - 0 architecture violations
-   No new `unwrap/expect`
-   No hardcoded fallback values

## Supported Providers

**Embedding**: OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null

**Vector Store**: Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null

**Cache**: Moka, Redis, Null

**Languages (AST)**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin

## Documentation

-   ADRs: `docs/adr/README.md` (31 architectural decisions)
-   Architecture: `docs/architecture/ARCHITECTURE.md`
-   Migration: `docs/migration/FROM_CLAUDE_CONTEXT.md`
