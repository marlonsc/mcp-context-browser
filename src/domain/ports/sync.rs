//! Sync Coordinator Domain Port
//!
//! Defines the business contract for file synchronization coordination.
//! This abstraction enables services to coordinate sync operations without
//! coupling to specific debouncing, queueing, or file-watching implementations.

use crate::domain::error::Result;
use async_trait::async_trait;
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
/// ```rust,ignore
/// use crate::domain::ports::sync::{SyncCoordinator, SyncOptions};
///
/// async fn sync_codebase(
///     coordinator: &dyn SyncCoordinator,
///     path: &Path,
/// ) -> Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    /// Mock sync coordinator for testing
    struct MockSyncCoordinator {
        should_debounce: AtomicBool,
        synced_paths: Mutex<HashSet<String>>,
        tracked_count: AtomicUsize,
        changed_files: Vec<String>,
    }

    impl MockSyncCoordinator {
        fn new() -> Self {
            Self {
                should_debounce: AtomicBool::new(false),
                synced_paths: Mutex::new(HashSet::new()),
                tracked_count: AtomicUsize::new(0),
                changed_files: vec!["changed.rs".to_string()],
            }
        }

        fn set_debounce(&self, debounce: bool) {
            self.should_debounce.store(debounce, Ordering::Relaxed);
        }
    }

    #[async_trait]
    impl SyncCoordinator for MockSyncCoordinator {
        async fn should_debounce(&self, _codebase_path: &Path) -> Result<bool> {
            Ok(self.should_debounce.load(Ordering::Relaxed))
        }

        async fn sync(&self, codebase_path: &Path, options: SyncOptions) -> Result<SyncResult> {
            if !options.force && self.should_debounce.load(Ordering::Relaxed) {
                return Ok(SyncResult::skipped());
            }

            let path_str = codebase_path.to_string_lossy().to_string();
            self.synced_paths.lock().unwrap().insert(path_str);

            Ok(SyncResult::completed(self.changed_files.clone()))
        }

        async fn get_changed_files(&self, _codebase_path: &Path) -> Result<Vec<String>> {
            Ok(self.changed_files.clone())
        }

        async fn mark_synced(&self, codebase_path: &Path) -> Result<()> {
            let path_str = codebase_path.to_string_lossy().to_string();
            self.synced_paths.lock().unwrap().insert(path_str);
            Ok(())
        }

        fn tracked_file_count(&self) -> usize {
            self.tracked_count.load(Ordering::Relaxed)
        }
    }

    #[tokio::test]
    async fn test_sync_coordinator_sync() {
        let coordinator = MockSyncCoordinator::new();
        let result = coordinator
            .sync(Path::new("/test"), SyncOptions::default())
            .await;

        assert!(result.is_ok());
        let sync_result = result.unwrap();
        assert!(sync_result.performed);
        assert_eq!(sync_result.files_changed, 1);
    }

    #[tokio::test]
    async fn test_sync_coordinator_debounce() {
        let coordinator = MockSyncCoordinator::new();
        coordinator.set_debounce(true);

        let result = coordinator
            .sync(Path::new("/test"), SyncOptions::default())
            .await;

        assert!(result.is_ok());
        let sync_result = result.unwrap();
        assert!(!sync_result.performed);
    }

    #[tokio::test]
    async fn test_sync_coordinator_force_sync() {
        let coordinator = MockSyncCoordinator::new();
        coordinator.set_debounce(true);

        let options = SyncOptions {
            force: true,
            ..Default::default()
        };

        let result = coordinator.sync(Path::new("/test"), options).await;

        assert!(result.is_ok());
        let sync_result = result.unwrap();
        assert!(sync_result.performed);
    }

    #[tokio::test]
    async fn test_sync_coordinator_get_changed_files() {
        let coordinator = MockSyncCoordinator::new();
        let result = coordinator.get_changed_files(Path::new("/test")).await;

        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files.contains(&"changed.rs".to_string()));
    }

    #[test]
    fn test_sync_result_skipped() {
        let result = SyncResult::skipped();
        assert!(!result.performed);
        assert_eq!(result.files_changed, 0);
    }

    #[test]
    fn test_sync_result_completed() {
        let result = SyncResult::completed(vec!["a.rs".to_string(), "b.rs".to_string()]);
        assert!(result.performed);
        assert_eq!(result.files_changed, 2);
    }
}
