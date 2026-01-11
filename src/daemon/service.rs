//! Context daemon implementation

use crate::domain::error::{Error, Result};
use crate::domain::types::SyncBatch;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use super::{DaemonConfig, DaemonStats};

/// Internal atomic statistics
pub(crate) struct AtomicDaemonStats {
    pub(crate) cleanup_cycles: AtomicU64,
    pub(crate) locks_cleaned: AtomicU64,
    pub(crate) monitoring_cycles: AtomicU64,
    pub(crate) active_locks: AtomicUsize,
    pub(crate) last_cleanup: AtomicU64,    // Seconds since epoch
    pub(crate) last_monitoring: AtomicU64, // Seconds since epoch
}

impl AtomicDaemonStats {
    pub(crate) fn new() -> Self {
        Self {
            cleanup_cycles: AtomicU64::new(0),
            locks_cleaned: AtomicU64::new(0),
            monitoring_cycles: AtomicU64::new(0),
            active_locks: AtomicUsize::new(0),
            last_cleanup: AtomicU64::new(0),
            last_monitoring: AtomicU64::new(0),
        }
    }

    pub(crate) async fn to_stats(&self) -> DaemonStats {
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
    cache_manager: Option<Arc<crate::infrastructure::cache::CacheManager>>,
    stats: Arc<AtomicDaemonStats>,
    running: Arc<AtomicBool>,
}

impl ContextDaemon {
    /// Create a new daemon with default config
    pub fn new() -> Self {
        Self::with_config(DaemonConfig::from_env(), None)
    }

    /// Create a new daemon with custom config
    pub fn with_config(
        config: DaemonConfig,
        cache_manager: Option<Arc<crate::infrastructure::cache::CacheManager>>,
    ) -> Self {
        Self {
            config,
            cache_manager,
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
            let cache_manager = self.cache_manager.clone();

            tokio::spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.cleanup_interval_secs));

                loop {
                    interval.tick().await;

                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Err(e) =
                        Self::run_cleanup_cycle(&stats, config.max_lock_age_secs, &cache_manager)
                            .await
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
            let cache_manager = self.cache_manager.clone();

            tokio::spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.monitoring_interval_secs));

                loop {
                    interval.tick().await;

                    if !running.load(Ordering::Relaxed) {
                        break;
                    }

                    if let Err(e) = Self::run_monitoring_cycle(&stats, &cache_manager).await {
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
    async fn run_cleanup_cycle(
        stats: &Arc<AtomicDaemonStats>,
        max_age_secs: u64,
        cache_manager: &Option<Arc<crate::infrastructure::cache::CacheManager>>,
    ) -> Result<()> {
        let mut cleaned_count = 0;
        if let Some(cache) = cache_manager {
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
    async fn run_monitoring_cycle(
        stats: &Arc<AtomicDaemonStats>,
        cache_manager: &Option<Arc<crate::infrastructure::cache::CacheManager>>,
    ) -> Result<()> {
        let mut queue_size = 0;
        if let Some(cache) = cache_manager {
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
