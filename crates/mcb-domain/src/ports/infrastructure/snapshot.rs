//! Infrastructure Provider Interfaces
//!
//! Defines the port interfaces for infrastructure services that are used
//! by the application layer. These are cross-cutting concerns that support
//! business logic operations.

use crate::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use crate::error::Result;
use crate::value_objects::config::SyncBatch;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

// ============================================================================
// Sync Provider Interface
// ============================================================================

/// Sync Provider Interface
///
/// Defines the contract for codebase synchronization operations.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::infrastructure::snapshot::SyncProvider;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// async fn sync_codebase(sync: Arc<dyn SyncProvider>, codebase_path: &Path) -> mcb_domain::Result<()> {
///     // Check if sync should be debounced (too recent)
///     if !sync.should_debounce(codebase_path).await? {
///         let changed = sync.get_changed_files(codebase_path).await?;
///         println!("Changed files: {:?}", changed);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SyncProvider: Send + Sync {
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
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::infrastructure::snapshot::SnapshotProvider;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// async fn snapshot_codebase(snapshot: Arc<dyn SnapshotProvider>, project_path: &Path) -> mcb_domain::Result<()> {
///     // Create a new snapshot of the codebase
///     let new_snapshot = snapshot.create_snapshot(project_path).await?;
///     println!("Created snapshot with {} files", new_snapshot.files.len());
///
///     // Shortcut: get files needing re-indexing
///     let changed_files = snapshot.get_changed_files(project_path).await?;
///     println!("Changed files: {:?}", changed_files);
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
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
