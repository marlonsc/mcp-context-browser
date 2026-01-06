//! Background daemon for automatic lock cleanup and monitoring
//!
//! Provides continuous monitoring and maintenance services:
//! - Automatic cleanup of stale lockfiles
//! - Sync activity monitoring and reporting
//! - Background health checks

use crate::core::error::{Error, Result};
use crate::sync::lockfile::CodebaseLockManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

/// Background daemon configuration
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Lock cleanup interval in seconds (default: 30)
    pub cleanup_interval_secs: u64,
    /// Monitoring interval in seconds (default: 30)
    pub monitoring_interval_secs: u64,
    /// Maximum age for lock cleanup in seconds (default: 300 = 5 minutes)
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
#[derive(Debug, Clone)]
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

/// Background daemon for maintenance tasks
pub struct ContextDaemon {
    config: DaemonConfig,
    stats: Arc<Mutex<DaemonStats>>,
    running: Arc<Mutex<bool>>,
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
            stats: Arc::new(Mutex::new(DaemonStats {
                cleanup_cycles: 0,
                locks_cleaned: 0,
                monitoring_cycles: 0,
                active_locks: 0,
                last_cleanup: None,
                last_monitoring: None,
            })),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the daemon (non-blocking)
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(Error::internal("Daemon is already running"));
        }
        *running = true;
        drop(running);

        println!("[DAEMON] Starting background daemon...");
        println!("[DAEMON] Cleanup interval: {}s", self.config.cleanup_interval_secs);
        println!("[DAEMON] Monitoring interval: {}s", self.config.monitoring_interval_secs);
        println!("[DAEMON] Max lock age: {}s", self.config.max_lock_age_secs);

        // Start cleanup task
        let cleanup_handle = {
            let stats = Arc::clone(&self.stats);
            let config = self.config.clone();
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(config.cleanup_interval_secs));

                loop {
                    interval.tick().await;

                    let is_running = *running.lock().await;
                    if !is_running {
                        break;
                    }

                    if let Err(e) = Self::run_cleanup_cycle(&stats, config.max_lock_age_secs).await {
                        eprintln!("[DAEMON] Cleanup cycle failed: {}", e);
                    }
                }
            })
        };

        // Start monitoring task
        let monitoring_handle = {
            let stats = Arc::clone(&self.stats);
            let running = Arc::clone(&self.running);

            tokio::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(self.config.monitoring_interval_secs));

                loop {
                    interval.tick().await;

                    let is_running = *running.lock().await;
                    if !is_running {
                        break;
                    }

                    if let Err(e) = Self::run_monitoring_cycle(&stats).await {
                        eprintln!("[DAEMON] Monitoring cycle failed: {}", e);
                    }
                }
            })
        };

        // Wait for both tasks (they run indefinitely until stopped)
        tokio::select! {
            _ = cleanup_handle => {
                println!("[DAEMON] Cleanup task ended");
            }
            _ = monitoring_handle => {
                println!("[DAEMON] Monitoring task ended");
            }
        }

        Ok(())
    }

    /// Stop the daemon
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        println!("[DAEMON] Stop signal sent to background daemon");
        Ok(())
    }

    /// Get current daemon statistics
    pub async fn get_stats(&self) -> DaemonStats {
        let stats = self.stats.lock().await;
        (*stats).clone()
    }

    /// Run a single cleanup cycle
    async fn run_cleanup_cycle(stats: &Arc<Mutex<DaemonStats>>, max_age_secs: u64) -> Result<()> {
        let cleaned = CodebaseLockManager::cleanup_stale_locks().await?;

        let mut stats = stats.lock().await;
        stats.cleanup_cycles += 1;
        stats.locks_cleaned += cleaned as u64;
        stats.last_cleanup = Some(std::time::SystemTime::now());

        if cleaned > 0 {
            println!("[DAEMON] Cleaned up {} stale locks", cleaned);
        }

        Ok(())
    }

    /// Run a single monitoring cycle
    async fn run_monitoring_cycle(stats: &Arc<Mutex<DaemonStats>>) -> Result<()> {
        let active_locks = CodebaseLockManager::get_active_locks().await?;
        let lock_count = active_locks.len();

        let mut stats = stats.lock().await;
        stats.monitoring_cycles += 1;
        stats.active_locks = lock_count;
        stats.last_monitoring = Some(std::time::SystemTime::now());

        // Warn about high concurrency
        if lock_count > 3 {
            println!("[DAEMON] Warning: {} concurrent sync operations detected", lock_count);
        }

        // Log active locks for debugging
        if lock_count > 0 {
            println!("[DAEMON] Active sync operations: {}", lock_count);
            for lock in &active_locks {
                println!("[DAEMON]   - {} (PID: {}, Host: {})",
                    lock.codebase_path,
                    lock.pid,
                    lock.hostname
                );
            }
        }

        Ok(())
    }

    /// Get daemon configuration
    pub fn config(&self) -> &DaemonConfig {
        &self.config
    }

    /// Check if daemon is running
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
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
    use std::time::Duration;

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