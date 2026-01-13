# MCP Context Browser - Claude Code Guide

MCP server for semantic code search using vector embeddings. **v0.1.0 production-ready.**

## Quick Reference

```bash
# Development
make build          # Compile
make test           # Run 790+ tests
make lint           # Clippy
make fmt            # Format
make quality        # Full check (fmt + lint + test + audit)

# Git (ALWAYS use make, never raw git)
make sync           # Add + commit + push

# Release
make release        # test + build-release + package
```

## Project Rules

### Commands (MANDATORY)

Use `make` commands, never raw Cargo/git:

| Instead of | Use |
|------------|-----|
| `cargo test` | `make test` |
| `cargo build` | `make build` |
| `cargo clippy` | `make lint` |
| `git commit` | `make sync` or `make commit` |

### Code Standards

1.  **No unwrap/expect** - Use `?` operator with proper error types
2.  **File size < 500 lines** - Split large files
3.  **Trait-based DI** - Use `Arc<dyn Trait>`, not `Arc<ConcreteType>`
4.  **Async-first** - All I/O operations async with Tokio
5.  **Error handling** - Custom types with `thiserror`, context with `anyhow`

### Architecture Patterns

```rust
// Provider pattern (REQUIRED)
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
}

// Constructor injection
pub struct Service {
    provider: Arc<dyn EmbeddingProvider>,
}
```

## Directory Structure (Clean Architecture)

```text
src/
├── domain/            # Core business logic, ports (traits), types
│   ├── ports/         # 14 port traits (interfaces)
│   ├── chunking/      # AST-based code chunking (12 languages)
│   ├── types.rs       # Domain types (CodeChunk, Embedding, etc.)
│   └── error.rs       # Domain errors
├── application/       # Business services
│   ├── context.rs     # ContextService
│   ├── search.rs      # SearchService
│   └── indexing/      # IndexingService, ChunkingOrchestrator
├── adapters/          # External integrations
│   ├── providers/     # Embedding (6), VectorStore (6), Routing
│   ├── hybrid_search/ # BM25 + semantic search
│   └── repository/    # Chunk and search repositories
├── infrastructure/    # Shared technical services
│   ├── di/            # Shaku dependency injection
│   ├── auth/          # JWT, rate limiting
│   ├── config/        # Configuration management
│   ├── events/        # Event bus (Tokio, NATS)
│   ├── sync/          # File synchronization
│   ├── snapshot/      # Change tracking
│   └── daemon/        # Background processes
└── server/            # MCP protocol server
    ├── handlers/      # Tool handlers
    └── admin/         # Admin service
```

## Testing

All tests must pass before commit:

```bash
make test           # Must show 0 failures
make lint           # Must be clean
```

Test categories: core_types, services, protocol, integration, providers, routing, security.

## Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Provider error: {message}")]
    Provider { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
```

## Security

-   JWT authentication for API access
-   Rate limiting on all endpoints
-   AES-GCM encryption at rest

## Current Version: v0.1.0

**First stable release** - Drop-in replacement for Claude-context:

-   12 languages with AST parsing (Rust, Python, JS/TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
-   6 embedding providers (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null)
-   6 vector stores (Milvus, EdgeVec, In-Memory, Filesystem, Encrypted, Null)
-   790+ tests (100% pass rate)
-   HTTP transport foundation
-   Systemd integration

## Next Version: v0.2.0 (Planned)

**Git-Aware Indexing + Persistent Session Memory**:

**Git Integration** - See [ADR-008](docs/adr/008-git-aware-semantic-indexing-v0.2.0.md):

-   Project-relative indexing (portable)
-   Multi-branch indexing
-   Commit history search
-   Submodule support
-   Monorepo detection
-   Impact analysis

**Session Memory** - See [ADR-009](docs/adr/009-persistent-session-memory-v0.2.0.md):

-   Cross-session observation storage
-   Session summaries and tracking
-   Hybrid search (BM25 + vector)
-   Progressive disclosure (3-layer workflow)
-   Context injection for SessionStart hooks
-   Git-tagged memory entries

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Tests fail | `make validate` to diagnose |
| Build breaks | Check `Cargo.toml` deps |
| Lint errors | `make fmt` then `make lint` |

## Documentation

-   Architecture: `docs/architecture/ARCHITECTURE.md`
-   ADRs: `docs/adr/README.md`
-   Roadmap: `docs/developer/ROADMAP.md`
-   Version History: `docs/VERSION_HISTORY.md`
-   Detailed project info: `.claude/rules/custom/project.md`

## Quality Gates

Before any commit:

-   [ ] `make test` - 0 failures
-   [ ] `make lint` - clean output
-   [ ] `make fmt` - no changes
-   [ ] No new `unwrap/expect`
-   [ ] Files < 500 lines
