//! Logging configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable JSON output format
    pub json_format: bool,

    /// Log to file in addition to stdout
    pub file_output: Option<PathBuf>,

    /// Maximum file size before rotation (bytes)
    pub max_file_size: u64,

    /// Maximum number of rotated files to keep
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: DEFAULT_LOG_LEVEL.to_string(),
            json_format: false,
            file_output: None,
            max_file_size: LOG_ROTATION_SIZE,
            max_files: LOG_MAX_FILES,
        }
    }
}
