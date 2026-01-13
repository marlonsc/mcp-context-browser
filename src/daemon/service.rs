//! Context daemon implementation

use super::{DaemonConfig, DaemonStats};
use crate::domain::error::{Error, Result};
use crate::domain::types::SyncBatch;
use crate::infrastructure::cache::{CacheProviderQueue, SharedCacheProvider};
use crate::infrastructure::utils::TimeUtils;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time;
use tokio_util::sync::CancellationToken;

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
///
/// Uses `CancellationToken` for async-native shutdown signaling and
/// `JoinSet` for clean task lifecycle management.
pub struct ContextDaemon {
    config: DaemonConfig,
    cache_manager: Option<SharedCacheProvider>,
    stats: Arc<AtomicDaemonStats>,
    cancel_token: CancellationToken,
}

impl ContextDaemon {
    /// Create a new daemon with default config
    pub fn new() -> Self {
        Self::with_config(DaemonConfig::from_env(), None)
    }

    /// Create a new daemon with custom config
    pub fn with_config(config: DaemonConfig, cache_manager: Option<SharedCacheProvider>) -> Self {
        Self {
            config,
            cache_manager,
            stats: Arc::new(AtomicDaemonStats::new()),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Start the daemon (non-blocking)
    ///
    /// Note: Once stopped, a new daemon instance must be created to restart
    /// (CancellationToken cannot be reset).
    pub async fn start(&self) -> Result<()> {
        // Check if already cancelled (stop was called)
        if self.cancel_token.is_cancelled() {
            return Err(Error::internal(
                "Daemon was already stopped. Create a new instance to restart.",
            ));
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

        let mut join_set = JoinSet::new();

        // Spawn cleanup task with cancellation
        {
            let stats = Arc::clone(&self.stats);
            let config = self.config.clone();
            let token = self.cancel_token.clone();
            let cache_manager = self.cache_manager.clone();

            join_set.spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.cleanup_interval_secs));

                loop {
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => {
                            tracing::debug!("[DAEMON] Cleanup task received cancellation signal");
                            break;
                        }
                        _ = interval.tick() => {
                            if let Err(e) =
                                Self::run_cleanup_cycle(&stats, config.max_lock_age_secs, &cache_manager)
                                    .await
                            {
                                tracing::error!("[DAEMON] Cleanup cycle failed: {}", e);
                            }
                        }
                    }
                }
            });
        }

        // Spawn monitoring task with cancellation
        {
            let stats = Arc::clone(&self.stats);
            let config = self.config.clone();
            let token = self.cancel_token.clone();
            let cache_manager = self.cache_manager.clone();

            join_set.spawn(async move {
                let mut interval =
                    time::interval(Duration::from_secs(config.monitoring_interval_secs));

                loop {
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => {
                            tracing::debug!("[DAEMON] Monitoring task received cancellation signal");
                            break;
                        }
                        _ = interval.tick() => {
                            if let Err(e) = Self::run_monitoring_cycle(&stats, &cache_manager).await {
                                tracing::error!("[DAEMON] Monitoring cycle failed: {}", e);
                            }
                        }
                    }
                }
            });
        }

        // Wait for all tasks to complete (they run until cancelled)
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(()) => tracing::debug!("[DAEMON] Background task completed cleanly"),
                Err(e) => tracing::warn!("[DAEMON] Background task panicked: {}", e),
            }
        }

        tracing::info!("[DAEMON] All daemon tasks have ended");
        Ok(())
    }

    /// Stop the daemon
    ///
    /// Signals all background tasks to stop via CancellationToken.
    /// Tasks will exit cleanly at their next check point.
    pub async fn stop(&self) -> Result<()> {
        self.cancel_token.cancel();
        tracing::info!("[DAEMON] Cancellation signal sent to background daemon");
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
        cache_manager: &Option<SharedCacheProvider>,
    ) -> Result<()> {
        let mut cleaned_count = 0;
        if let Some(cache) = cache_manager {
            if let Ok(queue) = cache.get_queue::<SyncBatch>("sync_batches", "queue").await {
                let now = TimeUtils::now_unix_secs();

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
        stats
            .last_cleanup
            .store(TimeUtils::now_unix_secs(), Ordering::Relaxed);

        if cleaned_count > 0 {
            tracing::info!("[DAEMON] Cleaned up {} stale batches", cleaned_count);
        }

        Ok(())
    }

    /// Run a single monitoring cycle
    async fn run_monitoring_cycle(
        stats: &Arc<AtomicDaemonStats>,
        cache_manager: &Option<SharedCacheProvider>,
    ) -> Result<()> {
        let mut queue_size = 0;
        if let Some(cache) = cache_manager {
            if let Ok(queue) = cache.get_queue::<SyncBatch>("sync_batches", "queue").await {
                queue_size = queue.len();
            }
        }

        stats.monitoring_cycles.fetch_add(1, Ordering::Relaxed);
        stats.active_locks.store(queue_size, Ordering::Relaxed);
        stats
            .last_monitoring
            .store(TimeUtils::now_unix_secs(), Ordering::Relaxed);

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
    ///
    /// Returns true if the daemon has not been stopped (cancellation not signaled).
    pub fn is_running(&self) -> bool {
        !self.cancel_token.is_cancelled()
    }
}

impl Default for ContextDaemon {
    fn default() -> Self {
        Self::new()
    }
}
