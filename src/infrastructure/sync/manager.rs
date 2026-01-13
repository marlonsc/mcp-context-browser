//! Synchronization manager with cross-process coordination and debouncing
//!
//! Manages codebase synchronization with:
//! - Cross-process lockfile coordination
//! - Configurable sync intervals
//! - Debouncing to prevent excessive syncs (delegated to DebounceService)
//! - Statistics collection (delegated to SyncStatsCollector)

use super::debounce::{DebounceConfig, DebounceService};
use super::stats::SyncStatsCollector;
use crate::domain::error::Result;
use crate::domain::ports::SyncProvider;
use crate::domain::types::SyncBatch;
use crate::infrastructure::cache::{CacheProviderQueue, SharedCacheProvider};
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use crate::infrastructure::utils::TimeUtils;
use async_trait::async_trait;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;
use walkdir::WalkDir;

/// Synchronization configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct SyncConfig {
    /// Sync interval in milliseconds (default: 15 minutes)
    #[validate(range(min = 1))]
    pub interval_ms: u64,
    /// Minimum debounce interval between syncs per codebase (default: 60 seconds)
    #[validate(range(min = 1))]
    pub debounce_ms: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_ms: 15 * 60 * 1000, // 15 minutes
            debounce_ms: 60 * 1000,      // 60 seconds
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
            debounce_ms: 60 * 1000, // Fixed 60s debounce
        }
    }
}

// SyncStats is re-exported from stats module
pub use super::stats::SyncStats;

/// Synchronization manager with cross-process coordination
pub struct SyncManager {
    config: SyncConfig,
    cache_manager: Option<SharedCacheProvider>,
    /// Debounce service for rate limiting syncs
    debounce: Arc<DebounceService>,
    /// File modification times for change detection
    file_mod_times: DashMap<String, u64>,
    /// Statistics collector service
    stats: Arc<SyncStatsCollector>,
    event_bus: Option<SharedEventBusProvider>,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncManager {
    /// Create a new sync manager with default config
    pub fn new() -> Self {
        Self::with_config(SyncConfig::from_env(), None)
    }

    /// Create a new sync manager with custom config
    pub fn with_config(config: SyncConfig, cache_manager: Option<SharedCacheProvider>) -> Self {
        let debounce_config = DebounceConfig {
            debounce_ms: config.debounce_ms,
        };
        Self {
            config,
            cache_manager,
            debounce: Arc::new(DebounceService::new(debounce_config)),
            file_mod_times: DashMap::new(),
            stats: Arc::new(SyncStatsCollector::new()),
            event_bus: None,
        }
    }

    /// Create a new sync manager with event bus for publishing sync events
    pub fn with_event_bus(
        config: SyncConfig,
        event_bus: SharedEventBusProvider,
        cache_manager: Option<SharedCacheProvider>,
    ) -> Self {
        let debounce_config = DebounceConfig {
            debounce_ms: config.debounce_ms,
        };
        Self {
            config,
            cache_manager,
            debounce: Arc::new(DebounceService::new(debounce_config)),
            file_mod_times: DashMap::new(),
            stats: Arc::new(SyncStatsCollector::new()),
            event_bus: Some(event_bus),
        }
    }

    /// Check if codebase should be debounced (synced too recently)
    ///
    /// Delegates to `DebounceService` for the actual debounce logic.
    pub async fn should_debounce(&self, codebase_path: &Path) -> Result<bool> {
        Ok(self.debounce.should_debounce(codebase_path))
    }

    /// Update last sync time for a codebase
    ///
    /// Delegates to `DebounceService` for recording sync time.
    pub async fn update_last_sync(&self, codebase_path: &Path) {
        self.debounce.record_sync(codebase_path);
    }

    /// Handle synchronization with batch queue coordination
    pub async fn sync_codebase(&self, codebase_path: &Path) -> Result<bool> {
        // Verify path exists before proceeding
        if !codebase_path.exists() {
            return Err(crate::domain::error::Error::Io {
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Path does not exist: {}", codebase_path.display()),
                ),
            });
        }

        // Record attempt via stats collector service
        self.stats.record_attempt();

        // Check debounce (delegated to DebounceService)
        if self.should_debounce(codebase_path).await? {
            self.stats.record_skip();
            return Ok(false);
        }

        // Try to acquire sync slot
        let batch = match self.acquire_sync_slot(codebase_path).await? {
            Some(b) => b,
            None => {
                self.stats.record_skip();
                return Ok(false);
            }
        };

        // Perform actual sync operation
        tracing::info!("[SYNC] Starting sync for {}", codebase_path.display());

        // Scan codebase for changed files
        let changed_files = self.scan_for_changes(codebase_path).await;

        if !changed_files.is_empty() {
            tracing::info!(
                "[SYNC] Found {} changed files in {}",
                changed_files.len(),
                codebase_path.display()
            );

            // Update modification times for changed files (using millis for precision)
            for file_path in &changed_files {
                if let Ok(metadata) = std::fs::metadata(file_path) {
                    if let Ok(modified) = metadata.modified() {
                        let mod_time = modified
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(Duration::from_secs(0))
                            .as_millis() as u64;
                        self.file_mod_times.insert(file_path.clone(), mod_time);
                    }
                }
            }
        } else {
            tracing::debug!("[SYNC] No changes detected in {}", codebase_path.display());
        }

        // Update last sync time
        self.update_last_sync(codebase_path).await;

        // Release sync slot
        self.release_sync_slot(codebase_path, batch).await?;

        // Record success via stats collector service
        self.stats.record_success();

        // Publish SyncCompleted event if event bus is available
        if let Some(ref event_bus) = self.event_bus {
            let path = codebase_path.to_string_lossy().to_string();
            let files_changed = changed_files.len() as i32;
            if let Err(e) = event_bus
                .publish(SystemEvent::SyncCompleted {
                    path,
                    files_changed,
                })
                .await
            {
                tracing::warn!("[SYNC] Failed to publish SyncCompleted event: {}", e);
            }
        }

        tracing::info!("[SYNC] Completed sync for {}", codebase_path.display());
        Ok(true)
    }

    /// Acquire a synchronization slot in the queue
    pub async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>> {
        // Create SyncBatch
        let batch_id = Uuid::new_v4().to_string();
        let batch = SyncBatch {
            id: batch_id.clone(),
            path: codebase_path.to_string_lossy().to_string(),
            created_at: TimeUtils::now_unix_secs(),
        };

        // Enqueue if cache is available
        if let Some(cache) = &self.cache_manager {
            cache
                .enqueue_item::<SyncBatch>("sync_batches", "queue", batch.clone())
                .await?;

            // Check if we are head of queue
            let queue: Vec<SyncBatch> = cache.get_queue("sync_batches", "queue").await?;

            // Filter queue for this path
            let path_batches: Vec<&SyncBatch> =
                queue.iter().filter(|b| b.path == batch.path).collect();

            if let Some(first) = path_batches.first() {
                if first.id != batch.id {
                    // We are not first, so skip
                    tracing::info!("[SYNC] Queued behind batch {}", first.id);
                    return Ok(None);
                }
            }
            Ok(Some(batch))
        } else {
            tracing::warn!("[SYNC] Cache not available, proceeding without coordination");
            Ok(Some(batch))
        }
    }

    /// Release a synchronization slot in the queue
    pub async fn release_sync_slot(&self, _codebase_path: &Path, batch: SyncBatch) -> Result<()> {
        if let Some(cache) = &self.cache_manager {
            if let Err(e) = cache
                .remove_item::<SyncBatch>("sync_batches", "queue", batch.clone())
                .await
            {
                tracing::warn!("[SYNC] Failed to remove batch from queue: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }

    /// Get current sync statistics
    ///
    /// Delegates to `SyncStatsCollector` for the statistics snapshot.
    pub fn get_stats(&self) -> SyncStats {
        self.stats.snapshot()
    }

    /// Get the count of tracked files
    pub fn get_tracked_file_count(&self) -> usize {
        self.file_mod_times.len()
    }

    /// Get list of files that have changed since last sync
    pub async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>> {
        Ok(self.scan_for_changes(codebase_path).await)
    }

    /// Get sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Clean old sync timestamps (older than max_age)
    ///
    /// Delegates to `DebounceService` for timestamp cleanup.
    pub async fn clean_old_timestamps(&self, max_age: Duration) {
        // Delegate to DebounceService
        self.debounce.clean_older_than(max_age);

        // Also clean old batches
        self.clean_old_batches(Duration::from_secs(86400)).await; // 24h
    }

    /// Clean old sync batches from queue
    pub async fn clean_old_batches(&self, max_age: Duration) {
        if let Some(cache) = &self.cache_manager {
            if let Ok(queue) = cache.get_queue::<SyncBatch>("sync_batches", "queue").await {
                let now = TimeUtils::now_unix_secs();

                for batch in queue {
                    if now.saturating_sub(batch.created_at) > max_age.as_secs() {
                        tracing::info!("[SYNC] Removing stale batch {}", batch.id);
                        let _ = cache.remove_item("sync_batches", "queue", batch).await;
                    }
                }
            }
        }
    }

    /// Scan codebase for files that have changed since last sync
    async fn scan_for_changes(&self, codebase_path: &Path) -> Vec<String> {
        let mut changed_files = Vec::new();

        // Walk directory tree looking for source files
        for entry in WalkDir::new(codebase_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories and non-source files
            if !path.is_file() {
                continue;
            }

            // Check common source file extensions
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(
                extension,
                "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
            ) {
                continue;
            }

            let path_str = path.to_string_lossy().to_string();

            // Check if file has been modified since last sync
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let mod_time = modified
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_millis() as u64;

                    // Check if we have a previous modification time
                    if let Some(prev_mod_time) = self.file_mod_times.get(&path_str) {
                        if mod_time > *prev_mod_time {
                            changed_files.push(path_str);
                        }
                    } else {
                        // New file, not seen before
                        changed_files.push(path_str);
                    }
                }
            }
        }

        changed_files
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

#[async_trait]
impl SyncProvider for SyncManager {
    async fn should_debounce(&self, codebase_path: &Path) -> Result<bool> {
        self.should_debounce(codebase_path).await
    }

    async fn update_last_sync(&self, codebase_path: &Path) {
        self.update_last_sync(codebase_path).await
    }

    async fn acquire_sync_slot(&self, codebase_path: &Path) -> Result<Option<SyncBatch>> {
        self.acquire_sync_slot(codebase_path).await
    }

    async fn release_sync_slot(&self, codebase_path: &Path, batch: SyncBatch) -> Result<()> {
        self.release_sync_slot(codebase_path, batch).await
    }

    async fn get_changed_files(&self, codebase_path: &Path) -> Result<Vec<String>> {
        self.get_changed_files(codebase_path).await
    }

    fn sync_interval(&self) -> Duration {
        self.sync_interval()
    }

    fn debounce_interval(&self) -> Duration {
        self.debounce_interval()
    }
}
