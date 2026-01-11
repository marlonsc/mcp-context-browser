//! Background daemon for automatic lock cleanup and monitoring
//!
//! Provides continuous monitoring and maintenance services:
//! - Automatic cleanup of stale sync batches
//! - Sync activity monitoring and reporting
//! - Background health checks

pub use crate::sync::manager::{SyncConfig, SyncManager, SyncStats};

// DaemonConfig is defined in this module

use crate::domain::error::{Error, Result};
use crate::domain::types::SyncBatch;
use crate::infrastructure::cache::get_global_cache_manager;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
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

/// Internal atomic statistics
struct AtomicDaemonStats {
    cleanup_cycles: AtomicU64,
    locks_cleaned: AtomicU64,
    monitoring_cycles: AtomicU64,
    active_locks: AtomicUsize,
    last_cleanup: AtomicU64,    // Seconds since epoch
    last_monitoring: AtomicU64, // Seconds since epoch
}

impl AtomicDaemonStats {
    fn new() -> Self {
        Self {
            cleanup_cycles: AtomicU64::new(0),
            locks_cleaned: AtomicU64::new(0),
            monitoring_cycles: AtomicU64::new(0),
            active_locks: AtomicUsize::new(0),
            last_cleanup: AtomicU64::new(0),
            last_monitoring: AtomicU64::new(0),
        }
    }

    async fn to_stats(&self) -> DaemonStats {
        let last_cleanup = self.last_cleanup.load(Ordering::Relaxed);
        let last_monitoring = self.last_monitoring.load(Ordering::Relaxed);

        DaemonStats {
            cleanup_cycles: self.cleanup_cycles.load(Ordering::Relaxed),
            locks_cleaned: self.locks_cleaned.load(Ordering::Relaxed),
            monitoring_cycles: self.monitoring_cycles.load(Ordering::Relaxed),
            active_locks: self.active_locks.load(Ordering::Relaxed),
            last_cleanup: if last_cleanup > 0 {
                Some(std::time::UNIX_EPOCH + Duration::from_secs(last_cleanup))
            } else {
                None
            },
            last_monitoring: if last_monitoring > 0 {
                Some(std::time::UNIX_EPOCH + Duration::from_secs(last_monitoring))
            } else {
                None
            },
        }
    }
}

/// Background daemon for maintenance tasks
pub struct ContextDaemon {
    config: DaemonConfig,
    stats: Arc<AtomicDaemonStats>,
    running: Arc<AtomicBool>,
}

impl ContextDaemon {
    /// Create a new daemon with default config
    pub fn new() -> Self {
        Self::with_config(DaemonConfig::from_env())
    }

    /// Create a new daemon with custom config
    pub fn with_config(config: DaemonConfig) -> Self {
        Self {
            config,
            stats: Arc::new(AtomicDaemonStats::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the daemon (non-blocking)
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(Error::internal("Daemon is already running"));
        }

        tracing::info!("[DAEMON] Starting background daemon...");
        tracing::debug!(
            "[DAEMON] Cleanup interval: {}s",
            self.config.cleanup_interval_secs
        );
        tracing::debug!(
            "[DAEMON] Monitoring interval: {}s",
            self.config.monitoring_interval_secs
        );
        tracing::debug!("[DAEMON] Max lock age: {}s", self.config.max_lock_age_secs);

        // Start cleanup task
        let cleanup_handle = {
            let stats = Arc::clone(&self.stats);
            let config = self.config.clone();
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.cleanup_interval_secs));

                loop {
                    interval.tick().await;

                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Err(e) = Self::run_cleanup_cycle(&stats, config.max_lock_age_secs).await
                    {
                        tracing::error!("[DAEMON] Cleanup cycle failed: {}", e);
                    }
                }
            })
        };

        // Start monitoring task
        let monitoring_handle = {
            let stats = Arc::clone(&self.stats);
            let running = Arc::clone(&self.running);
            let config = self.config.clone();

            tokio::spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.monitoring_interval_secs));

                loop {
                    interval.tick().await;

                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Err(e) = Self::run_monitoring_cycle(&stats).await {
                        tracing::error!("[DAEMON] Monitoring cycle failed: {}", e);
                    }
                }
            })
        };

        // Wait for both tasks (they run indefinitely until stopped)
        tokio::select! {
            _ = cleanup_handle => {
                tracing::info!("[DAEMON] Cleanup task ended");
            }
            _ = monitoring_handle => {
                tracing::info!("[DAEMON] Monitoring task ended");
            }
        }

        Ok(())
    }

    /// Stop the daemon
    pub async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        tracing::info!("[DAEMON] Stop signal sent to background daemon");
        Ok(())
    }

    /// Get current daemon statistics
    pub async fn get_stats(&self) -> DaemonStats {
        self.stats.to_stats().await
    }

    /// Run a single cleanup cycle
    async fn run_cleanup_cycle(stats: &Arc<AtomicDaemonStats>, max_age_secs: u64) -> Result<()> {
        let mut cleaned_count = 0;
        if let Some(cache) = get_global_cache_manager() {
            if let Ok(queue) = cache.get_queue::<SyncBatch>("sync_batches", "queue").await {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs();

                for batch in queue {
                    if now.saturating_sub(batch.created_at) > max_age_secs
                        && cache
                            .remove_item("sync_batches", "queue", batch)
                            .await
                            .is_ok()
                    {
                        cleaned_count += 1;
                    }
                }
            }
        }

        stats.cleanup_cycles.fetch_add(1, Ordering::Relaxed);
        stats
            .locks_cleaned
            .fetch_add(cleaned_count, Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        stats.last_cleanup.store(now, Ordering::Relaxed);

        if cleaned_count > 0 {
            tracing::info!("[DAEMON] Cleaned up {} stale batches", cleaned_count);
        }

        Ok(())
    }

    /// Run a single monitoring cycle
    async fn run_monitoring_cycle(stats: &Arc<AtomicDaemonStats>) -> Result<()> {
        let mut queue_size = 0;
        if let Some(cache) = get_global_cache_manager() {
            if let Ok(queue) = cache.get_queue::<SyncBatch>("sync_batches", "queue").await {
                queue_size = queue.len();
            }
        }

        stats.monitoring_cycles.fetch_add(1, Ordering::Relaxed);
        stats.active_locks.store(queue_size, Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        stats.last_monitoring.store(now, Ordering::Relaxed);

        // Warn about high backlog
        if queue_size > 10 {
            tracing::warn!("[DAEMON] Warning: {} pending sync batches", queue_size);
        }

        // Log active batches for debugging
        if queue_size > 0 {
            tracing::debug!("[DAEMON] Active sync batches: {}", queue_size);
        }

        Ok(())
    }

    /// Get daemon configuration
    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
impl Default for ContextDaemon {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert_eq!(config.cleanup_interval_secs, 30);
        assert_eq!(config.monitoring_interval_secs, 30);
        assert_eq!(config.max_lock_age_secs, 300);
    }

    #[test]
    fn test_daemon_config_from_env() {
        // Test with default values since no env vars are set
        let config = DaemonConfig::from_env();
        assert_eq!(config.cleanup_interval_secs, 30);
        assert_eq!(config.monitoring_interval_secs, 30);
        assert_eq!(config.max_lock_age_secs, 300);
    }

    #[tokio::test]
    async fn test_daemon_creation() {
        let daemon = ContextDaemon::new();
        assert!(!daemon.is_running().await);

        let stats = daemon.get_stats().await;
        assert_eq!(stats.cleanup_cycles, 0);
        assert_eq!(stats.locks_cleaned, 0);
        assert_eq!(stats.monitoring_cycles, 0);
        assert_eq!(stats.active_locks, 0);
    }

    #[tokio::test]
    async fn test_daemon_stop_before_start() {
        let daemon = ContextDaemon::new();
        assert!(daemon.stop().await.is_ok());
        assert!(!daemon.is_running().await);
    }
}
