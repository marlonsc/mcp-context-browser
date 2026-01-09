//! Advanced distributed caching system with Redis
//!
//! Provides high-performance caching for embeddings, search results, and metadata
//! with intelligent TTL management and cache invalidation strategies.
//!
//! # Architecture
//!
//! The cache system operates in one of two mutually exclusive modes:
//! 1. **Local Mode (Moka)**: High-performance in-memory caching using `moka`. Used when Redis is not configured.
//! 2. **Remote Mode (Redis)**: Distributed caching using Redis. Used when `redis_url` is configured.
//!
//! This split ensures predictable behavior:
//! - Single instances use fast local caching.
//! - Clustered/Distributed instances use Redis for consistency.

use crate::core::error::{Error, Result};
use crate::core::events::{SharedEventBus, SystemEvent};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use validator::Validate;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    /// Redis connection URL
    /// If provided and not empty, Redis (Remote) mode is used.
    /// If empty, Moka (Local) mode is used.
    pub redis_url: String,
    /// Default TTL for cache entries (seconds)
    #[validate(range(min = 1))]
    pub default_ttl_seconds: u64,
    /// Maximum cache size (number of entries) - Applies to Local Moka cache
    #[validate(range(min = 1))]
    pub max_size: usize,
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache namespaces configuration
    #[validate(nested)]
    pub namespaces: CacheNamespacesConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: String::new(),  // Default to Local (Moka) mode
            default_ttl_seconds: 3600, // 1 hour
            max_size: 10000,
            enabled: true,
            namespaces: CacheNamespacesConfig::default(),
        }
    }
}

/// Configuration for different cache namespaces
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheNamespacesConfig {
    /// Embedding cache settings
    #[validate(nested)]
    pub embeddings: CacheNamespaceConfig,
    /// Search results cache settings
    #[validate(nested)]
    pub search_results: CacheNamespaceConfig,
    /// Metadata cache settings
    #[validate(nested)]
    pub metadata: CacheNamespaceConfig,
    /// Provider responses cache settings
    #[validate(nested)]
    pub provider_responses: CacheNamespaceConfig,
    /// Sync batches cache settings
    #[validate(nested)]
    pub sync_batches: CacheNamespaceConfig,
}

impl Default for CacheNamespacesConfig {
    fn default() -> Self {
        Self {
            embeddings: CacheNamespaceConfig {
                ttl_seconds: 7200, // 2 hours
                max_entries: 5000,
                compression: true,
            },
            search_results: CacheNamespaceConfig {
                ttl_seconds: 1800, // 30 minutes
                max_entries: 2000,
                compression: false,
            },
            metadata: CacheNamespaceConfig {
                ttl_seconds: 3600, // 1 hour
                max_entries: 1000,
                compression: false,
            },
            provider_responses: CacheNamespaceConfig {
                ttl_seconds: 300, // 5 minutes
                max_entries: 3000,
                compression: true,
            },
            sync_batches: CacheNamespaceConfig {
                ttl_seconds: 86400, // 24 hours
                max_entries: 1000,
                compression: false,
            },
        }
    }
}

/// Configuration for a specific cache namespace
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheNamespaceConfig {
    /// TTL for entries in this namespace (seconds)
    #[validate(range(min = 1))]
    pub ttl_seconds: u64,
    /// Maximum number of entries for this namespace
    #[validate(range(min = 1))]
    pub max_entries: usize,
    /// Whether to compress entries
    pub compression: bool,
}

/// Cache entry with metadata
/// Used primarily for Redis serialization to preserve metadata across instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// Timestamp when entry was created
    pub created_at: u64,
    /// Timestamp when entry was last accessed
    pub accessed_at: u64,
    /// Number of times entry was accessed
    pub access_count: u64,
    /// Size of the entry in bytes
    pub size_bytes: usize,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Total cache size in bytes
    pub total_size_bytes: usize,
    /// Cache hit count
    pub hits: u64,
    /// Cache miss count
    pub misses: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Number of evictions
    pub evictions: u64,
    /// Average access time in microseconds
    pub avg_access_time_us: f64,
}

/// Cache operation result
#[derive(Debug)]
pub enum CacheResult<T> {
    /// Cache hit with data
    Hit(T),
    /// Cache miss
    Miss,
    /// Cache error
    Error(Error),
}

impl<T> CacheResult<T> {
    /// Check if this is a cache hit
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheResult::Hit(_))
    }

    /// Check if this is a cache miss
    pub fn is_miss(&self) -> bool {
        matches!(self, CacheResult::Miss)
    }

    /// Get the data if it's a hit
    pub fn data(self) -> Option<T> {
        match self {
            CacheResult::Hit(data) => Some(data),
            _ => None,
        }
    }
}

/// Advanced distributed cache manager
#[derive(Clone)]
pub struct CacheManager {
    config: CacheConfig,
    /// Redis client (Present if in Remote mode)
    redis_client: Option<redis::Client>,
    /// Local Moka caches (Present/Used if in Local mode)
    /// We keep these initialized but empty if in Redis mode to simplify struct
    embeddings_cache: Cache<String, serde_json::Value>,
    search_results_cache: Cache<String, serde_json::Value>,
    metadata_cache: Cache<String, serde_json::Value>,
    provider_responses_cache: Cache<String, serde_json::Value>,
    sync_batches_cache: Cache<String, serde_json::Value>,

    stats_hits: Arc<AtomicU64>,
    stats_misses: Arc<AtomicU64>,
    stats_evictions: Arc<AtomicU64>,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: CacheConfig, event_bus: Option<SharedEventBus>) -> Result<Self> {
        tracing::debug!("CacheManager::new started");

        let mut redis_client = None;

        if config.enabled && !config.redis_url.is_empty() {
            tracing::debug!(
                "Redis configured, attempting to connect to {}",
                config.redis_url
            );
            match redis::Client::open(config.redis_url.clone()) {
                Ok(client) => {
                    tracing::debug!("Redis Client open success");
                    // Test connection
                    tracing::debug!("Testing redis connection...");
                    match Self::test_redis_connection(&client).await {
                        Ok(_) => {
                            tracing::debug!("Redis connection established");
                            tracing::info!("✅ Redis cache connection established (Remote Mode)");
                            redis_client = Some(client);
                        }
                        Err(e) => {
                            tracing::error!("Redis connection failed: {}", e);
                            return Err(e); // Strict failure if Redis is configured
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Redis Client open failed: {}", e);
                    return Err(Error::generic(format!("Redis client open failed: {}", e)));
                }
            }
        } else if config.enabled {
            tracing::info!("ℹ️  Redis not configured, using Local Moka Cache (Local Mode)");
        } else {
            tracing::info!("ℹ️  Caching disabled");
        }

        // Initialize Moka caches (Used in Local Mode)
        // We initialize them even in Redis mode to keep struct consistent, but they won't be used.
        // This overhead is minimal (lazy allocation).

        let stats_hits = Arc::new(AtomicU64::new(0));
        let stats_misses = Arc::new(AtomicU64::new(0));
        let stats_evictions = Arc::new(AtomicU64::new(0));

        let evictions = stats_evictions.clone();
        let embeddings_cache = Cache::builder()
            .max_capacity(config.namespaces.embeddings.max_entries as u64)
            .time_to_live(Duration::from_secs(
                config.namespaces.embeddings.ttl_seconds,
            ))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        let evictions = stats_evictions.clone();
        let search_results_cache = Cache::builder()
            .max_capacity(config.namespaces.search_results.max_entries as u64)
            .time_to_live(Duration::from_secs(
                config.namespaces.search_results.ttl_seconds,
            ))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        let evictions = stats_evictions.clone();
        let metadata_cache = Cache::builder()
            .max_capacity(config.namespaces.metadata.max_entries as u64)
            .time_to_live(Duration::from_secs(config.namespaces.metadata.ttl_seconds))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        let evictions = stats_evictions.clone();
        let provider_responses_cache = Cache::builder()
            .max_capacity(config.namespaces.provider_responses.max_entries as u64)
            .time_to_live(Duration::from_secs(
                config.namespaces.provider_responses.ttl_seconds,
            ))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        let evictions = stats_evictions.clone();
        let sync_batches_cache = Cache::builder()
            .max_capacity(config.namespaces.sync_batches.max_entries as u64)
            .time_to_live(Duration::from_secs(
                config.namespaces.sync_batches.ttl_seconds,
            ))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        let manager = Self {
            config,
            redis_client,
            embeddings_cache,
            search_results_cache,
            metadata_cache,
            provider_responses_cache,
            sync_batches_cache,
            stats_hits,
            stats_misses,
            stats_evictions,
        };

        if let Some(bus) = event_bus {
            manager.start_event_listener(bus);
        }

        Ok(manager)
    }

    /// Start listening for system events
    pub fn start_event_listener(&self, event_bus: SharedEventBus) {
        let mut receiver = event_bus.subscribe();
        let manager = self.clone();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let SystemEvent::CacheClear { namespace } = event {
                    if let Some(ns) = namespace {
                        tracing::info!("[CACHE] Clearing namespace: {}", ns);
                        if let Err(e) = manager.clear_namespace(&ns).await {
                            tracing::error!("[CACHE] Failed to clear namespace {}: {}", ns, e);
                        }
                    } else {
                        tracing::info!("[CACHE] Clearing all namespaces");
                        let namespaces = [
                            "embeddings",
                            "search_results",
                            "metadata",
                            "provider_responses",
                            "sync_batches",
                        ];
                        for ns in namespaces {
                            if let Err(e) = manager.clear_namespace(ns).await {
                                tracing::error!("[CACHE] Failed to clear namespace {}: {}", ns, e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Test Redis connection
    async fn test_redis_connection(client: &redis::Client) -> Result<()> {
        let timeout_duration = Duration::from_secs(2);

        let mut conn =
            match tokio::time::timeout(timeout_duration, client.get_multiplexed_async_connection())
                .await
            {
                Ok(Ok(conn)) => conn,
                Ok(Err(e)) => {
                    return Err(Error::generic(format!("Redis connection failed: {}", e)));
                }
                Err(_) => return Err(Error::generic("Redis connection timed out")),
            };

        match tokio::time::timeout(
            timeout_duration,
            redis::cmd("PING").query_async::<()>(&mut conn),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(Error::generic(format!("Redis PING failed: {}", e))),
            Err(_) => Err(Error::generic("Redis PING timed out")),
        }
    }

    /// Get a value from cache
    pub async fn get<T>(&self, namespace: &str, key: &str) -> CacheResult<T>
    where
        T: for<'de> serde::Deserialize<'de> + Clone,
    {
        if !self.config.enabled {
            return CacheResult::Miss;
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            match self.get_from_redis(&full_key).await {
                Ok(Some(data)) => match serde_json::from_value(data) {
                    Ok(deserialized) => {
                        self.stats_hits.fetch_add(1, Ordering::Relaxed);
                        return CacheResult::Hit(deserialized);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached data from Redis: {}", e);
                    }
                },
                Ok(None) => {} // Miss
                Err(e) => {
                    tracing::warn!("Redis get failed: {}", e);
                    // In Remote mode, we don't fallback to local to avoid inconsistency
                }
            }
            self.stats_misses.fetch_add(1, Ordering::Relaxed);
            return CacheResult::Miss;
        }

        // Local Mode (Moka)
        match self.get_from_local(&full_key).await {
            Ok(Some(data)) => match serde_json::from_value(data) {
                Ok(deserialized) => {
                    self.stats_hits.fetch_add(1, Ordering::Relaxed);
                    CacheResult::Hit(deserialized)
                }
                Err(e) => {
                    self.stats_misses.fetch_add(1, Ordering::Relaxed);
                    CacheResult::Error(Error::generic(format!(
                        "Failed to deserialize local cached data: {}",
                        e
                    )))
                }
            },
            Ok(None) => {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
                CacheResult::Miss
            }
            Err(e) => {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
                CacheResult::Error(e)
            }
        }
    }

    /// Set a value in cache
    pub async fn set<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);
        let namespace_config = self.get_namespace_config(namespace);
        let ttl = namespace_config.ttl_seconds;

        // Serialize data
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            let size_bytes = serde_json::to_string(&data).map(|s| s.len()).unwrap_or(0);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Error::generic(format!("System time error: {}", e)))?
                .as_secs();

            let entry = CacheEntry {
                data,
                created_at: now,
                accessed_at: now,
                access_count: 1,
                size_bytes,
            };

            if let Err(e) = self.set_in_redis(&full_key, &entry, ttl).await {
                tracing::warn!("Redis set failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.set_in_local(&full_key, data).await?;

        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.delete_from_redis(&full_key).await {
                tracing::warn!("Redis delete failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.delete_from_local(&full_key).await?;

        Ok(())
    }

    /// Enqueue an item to a list in cache
    pub async fn enqueue_item<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Serialize data
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.enqueue_redis(&full_key, &data).await {
                tracing::warn!("Redis enqueue failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.enqueue_local(&full_key, data).await?;

        Ok(())
    }

    /// Get all items from a queue in cache
    pub async fn get_queue<T>(&self, namespace: &str, key: &str) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Clone + Send + Sync,
    {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            match self.get_queue_redis(&full_key).await {
                Ok(items) => {
                    let mut result = Vec::new();
                    for item in items {
                        match serde_json::from_value(item) {
                            Ok(deserialized) => result.push(deserialized),
                            Err(e) => tracing::warn!("Failed to deserialize queue item: {}", e),
                        }
                    }
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Redis get_queue failed: {}", e);
                    return Ok(Vec::new()); // Return empty on error to avoid breaking flow?
                }
            }
        }

        // Local Mode (Moka)
        if let Some(data_array) = self.get_from_local(&full_key).await?
            && let Some(array) = data_array.as_array()
        {
            let mut result = Vec::new();
            for item in array {
                match serde_json::from_value(item.clone()) {
                    Ok(deserialized) => result.push(deserialized),
                    Err(e) => tracing::warn!("Failed to deserialize local queue item: {}", e),
                }
            }
            return Ok(result);
        }

        Ok(Vec::new())
    }

    /// Remove an item from the queue
    pub async fn remove_item<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.remove_from_redis(&full_key, &data).await {
                tracing::warn!("Redis remove_item failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.remove_from_local(&full_key, &data).await?;

        Ok(())
    }

    /// Clear entire namespace
    pub async fn clear_namespace(&self, namespace: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let pattern = format!("{}:*", namespace);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.clear_namespace_redis(&pattern).await {
                tracing::warn!("Redis namespace clear failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.clear_namespace_local(namespace).await?;

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();

        // If in Redis mode, local caches are empty, stats will reflect that.
        // We could potentially fetch Redis info, but that's expensive/complex.
        // So stats will track local usage or be mostly empty for Redis mode (except hits/misses).

        for cache in [
            &self.embeddings_cache,
            &self.search_results_cache,
            &self.metadata_cache,
            &self.provider_responses_cache,
            &self.sync_batches_cache,
        ] {
            cache.run_pending_tasks().await;
            stats.total_entries += cache.entry_count() as usize;
        }

        stats.hits = self.stats_hits.load(Ordering::Relaxed);
        stats.misses = self.stats_misses.load(Ordering::Relaxed);
        stats.evictions = self.stats_evictions.load(Ordering::Relaxed);

        let total_requests = stats.hits + stats.misses;
        if total_requests > 0 {
            stats.hit_ratio = stats.hits as f64 / total_requests as f64;
        }

        stats
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    // Private methods

    fn get_namespace_config(&self, namespace: &str) -> &CacheNamespaceConfig {
        match namespace {
            "embeddings" => &self.config.namespaces.embeddings,
            "search_results" => &self.config.namespaces.search_results,
            "metadata" => &self.config.namespaces.metadata,
            "provider_responses" => &self.config.namespaces.provider_responses,
            "sync_batches" => &self.config.namespaces.sync_batches,
            _ => &self.config.namespaces.metadata, // Default to metadata config
        }
    }

    async fn enqueue_redis(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(value)?;
            redis::cmd("RPUSH")
                .arg(key)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn get_queue_redis(&self, key: &str) -> Result<Vec<serde_json::Value>> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let items: Vec<String> = redis::cmd("LRANGE")
                .arg(key)
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await?;

            let mut result = Vec::new();
            for item in items {
                let val: serde_json::Value = serde_json::from_str(&item)?;
                result.push(val);
            }
            return Ok(result);
        }
        Ok(Vec::new())
    }

    async fn remove_from_redis(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(value)?;
            // LREM key count value (count 0 means remove all occurrences)
            redis::cmd("LREM")
                .arg(key)
                .arg(0)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn enqueue_local(&self, full_key: &str, value: serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);

        let mut current_list = if let Some(existing) = cache.get(full_key).await {
            if let Some(arr) = existing.as_array() {
                arr.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        current_list.push(value);
        cache
            .insert(full_key.to_string(), serde_json::Value::Array(current_list))
            .await;
        Ok(())
    }

    async fn remove_from_local(&self, full_key: &str, value: &serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);

        if let Some(existing) = cache.get(full_key).await
            && let Some(arr) = existing.as_array()
        {
            // Remove all occurrences that match
            let new_list: Vec<serde_json::Value> =
                arr.iter().filter(|v| *v != value).cloned().collect();
            cache
                .insert(full_key.to_string(), serde_json::Value::Array(new_list))
                .await;
        }
        Ok(())
    }

    async fn get_from_redis(&self, key: &str) -> Result<Option<serde_json::Value>> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let data: Option<String> = redis::cmd("GET").arg(key).query_async(&mut conn).await?;

            if let Some(json_str) = data {
                let entry: CacheEntry<serde_json::Value> = serde_json::from_str(&json_str)?;
                return Ok(Some(entry.data));
            }
        }
        Ok(None)
    }

    async fn set_in_redis(
        &self,
        key: &str,
        entry: &CacheEntry<serde_json::Value>,
        ttl: u64,
    ) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(entry)?;
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn delete_from_redis(&self, key: &str) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            redis::cmd("DEL")
                .arg(key)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    async fn clear_namespace_redis(&self, pattern: &str) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(pattern)
                .query_async::<Vec<String>>(&mut conn)
                .await?;
            if !keys.is_empty() {
                redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<()>(&mut conn)
                    .await?;
            }
        }
        Ok(())
    }

    fn get_cache(&self, namespace: &str) -> &Cache<String, serde_json::Value> {
        match namespace {
            "embeddings" => &self.embeddings_cache,
            "search_results" => &self.search_results_cache,
            "metadata" => &self.metadata_cache,
            "provider_responses" => &self.provider_responses_cache,
            "sync_batches" => &self.sync_batches_cache,
            _ => &self.metadata_cache,
        }
    }

    async fn get_from_local(&self, full_key: &str) -> Result<Option<serde_json::Value>> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        Ok(cache.get(full_key).await)
    }

    async fn set_in_local(&self, full_key: &str, value: serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        cache.insert(full_key.to_string(), value).await;
        Ok(())
    }

    async fn delete_from_local(&self, full_key: &str) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        cache.invalidate(full_key).await;
        Ok(())
    }

    async fn clear_namespace_local(&self, namespace: &str) -> Result<()> {
        let cache = self.get_cache(namespace);
        let prefix = format!("{}:", namespace);
        if let Err(e) = cache.invalidate_entries_if(move |k, _v| k.starts_with(&prefix)) {
            tracing::warn!("Failed to invalidate entries: {}", e);
        }
        Ok(())
    }
}

/// Global cache manager instance
static CACHE_MANAGER: std::sync::OnceLock<CacheManager> = std::sync::OnceLock::new();

/// Initialize global cache manager
pub async fn init_global_cache_manager(
    config: CacheConfig,
    event_bus: Option<SharedEventBus>,
) -> Result<()> {
    let manager = CacheManager::new(config, event_bus).await?;
    CACHE_MANAGER
        .set(manager)
        .map_err(|_| "Cache manager already initialized".into())
}

/// Get global cache manager
pub fn get_global_cache_manager() -> Option<&'static CacheManager> {
    CACHE_MANAGER.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.default_ttl_seconds, 3600);
        assert_eq!(config.max_size, 10000);
        assert!(config.redis_url.is_empty()); // Default to Local mode
    }

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await.unwrap();
        assert!(!manager.is_enabled());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_local_cache_operations() {
        let config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(), // Explicitly empty for Local mode
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await.unwrap();

        // Test set and get
        manager
            .set("test", "key1", "value1".to_string())
            .await
            .unwrap();

        let result: CacheResult<String> = manager.get("test", "key1").await;
        assert!(result.is_hit());
        assert_eq!(result.data().unwrap(), "value1");

        // Test miss
        let result: CacheResult<String> = manager.get("test", "nonexistent").await;
        assert!(result.is_miss());

        // Test delete
        manager.delete("test", "key1").await.unwrap();
        let result: CacheResult<String> = manager.get("test", "key1").await;
        assert!(result.is_miss());

        // Check stats
        let stats = manager.get_stats().await;
        println!(
            "Stats after test: hits={}, misses={}, ratio={}",
            stats.hits, stats.misses, stats.hit_ratio
        );
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 2); // Should be 2 misses: nonexistent + deleted key
    }

    #[tokio::test]
    async fn test_namespace_operations() {
        let config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(),
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await.unwrap();

        // Set values in different namespaces
        manager
            .set("ns1", "key1", "value1".to_string())
            .await
            .unwrap();
        manager
            .set("ns2", "key1", "value2".to_string())
            .await
            .unwrap();

        // Get values
        let result1: CacheResult<String> = manager.get("ns1", "key1").await;
        let result2: CacheResult<String> = manager.get("ns2", "key1").await;

        assert!(result1.is_hit());
        assert!(result2.is_hit());
        assert_eq!(result1.data().unwrap(), "value1");
        assert_eq!(result2.data().unwrap(), "value2");

        // Clear namespace
        manager.clear_namespace("ns1").await.unwrap();

        manager.get_stats().await;

        let result1: CacheResult<String> = manager.get("ns1", "key1").await;
        let result2: CacheResult<String> = manager.get("ns2", "key1").await;

        assert!(result1.is_miss());
        assert!(result2.is_hit());
    }

    #[tokio::test]
    async fn test_cache_manager_fail_invalid_redis() {
        // Test with invalid Redis configuration
        // In the new Exclusive mode, this should FAIL on creation
        let config = CacheConfig {
            redis_url: "redis://invalid:6379".to_string(),
            default_ttl_seconds: 300,
            max_size: 100,
            enabled: true,
            namespaces: CacheNamespacesConfig::default(),
        };

        let result = CacheManager::new(config, None).await;
        assert!(result.is_err()); // Strict failure
    }

    #[tokio::test]
    async fn test_cache_manager_handles_disabled_cache_operations() {
        let config = CacheConfig {
            redis_url: "".to_string(),
            default_ttl_seconds: 300,
            max_size: 0, // Disabled
            enabled: false,
            namespaces: CacheNamespacesConfig::default(),
        };

        let manager = CacheManager::new(config, None).await.unwrap();

        // Operations on disabled cache should not panic
        let set_result = manager.set("test", "key", "value".to_string()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "key").await;
        assert!(!matches!(get_result, CacheResult::Error(_)));
    }

    #[tokio::test]
    async fn test_cache_manager_handles_large_data_operations() {
        let config = CacheConfig::default(); // Local mode
        let manager = CacheManager::new(config, None).await.unwrap();

        // Test with large data
        let large_data = "x".repeat(1024 * 1024); // 1MB string

        let set_result = manager.set("test", "large_key", large_data.clone()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "large_key").await;
        match get_result {
            CacheResult::Hit(data) => assert_eq!(data, large_data),
            CacheResult::Miss => panic!("Expected cache hit"),
            CacheResult::Error(_) => panic!("Expected no error"),
        }
    }

    #[tokio::test]
    async fn test_namespace_limits() {
        let mut config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(),
            ..Default::default()
        };
        // Configure metadata namespace with small limit
        config.namespaces.metadata.max_entries = 2;

        let manager = CacheManager::new(config, None).await.unwrap();

        // Add 3 items to metadata namespace
        manager.set("meta", "k1", "v1".to_string()).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        manager.set("meta", "k2", "v2".to_string()).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        manager.set("meta", "k3", "v3".to_string()).await.unwrap();

        // Stats should show total entries <= 2 (might be 2)
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert!(stats.evictions >= 1);
    }
}
