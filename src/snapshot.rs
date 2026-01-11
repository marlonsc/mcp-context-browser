//! Snapshot management for incremental codebase tracking
//!
//! Tracks file changes using SHA256 hashing for efficient incremental sync.
//! Avoids reprocessing unchanged files during codebase indexing.

mod manager;

pub use manager::SnapshotManager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// File snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    /// Relative path from codebase root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time (Unix timestamp)
    pub modified: u64,
    /// Content hash (Merkle tree hash)
    pub hash: String,
    /// File extension (for filtering)
    pub extension: String,
}

/// Codebase snapshot with all files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseSnapshot {
    /// Codebase root path
    pub root_path: String,
    /// Snapshot creation timestamp
    pub created_at: u64,
    /// Map of relative paths to file snapshots
    pub files: HashMap<String, FileSnapshot>,
    /// Total number of files
    pub file_count: usize,
    /// Total size in bytes
    pub total_size: u64,
}

/// Changes between snapshots
#[derive(Debug, Clone)]
pub struct SnapshotChanges {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
}

impl SnapshotChanges {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    /// Get total number of changes
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}
