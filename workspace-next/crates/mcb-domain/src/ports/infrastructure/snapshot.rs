//! Infrastructure Provider Interfaces
//!
//! Defines the port interfaces for infrastructure services that are used
//! by the application layer. These are cross-cutting concerns that support
//! business logic operations.

use crate::error::Result;
use crate::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use crate::value_objects::config::SyncBatch;
use async_trait::async_trait;
use shaku::Interface;
use std::path::Path;
use std::time::Duration;

// ============================================================================
// Sync Provider Interface
// ============================================================================

/// Sync Provider Interface
///
/// Defines the contract for codebase synchronization operations.
#[async_trait]
pub trait SyncProvider: Interface + Send + Sync {
    /// Check if codebase should be debounced (synced too recently)
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;

    /// Update last sync time for a codebase
    async fn update_last_sync(&self, codebase_path: &Path);

    /// Acquire a synchronization slot in the queue
    async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>>;

    /// Release a synchronization slot in the queue
    async fn release_sync_slot(&self, codebase_path: &Path, batch: SyncBatch) -> Result<()>;

    /// Get list of files that have changed since last sync
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;

    /// Get sync interval as Duration
    fn sync_interval(&self) -> Duration;

    /// Get debounce interval as Duration
    fn debounce_interval(&self) -> Duration;
}

// ============================================================================
// Snapshot Provider Interface
// ============================================================================

/// Snapshot Provider Interface
///
/// Defines the contract for codebase snapshot and change tracking operations.
/// Snapshots capture the state of files (paths, sizes, modification times, hashes)
/// to detect what has changed between indexing runs.
#[async_trait]
pub trait SnapshotProvider: Interface + Send + Sync {
    /// Create a new snapshot for a codebase
    ///
    /// Traverses the codebase at `root_path`, computes file hashes, and creates
    /// a snapshot representing the current state. The snapshot is automatically
    /// saved to persistent storage.
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot>;

    /// Load an existing snapshot for a codebase
    ///
    /// Retrieves the most recent snapshot for the given codebase path.
    async fn load_snapshot(&self, root_path: &Path) -> Result<Option<CodebaseSnapshot>>;

    /// Compare two snapshots to find changes
    ///
    /// Analyzes the differences between an old and new snapshot to determine
    /// which files were added, modified, removed, or unchanged.
    async fn compare_snapshots(
        &self,
        old_snapshot: &CodebaseSnapshot,
        new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges>;

    /// Get files that need processing (added or modified since last snapshot)
    ///
    /// Convenience method that creates a new snapshot, compares with the previous
    /// one, and returns the list of files that need to be re-indexed.
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}
