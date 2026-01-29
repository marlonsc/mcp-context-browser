# repository Module

**Source**: `crates/mcb-infrastructure/src/adapters/repository/`
**Traits**: `crates/mcb-domain/src/repositories/`
**Crate**: `mcb-infrastructure`
**Files**: 3
**Lines of Code**: ~400

## Overview

Repository pattern implementation for data access abstraction. Provides repository interfaces and null implementations following the Repository pattern to separate data access logic from business logic.

## Components

### Repository Traits (`mcb-domain`)

Port definitions for repositories:

-   `ChunkRepository` - Code chunk persistence operations
-   `SearchRepository` - Search Result retrieval operations

### Null Implementations (`mcb-infrastructure`)

Test/development implementations:

-   `NullChunkRepository` - No-op chunk repository
-   `NullSearchRepository` - No-op search repository

## File Structure

```text
crates/mcb-domain/src/repositories/
├── chunk_repository.rs       # ChunkRepository trait
├── search_repository.rs      # SearchRepository trait
└── mod.rs

crates/mcb-infrastructure/src/adapters/repository/
├── chunk_repository.rs       # NullChunkRepository
├── search_repository.rs      # NullSearchRepository
└── mod.rs
```

## Repository Pattern

```rust
// Port trait (in mcb-domain); DI via dill (ADR-029)
#[async_trait]
pub trait ChunkRepository: Send + Sync {
    async fn store(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    async fn get(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>>;
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;
}

// Null implementation (in mcb-infrastructure)
pub struct NullChunkRepository;

#[async_trait]
impl ChunkRepository for NullChunkRepository {
    async fn store(&self, _: &str, _: &[CodeChunk]) -> Result<()> { Ok(()) }
    async fn get(&self, _: &str, _: &str) -> Result<Option<CodeChunk>> { Ok(None) }
    async fn delete(&self, _: &str, _: &str) -> Result<()> { Ok(()) }
}
```

## Key Exports

```rust
// Traits (from mcb-domain)
pub use repositories::{ChunkRepository, SearchRepository};

// Null implementations (from mcb-infrastructure)
pub use adapters::repository::{NullChunkRepository, NullSearchRepository};
```

## Cross-References

-   **Domain**: [domain.md](./domain.md) (trait definitions)
-   **Infrastructure**: [infrastructure.md](./infrastructure.md) (null implementations)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.1.2)*
