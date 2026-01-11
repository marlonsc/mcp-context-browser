//! Background daemon for automatic lock cleanup and monitoring
//!
//! Provides continuous monitoring and maintenance services:
//! - Automatic cleanup of stale sync batches
//! - Sync activity monitoring and reporting
//! - Background health checks

mod service;

pub use service::ContextDaemon;

use validator::Validate;

/// Background daemon configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct DaemonConfig {
    /// Lock cleanup interval in seconds (default: 30)
    #[validate(range(min = 1))]
    pub cleanup_interval_secs: u64,
    /// Monitoring interval in seconds (default: 30)
    #[validate(range(min = 1))]
    pub monitoring_interval_secs: u64,
    /// Maximum age for lock cleanup in seconds (default: 300 = 5 minutes)
    #[validate(range(min = 1))]
    pub max_lock_age_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_secs: 30,
            monitoring_interval_secs: 30,
            max_lock_age_secs: 300, // 5 minutes
        }
    }
}

impl DaemonConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            cleanup_interval_secs: std::env::var("DAEMON_CLEANUP_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            monitoring_interval_secs: std::env::var("DAEMON_MONITORING_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_lock_age_secs: std::env::var("DAEMON_MAX_LOCK_AGE")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        }
    }
}

/// Daemon statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct DaemonStats {
    /// Total cleanup cycles run
    pub cleanup_cycles: u64,
    /// Total locks cleaned up
    pub locks_cleaned: u64,
    /// Total monitoring cycles run
    pub monitoring_cycles: u64,
    /// Current number of active locks
    pub active_locks: usize,
    /// Timestamp of last cleanup
    pub last_cleanup: Option<std::time::SystemTime>,
    /// Timestamp of last monitoring
    pub last_monitoring: Option<std::time::SystemTime>,
}
