//! Daemon configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Daemon enabled
    pub enabled: bool,

    /// Process check interval in seconds
    pub check_interval_secs: u64,

    /// Restart delay in seconds
    pub restart_delay_secs: u64,

    /// Maximum restart attempts
    pub max_restart_attempts: u32,

    /// Auto-start daemon
    pub auto_start: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DAEMON_CHECK_INTERVAL_SECS,
            restart_delay_secs: DAEMON_RESTART_DELAY_SECS,
            max_restart_attempts: DAEMON_MAX_RESTART_ATTEMPTS,
            auto_start: true,
        }
    }
}
