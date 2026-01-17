//! Sync Coordinator Domain Port
//!
//! Defines the business contract for file synchronization coordination.
//! This abstraction enables services to coordinate sync operations without
//! coupling to specific debouncing, queueing, or file-watching implementations.

use async_trait::async_trait;
use mcb_domain::error::Result;
use shaku::Interface;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for sync operations
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Minimum interval between syncs for the same codebase
    pub debounce_duration: Duration,
    /// Whether to force sync regardless of debounce
    pub force: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_secs(60),
            force: false,
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether the sync was performed (false if skipped due to debounce)
    pub performed: bool,
    /// Number of files that changed
    pub files_changed: usize,
    /// List of changed file paths (relative to root)
    pub changed_files: Vec<String>,
}

impl SyncResult {
    /// Create a result for a skipped sync
    pub fn skipped() -> Self {
        Self {
            performed: false,
            files_changed: 0,
            changed_files: Vec::new(),
        }
    }

    /// Create a result for a completed sync
    pub fn completed(changed_files: Vec<String>) -> Self {
        let files_changed = changed_files.len();
        Self {
            performed: true,
            files_changed,
            changed_files,
        }
    }
}

/// Domain Port for File Synchronization Coordination
///
/// This trait defines the contract for coordinating file synchronization
/// operations. It handles debouncing (preventing excessive syncs), change
/// detection, and cross-process coordination.
///
/// # Example
///
/// ```ignore
/// use mcb_application::ports::infrastructure::sync::{SyncCoordinator, SyncOptions};
/// use std::path::Path;
///
/// async fn sync_codebase(
///     coordinator: &dyn SyncCoordinator,
///     path: &Path,
/// ) -> mcb_domain::Result<bool> {
///     let result = coordinator.sync(path, SyncOptions::default()).await?;
///     Ok(result.performed)
/// }
/// ```
#[async_trait]
pub trait SyncCoordinator: Interface + Send + Sync {
    /// Check if sync should be skipped due to debouncing
    ///
    /// Returns true if the codebase was synced too recently.
    ///
    /// # Arguments
    ///
    /// * `codebase_path` - Root directory of the codebase
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool>;

    /// Perform sync operation for a codebase
    ///
    /// Coordinates the sync operation including:
    /// - Debounce checking (unless force=true)
    /// - Change detection
    /// - Timestamp tracking
    ///
    /// # Arguments
    ///
    /// * `codebase_path` - Root directory of the codebase to sync
    /// * `options` - Sync configuration options
    ///
    /// # Returns
    ///
    /// Result containing sync status and changed files
    async fn sync(&self, codebase_path: &Path, options: SyncOptions) -> Result<SyncResult>;

    /// Get list of files that have changed since last sync
    ///
    /// Scans the codebase for files that have been modified since the last
    /// sync operation.
    ///
    /// # Arguments
    ///
    /// * `codebase_path` - Root directory of the codebase
    ///
    /// # Returns
    ///
    /// List of changed file paths (relative to root)
    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>>;

    /// Update the last sync timestamp for a codebase
    ///
    /// Called after a successful sync to update debounce tracking.
    ///
    /// # Arguments
    ///
    /// * `codebase_path` - Root directory of the codebase
    async fn mark_synced(&self, codebase_path: &Path) -> Result<()>;

    /// Get the number of tracked files
    ///
    /// Returns the number of files currently being tracked for changes.
    fn tracked_file_count(&self) -> usize;
}

/// Shared sync coordinator for dependency injection
pub type SharedSyncCoordinator = Arc<dyn SyncCoordinator>;
