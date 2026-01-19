//! Snapshot Provider Adapter
//!
//! Null implementation of snapshot port for testing.

use async_trait::async_trait;
use mcb_application::ports::infrastructure::SnapshotProvider;
use mcb_domain::entities::codebase::{CodebaseSnapshot, SnapshotChanges};
use mcb_domain::error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Null snapshot provider for testing
///
/// Returns empty snapshots without accessing the filesystem.
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
