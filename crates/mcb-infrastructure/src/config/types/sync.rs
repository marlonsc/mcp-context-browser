//! Sync configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,

    /// Enable file watching for hot-reload
    ///
    /// When enabled, the system monitors config files for changes and
    /// automatically reloads configuration. Disable in environments
    /// where file watching is not supported (e.g., some containers).
    ///
    /// Configure via `MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED=false`
    /// to disable.
    #[serde(default = "default_watching_enabled")]
    pub watching_enabled: bool,

    /// Sync batch size
    pub batch_size: usize,

    /// Sync debounce delay in milliseconds
    pub debounce_delay_ms: u64,

    /// Sync timeout in seconds
    pub timeout_secs: u64,

    /// Maximum concurrent sync operations
    pub max_concurrent: usize,
}

fn default_watching_enabled() -> bool {
    true
}

/// Returns default sync configuration with:
/// - Sync enabled with file watching for hot-reload
/// - Batch size, debounce, and timeout from infrastructure constants
/// - Max 10 concurrent sync operations
impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watching_enabled: default_watching_enabled(),
            batch_size: SYNC_BATCH_SIZE,
            debounce_delay_ms: SYNC_DEBOUNCE_DELAY_MS,
            timeout_secs: SYNC_TIMEOUT_SECS,
            max_concurrent: 10,
        }
    }
}
