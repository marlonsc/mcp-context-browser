# Core Module

**Source**: `crates/mcb-domain/src/` (types, ports) and `crates/mcb-infrastructure/src/` (utilities)

Foundational types, traits, and utilities used throughout the system.

## Overview

The core module functionality is split across Clean Architecture layers:

-   **Domain types** (`crates/mcb-domain/src/types.rs`): Embedding, CodeChunk, SearchResult, Language
-   **Port traits** (`crates/mcb-application/src/ports/`): 20+ interfaces including EmbeddingProvider, VectorStoreProvider, HybridSearchProvider, LanguageChunkingProvider, EventBusProvider, SyncCoordinator, SnapshotProvider, and service interfaces
-   **Infrastructure utilities** (`crates/mcb-infrastructure/src/`): auth, cache, crypto, health, logging

## Submodules

### Types (`types.rs`)

Core data structures for code intelligence.

-   `Embedding` - Vector representation of text/code
-   `CodeChunk` - Parsed code segment with metadata
-   `SearchResult` - Ranked search item with score
-   `Language` - Supported programming languages

### Error Handling (`error.rs`)

Comprehensive error types with `thiserror`.

-   `Error` - Main error enum with variants
-   `Result<T>` - Type alias for `Result<T, Error>`

### Authentication (`auth/`)

JWT-based identity and access management (in mcb-server).

-   `AuthService` - Token validation and generation
-   `Claims` - JWT payload structure
-   `Permission` - Authorization controls

### Caching (`cache/`)

Multi-level caching with TTL and size limits (in mcb-infrastructure).

-   `CacheManager` - Main cache interface
-   Configurable TTL and eviction policies

### Crypto (`crypto/`)

Encryption utilities (in mcb-infrastructure).

-   AES-GCM encryption support
-   Hash computation utilities

### Health (`health/`)

Health check infrastructure (in mcb-infrastructure).

-   Component health monitoring
-   Readiness and liveness checks

## Key Exports

```rust
// Domain types (from mcb-domain)
pub use types::{Embedding, CodeChunk, SearchResult, Language};
pub use error::{Error, Result};

// Infrastructure (from mcb-infrastructure)
pub use config::{AppConfig, ServerConfig, AuthConfig};
pub use logging::{init_logging, init_json_logging};
```

## File Structure (Clean Architecture)

```text
crates/mcb-domain/src/
├── types.rs              # Domain types (Embedding, CodeChunk, etc.)
├── error.rs              # Domain error types
├── entities/             # Domain entities
├── value_objects/        # Value objects
├── ports/                # Port traits (interfaces)
│   ├── providers/        # Provider port traits
│   ├── infrastructure/   # Infrastructure port traits
│   └── admin.rs          # Admin service interfaces
└── repositories/         # Repository port traits

crates/mcb-infrastructure/src/
├── config/               # Configuration management
├── cache/                # Caching infrastructure
├── crypto/               # Encryption utilities
├── health/               # Health checks
├── logging.rs            # Structured logging
└── adapters/             # Null adapters for testing
```

## Testing

Domain tests are in `crates/mcb-domain/tests/`.
Infrastructure tests are in `crates/mcb-infrastructure/tests/`.

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Domain**: [domain.md](./domain.md)
-   **Infrastructure**: [infrastructure.md](./infrastructure.md)
-   **Services**: [services.md](./services.md) (uses core types)
-   **Providers**: [providers.md](./providers.md) (implements traits)
-   **Server**: [server.md](./server.md) (uses auth/rate limiting)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
