# sync Module

**Note**: Sync functionality is defined as a port trait in v0.1.1.

**Trait**: `crates/mcb-application/src/ports/infrastructure/sync.rs`
**Null Adapter**: `crates/mcb-infrastructure/src/adapters/infrastructure/sync.rs`

## Overview

File synchronization coordination for incremental indexing. Manages file change detection and coordinates re-indexing of modified files.

## Components

### SyncProvider Trait (`mcb-application`)

Port definition for sync operations:

```rust
#[async_trait]
pub trait SyncProvider: Send + Sync {
    async fn sync(&self, path: &Path) -> Result<SyncResult>;
    async fn get_changes(&self, path: &Path) -> Result<Vec<FileChange>>;
    fn is_file_changed(&self, path: &Path, hash: &str) -> bool;
}
```

### LockProvider Trait (`mcb-application`)

Distributed locking for concurrent sync:

```rust
#[async_trait]
pub trait LockProvider: Send + Sync {
    async fn acquire(&self, key: &str) -> Result<Lock>;
    async fn release(&self, lock: Lock) -> Result<()>;
}
```

### Null Implementations (`mcb-infrastructure`)

-   `NullSyncProvider` - No-op sync provider
-   `NullLockProvider` - No-op lock provider

## File Structure

```text
crates/mcb-application/src/ports/infrastructure/
└── sync.rs                  # SyncProvider, LockProvider traits

crates/mcb-infrastructure/src/adapters/infrastructure/
└── sync.rs                  # NullSyncProvider, NullLockProvider
```

## Key Exports

```rust
// Traits (from mcb-application)
pub use ports::infrastructure::sync::{SyncProvider, LockProvider};

// Null implementations (from mcb-infrastructure)
pub use adapters::infrastructure::sync::{NullSyncProvider, NullLockProvider};
```

## Cross-References

-   **Domain**: [domain.md](./domain.md) (trait definition)
-   **Infrastructure**: [infrastructure.md](./infrastructure.md) (null adapter)
-   **Snapshot**: [snapshot.md](./snapshot.md) (change detection)
-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

*Updated 2026-01-18 - Reflects modular crate architecture (v0.1.2)*
