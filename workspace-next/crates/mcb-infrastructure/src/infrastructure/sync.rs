//! Sync Provider Adapters
//!
//! Null implementations for both distributed locking and file sync coordination.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::{LockGuard, LockProvider};
use mcb_domain::ports::infrastructure::snapshot::SyncProvider;
use mcb_domain::value_objects::config::SyncBatch;
use std::path::Path;
use std::time::Duration;

// ============================================================================
// Lock Provider (Distributed Locking)
// ============================================================================

/// Null implementation for distributed lock testing
#[derive(shaku::Component)]
#[shaku(interface = LockProvider)]
pub struct NullLockProvider;

impl NullLockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullLockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LockProvider for NullLockProvider {
    async fn acquire_lock(&self, key: &str) -> Result<LockGuard> {
        Ok(LockGuard {
            key: key.to_string(),
            token: "null-token".to_string(),
        })
    }
    async fn release_lock(&self, _guard: LockGuard) -> Result<()> {
        Ok(())
    }
}

// ============================================================================
// Sync Provider (File Sync Coordination)
// ============================================================================

/// Null implementation for file sync testing
///
/// Returns default values without performing actual file system operations.
/// Used for testing when sync functionality is not needed.
#[derive(shaku::Component)]
#[shaku(interface = SyncProvider)]
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
