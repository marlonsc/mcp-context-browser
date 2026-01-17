# snapshot Module

**Note**: Snapshot functionality is defined as a port trait in v0.1.1.

**Trait**: `crates/mcb-application/src/ports/infrastructure/snapshot.rs`
**Null Adapter**: `crates/mcb-infrastructure/src/adapters/infrastructure/snapshot.rs`

## Overview

Snapshot management for incremental codebase tracking. Tracks file changes using SHA256 hashing for efficient incremental sync. Avoids reprocessing unchanged files during codebase indexing.

## Components

### SnapshotProvider Trait (`mcb-application`)

Port definition for snapshot operations:

```rust
#[async_trait]
pub trait SnapshotProvider: Send + Sync + shaku::Interface {
    async fn capture(&self, path: &Path) -> Result<CodebaseSnapshot>;
    async fn compare(&self, old: &CodebaseSnapshot, new: &CodebaseSnapshot) -> Result<SnapshotChanges>;
    async fn store(&self, snapshot: &CodebaseSnapshot) -> Result<()>;
    async fn load(&self, id: &str) -> Result<Option<CodebaseSnapshot>>;
}
```

### NullSnapshotProvider (`mcb-infrastructure`)

No-op implementation for testing and DI.

## File Structure

```text
crates/mcb-application/src/ports/infrastructure/
└── snapshot.rs              # SnapshotProvider trait, StateStoreProvider trait

crates/mcb-infrastructure/src/adapters/infrastructure/
└── snapshot.rs              # NullSnapshotProvider, NullStateStoreProvider
```

## Domain Types

Related types in `mcb-domain`:

-   `CodebaseSnapshot` - Point-in-time codebase state
-   `FileSnapshot` - Individual file state with hash
-   `SnapshotChanges` - Delta between snapshots

## Key Exports

```rust
// Trait (from mcb-application)
pub use ports::infrastructure::snapshot::{SnapshotProvider, StateStoreProvider};

// Null implementation (from mcb-infrastructure)
pub use adapters::infrastructure::snapshot::{NullSnapshotProvider, NullStateStoreProvider};
```

## Cross-References

-   **Domain**: [domain.md](./domain.md) (trait definition)
-   **Infrastructure**: [infrastructure.md](./infrastructure.md) (null adapter)
-   **Sync**: [sync.md](./sync.md) (uses snapshots)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-17 - Reflects modular crate architecture (v0.1.1)*
