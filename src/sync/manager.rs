//! Synchronization manager with cross-process coordination and debouncing
//!
//! Manages codebase synchronization with:
//! - Cross-process lockfile coordination
//! - Configurable sync intervals
//! - Debouncing to prevent excessive syncs

use crate::core::error::{Error, Result};
use crate::sync::lockfile::CodebaseLockManager;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Synchronization configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Sync interval in milliseconds (default: 15 minutes)
    pub interval_ms: u64,
    /// Enable lockfile coordination (default: true)
    pub enable_lockfile: bool,
    /// Minimum debounce interval between syncs per codebase (default: 60 seconds)
    pub debounce_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_ms: 15 * 60 * 1000, // 15 minutes
            enable_lockfile: true,
            debounce_ms: 60 * 1000, // 60 seconds
        }
    }
}

impl SyncConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            interval_ms: std::env::var("SYNC_INTERVAL_MS")
                .unwrap_or_else(|_| "900000".to_string()) // 15 min default
                .parse()
                .unwrap_or(15 * 60 * 1000),
            enable_lockfile: std::env::var("ENABLE_SYNC_LOCKFILE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            debounce_ms: 60 * 1000, // Fixed 60s debounce
        }
    }
}

/// Sync statistics for monitoring
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub total_attempts: u64,
    pub successful: u64,
    pub skipped: u64,
    pub failed: u64,
    pub skipped_rate: f64,
}

/// Synchronization manager with cross-process coordination
pub struct SyncManager {
    config: SyncConfig,
    last_sync_times: Arc<Mutex<HashMap<String, Instant>>>,
    stats: Arc<Mutex<SyncStats>>,
}

impl SyncManager {
    /// Create a new sync manager with default config
    pub fn new() -> Self {
        Self::with_config(SyncConfig::from_env())
    }

    /// Create a new sync manager with custom config
    pub fn with_config(config: SyncConfig) -> Self {
        Self {
            config,
            last_sync_times: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(SyncStats {
                total_attempts: 0,
                successful: 0,
                skipped: 0,
                failed: 0,
                skipped_rate: 0.0,
            })),
        }
    }

    /// Check if codebase should be debounced (synced too recently)
    pub async fn should_debounce(&self, codebase_path: &Path) -> Result<bool> {
        let path_key = codebase_path.to_string_lossy().to_string();
        let mut last_sync_times = self.last_sync_times.lock().await;

        if let Some(last_sync) = last_sync_times.get(&path_key) {
            let elapsed = last_sync.elapsed();
            let debounce_duration = Duration::from_millis(self.config.debounce_ms);

            if elapsed < debounce_duration {
                println!(
                    "[SYNC] Debouncing {} - synced {}s ago (min {}s)",
                    codebase_path.display(),
                    elapsed.as_secs(),
                    debounce_duration.as_secs()
                );
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Update last sync time for a codebase
    async fn update_last_sync(&self, codebase_path: &Path) {
        let path_key = codebase_path.to_string_lossy().to_string();
        let mut last_sync_times = self.last_sync_times.lock().await;
        last_sync_times.insert(path_key, Instant::now());
    }

    /// Handle synchronization with lockfile coordination
    pub async fn sync_codebase(&self, codebase_path: &Path) -> Result<bool> {
        let mut stats = self.stats.lock().await;
        stats.total_attempts += 1;

        // Check debounce
        if self.should_debounce(codebase_path).await? {
            stats.skipped += 1;
            self.update_stats(&mut stats);
            return Ok(false);
        }

        // Try to acquire lock if enabled
        let lock_release = if self.config.enable_lockfile {
            match CodebaseLockManager::acquire_lock(codebase_path).await? {
                Some(release_fn) => Some(release_fn),
                None => {
                    println!(
                        "[SYNC] Skipping {} - sync in progress by another instance",
                        codebase_path.display()
                    );
                    stats.skipped += 1;
                    self.update_stats(&mut stats);
                    return Ok(false);
                }
            }
        } else {
            None
        };

        // Perform sync operation (placeholder for actual sync logic)
        println!("[SYNC] Starting sync for {}", codebase_path.display());

        // TODO: Implement actual sync logic here
        // For now, just simulate successful sync
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Update last sync time
        self.update_last_sync(codebase_path).await;

        // Release lock if acquired
        if let Some(release_fn) = lock_release {
            if let Err(e) = release_fn() {
                eprintln!("[SYNC] Failed to release lock: {}", e);
            }
        }

        stats.successful += 1;
        self.update_stats(&mut stats);

        println!("[SYNC] Completed sync for {}", codebase_path.display());
        Ok(true)
    }

    /// Update calculated statistics
    fn update_stats(&self, stats: &mut SyncStats) {
        if stats.total_attempts > 0 {
            stats.skipped_rate = (stats.skipped as f64 / stats.total_attempts as f64) * 100.0;
        }
    }

    /// Get current sync statistics
    pub async fn get_stats(&self) -> SyncStats {
        let stats = self.stats.lock().await;
        (*stats).clone()
    }

    /// Get sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Clean old sync timestamps (older than max_age)
    pub async fn clean_old_timestamps(&self, max_age: Duration) {
        let mut last_sync_times = self.last_sync_times.lock().await;
        let now = Instant::now();

        last_sync_times.retain(|_path, timestamp| now.duration_since(*timestamp) < max_age);
    }

    /// Get sync interval as Duration
    pub fn sync_interval(&self) -> Duration {
        Duration::from_millis(self.config.interval_ms)
    }

    /// Get debounce interval as Duration
    pub fn debounce_interval(&self) -> Duration {
        Duration::from_millis(self.config.debounce_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = SyncManager::new();
        assert!(manager.config().enable_lockfile);
        assert_eq!(manager.config().interval_ms, 15 * 60 * 1000); // 15 minutes
        assert_eq!(manager.config().debounce_ms, 60 * 1000); // 60 seconds
    }

    #[tokio::test]
    async fn test_sync_config_from_env() {
        // Test default config
        let config = SyncConfig::from_env();
        assert!(config.enable_lockfile);
        assert_eq!(config.interval_ms, 15 * 60 * 1000);
        assert_eq!(config.debounce_ms, 60 * 1000);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.enable_lockfile);
        assert_eq!(config.interval_ms, 15 * 60 * 1000);
        assert_eq!(config.debounce_ms, 60 * 1000);
    }

    #[tokio::test]
    async fn test_should_debounce() {
        let manager = SyncManager::new();
        let path = PathBuf::from("/tmp/test");

        // First call should not debounce
        assert!(!manager.should_debounce(&path).await.unwrap());

        // Update last sync time
        manager.update_last_sync(&path).await;

        // Second call should debounce (within 60 seconds)
        assert!(manager.should_debounce(&path).await.unwrap());
    }

    #[tokio::test]
    async fn test_sync_stats_initialization() {
        let manager = SyncManager::new();
        let stats = manager.get_stats().await;

        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.skipped_rate, 0.0);
    }

    #[tokio::test]
    async fn test_sync_intervals() {
        let manager = SyncManager::new();

        assert_eq!(manager.sync_interval(), Duration::from_millis(15 * 60 * 1000));
        assert_eq!(manager.debounce_interval(), Duration::from_millis(60 * 1000));
    }
}
