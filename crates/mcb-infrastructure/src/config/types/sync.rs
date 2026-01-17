//! Sync configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,

    /// Sync batch size
    pub batch_size: usize,

    /// Sync debounce delay in milliseconds
    pub debounce_delay_ms: u64,

    /// Sync timeout in seconds
    pub timeout_secs: u64,

    /// Maximum concurrent sync operations
    pub max_concurrent: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: SYNC_BATCH_SIZE,
            debounce_delay_ms: SYNC_DEBOUNCE_DELAY_MS,
            timeout_secs: SYNC_TIMEOUT_SECS,
            max_concurrent: 10,
        }
    }
}
