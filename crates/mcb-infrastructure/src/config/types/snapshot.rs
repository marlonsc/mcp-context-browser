//! Snapshot configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Snapshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Snapshot enabled
    pub enabled: bool,

    /// Snapshot directory
    pub directory: PathBuf,

    /// Maximum file size for snapshot operations
    pub max_file_size: usize,

    /// Snapshot compression enabled
    pub compression_enabled: bool,

    /// Change detection enabled
    pub change_detection_enabled: bool,
}

/// Returns default snapshot configuration with:
/// - Snapshots enabled in ./snapshots directory
/// - Max file size from infrastructure constants
/// - Compression and change detection enabled
impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: PathBuf::from("./snapshots"),
            max_file_size: MAX_SNAPSHOT_FILE_SIZE,
            compression_enabled: true,
            change_detection_enabled: true,
        }
    }
}
