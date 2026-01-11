# MCP Context Browser - Claude Code Guide

MCP server for semantic code search using vector embeddings. **v0.1.0 production-ready.**

## Quick Reference

```bash
# Development
make build          # Compile
make test           # Run 549+ tests
make lint           # Clippy
make fmt            # Format
make quality        # Full check (fmt + lint + test + audit)

# Git (ALWAYS use make, never raw git)
make git-force-all  # Add + commit + push

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
| `git commit` | `make git-force-all` |

### Code Standards

1. **No unwrap/expect** - Use `?` operator with proper error types
2. **File size < 500 lines** - Split large files
3. **Trait-based DI** - Use `Arc<dyn Trait>`, not `Arc<ConcreteType>`
4. **Async-first** - All I/O operations async with Tokio
5. **Error handling** - Custom types with `thiserror`, context with `anyhow`

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

## Directory Structure

```text
src/
├── domain/         # Domain types, validation, error, ports (traits)
├── application/    # Business services (indexing, search, context)
├── adapters/       # Infrastructure implementations (providers, db, repositories)
├── infrastructure/ # Shared systems (cache, auth, config, metrics, events)
├── server/         # MCP protocol implementation
├── chunking/       # Code chunking logic (14 language processors)
├── daemon/         # Background processes
├── snapshot/       # Snapshot management
└── sync/           # Codebase synchronization
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

- JWT authentication for API access
- Rate limiting on all endpoints
- AES-GCM encryption at rest

## Current Version: v0.1.0

**First stable release** - Drop-in replacement for claude-context:

- 14 programming languages with AST parsing
- 6 embedding providers
- 6 vector stores
- 549+ tests
- HTTP transport foundation
- Systemd integration

## Next Version: v0.2.0 (Planned)

**Git-Aware Semantic Indexing** - See [ADR-008](docs/adr/008-git-aware-semantic-indexing-v0.2.0.md):

- Project-relative indexing (portable)
- Multi-branch indexing
- Commit history search
- Submodule support
- Monorepo detection
- Impact analysis

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Tests fail | `make validate` to diagnose |
| Build breaks | Check `Cargo.toml` deps |
| Lint errors | `make fmt` then `make lint` |

## Documentation

- Architecture: `docs/architecture/ARCHITECTURE.md`
- ADRs: `docs/adr/README.md`
- Roadmap: `docs/developer/ROADMAP.md`
- Version History: `docs/VERSION_HISTORY.md`
- Detailed project info: `.claude/rules/custom/project.md`

## Quality Gates

Before any commit:

- [ ] `make test` - 0 failures
- [ ] `make lint` - clean output
- [ ] `make fmt` - no changes
- [ ] No new `unwrap/expect`
- [ ] Files < 500 lines
