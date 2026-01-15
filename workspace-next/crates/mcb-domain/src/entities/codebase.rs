//! Codebase State Entities
//!
//! Entities for managing codebase state, snapshots, and change tracking.
//! These entities enable the system to track changes over time and
//! maintain consistency across indexing operations.

use crate::value_objects::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entity: File State for Change Tracking
///
/// Represents the state of a file at a specific point in time.
/// Used for detecting changes and managing incremental updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileSnapshot {
    /// Relative path to the file from repository root
    pub path: String,
    /// Last modification timestamp (Unix timestamp)
    pub modified_at: i64,
    /// File size in bytes
    pub size: u64,
    /// Content hash for change detection
    pub hash: String,
    /// Detected programming language
    pub language: Language,
}

/// Entity: Complete Codebase State Snapshot
///
/// Represents the complete state of a codebase at a specific point in time.
/// Used for change detection, backup, and incremental indexing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodebaseSnapshot {
    /// Unique identifier for this snapshot
    pub id: String,
    /// Timestamp when this snapshot was created
    pub created_at: i64,
    /// Repository or collection identifier
    pub collection: String,
    /// Map of file path to file snapshot
    pub files: HashMap<String, FileSnapshot>,
    /// Total number of files in the snapshot
    pub total_files: usize,
    /// Total size of all files in bytes
    pub total_size: u64,
}

/// Value Object: Changes Between Snapshots
///
/// Represents the differences between two codebase snapshots.
/// Used for incremental indexing and change notifications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotChanges {
    /// Files that were added
    pub added: Vec<String>,
    /// Files that were modified
    pub modified: Vec<String>,
    /// Files that were removed
    pub removed: Vec<String>,
}

impl SnapshotChanges {
    /// Check if there are any changes in this snapshot
    ///
    /// # Returns
    /// true if there are added, modified, or removed files, false otherwise
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.modified.is_empty() || !self.removed.is_empty()
    }

    /// Get the total number of changes across all categories
    ///
    /// # Returns
    /// The sum of added, modified, and removed files
    pub fn total_changes(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}