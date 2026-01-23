//! Operations configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsConfig {
    /// Operations tracking enabled
    pub tracking_enabled: bool,

    /// Operations cleanup interval in seconds
    pub cleanup_interval_secs: u64,

    /// Operations retention period in seconds
    pub retention_secs: u64,

    /// Maximum operations to keep in memory
    pub max_operations_in_memory: usize,
}

/// Returns default operations configuration with:
/// - Tracking enabled with periodic cleanup
/// - Cleanup interval and retention from infrastructure constants
/// - Memory limits from infrastructure constants
impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            cleanup_interval_secs: OPERATIONS_CLEANUP_INTERVAL_SECS,
            retention_secs: OPERATIONS_RETENTION_SECS,
            max_operations_in_memory: OPERATIONS_MAX_IN_MEMORY,
        }
    }
}
