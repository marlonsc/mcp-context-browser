//! Infrastructure Provider Interfaces
//!
//! Defines the port interfaces for infrastructure services that are used
//! by the application layer. These are cross-cutting concerns that support
//! business logic operations.

use crate::domain::error::Result;
use crate::domain::types::SyncBatch;
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
#[async_trait]
pub trait SnapshotProvider: Interface + Send + Sync {
    /// Get files that need processing (added or modified since last snapshot)
    async fn get_changed_files(&self, root_path: &Path) -> Result<Vec<String>>;
}
