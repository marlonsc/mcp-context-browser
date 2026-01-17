//! Snapshot and State Store Adapters
//!
//! Null implementations of snapshot and state store ports for testing.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::{SnapshotProvider, StateStoreProvider};
use mcb_domain::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use mcb_domain::error::Result;
use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// Snapshot Provider (File Change Tracking)
// ============================================================================

/// Null snapshot provider for testing and Shaku DI default
///
/// Returns empty snapshots without accessing the filesystem.
#[derive(shaku::Component)]
#[shaku(interface = SnapshotProvider)]
pub struct NullSnapshotProvider;

impl NullSnapshotProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullSnapshotProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SnapshotProvider for NullSnapshotProvider {
    async fn create_snapshot(&self, root_path: &Path) -> Result<CodebaseSnapshot> {
        Ok(CodebaseSnapshot {
            id: "null-snapshot".to_string(),
            created_at: 0,
            collection: root_path.to_string_lossy().to_string(),
            files: HashMap::new(),
            total_files: 0,
            total_size: 0,
        })
    }

    async fn load_snapshot(&self, _root_path: &Path) -> Result<Option<CodebaseSnapshot>> {
        Ok(None)
    }

    async fn compare_snapshots(
        &self,
        _old_snapshot: &CodebaseSnapshot,
        _new_snapshot: &CodebaseSnapshot,
    ) -> Result<SnapshotChanges> {
        Ok(SnapshotChanges {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
        })
    }

    async fn get_changed_files(&self, _root_path: &Path) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

// ============================================================================
// State Store Provider (Key-Value Persistence)
// ============================================================================

/// Null state store provider for testing and Shaku DI default
#[allow(dead_code)] // Instantiated by Shaku DI container
#[derive(shaku::Component)]
#[shaku(interface = StateStoreProvider)]
pub struct NullStateStoreProvider;

impl NullStateStoreProvider {
    #[allow(dead_code)] // Used by Shaku DI
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullStateStoreProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StateStoreProvider for NullStateStoreProvider {
    async fn save(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn load(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }
}
