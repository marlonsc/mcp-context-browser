//! Sync Provider Adapter
//!
//! Null implementation for file sync coordination.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::SyncProvider;
use mcb_domain::error::Result;
use mcb_domain::value_objects::config::SyncBatch;
use std::path::Path;
use std::time::Duration;

/// Null sync provider for testing
///
/// Returns default values without performing actual file system operations.
pub struct NullSyncProvider;

impl NullSyncProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullSyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyncProvider for NullSyncProvider {
    async fn should_debounce(&self, _codebase_path: &Path) -> Result<bool> {
        Ok(false)
    }

    async fn update_last_sync(&self, _codebase_path: &Path) {
        // No-op for null implementation
    }

    async fn acquire_sync_slot(&self, _codebase_path: &Path) -> Result<Option<SyncBatch>> {
        Ok(None)
    }

    async fn release_sync_slot(&self, _codebase_path: &Path, _batch: SyncBatch) -> Result<()> {
        Ok(())
    }

    async fn get_changed_files(&self, _codebase_path: &Path) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn sync_interval(&self) -> Duration {
        Duration::from_secs(60)
    }

    fn debounce_interval(&self) -> Duration {
        Duration::from_secs(5)
    }
}
