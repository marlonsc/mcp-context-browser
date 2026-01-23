//! Backup configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup enabled
    pub enabled: bool,

    /// Backup directory
    pub directory: PathBuf,

    /// Backup interval in seconds
    pub interval_secs: u64,

    /// Maximum number of backups to keep
    pub max_backups: usize,

    /// Compress backups
    pub compress: bool,

    /// Encrypt backups
    pub encrypt: bool,

    /// Backup encryption key (if encryption enabled)
    pub encryption_key: Option<String>,
}

/// Returns default backup configuration with:
/// - Backups disabled by default
/// - Directory: ./backups
/// - Interval: 24 hours
/// - Keep last 7 backups
/// - Compression enabled, encryption disabled
impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: PathBuf::from("./backups"),
            interval_secs: 86400, // 24 hours
            max_backups: 7,
            compress: true,
            encrypt: false,
            encryption_key: None,
        }
    }
}
