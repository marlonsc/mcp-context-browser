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

mod config;
mod local;
mod operations;
mod redis;
mod stats;

// Re-export configuration types
pub use config::{
    CacheConfig, CacheEntry, CacheNamespaceConfig, CacheNamespacesConfig, CacheResult, CacheStats,
};

use crate::domain::error::{Error, Result};
use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use moka::future::Cache;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Convert Redis errors to domain errors in the infrastructure layer
impl From<::redis::RedisError> for Error {
    fn from(err: ::redis::RedisError) -> Self {
        Self::Cache {
            message: err.to_string(),
        }
    }
}

/// Advanced distributed cache manager
#[derive(Clone)]
pub struct CacheManager {
    pub(crate) config: CacheConfig,
    /// Redis client (Present if in Remote mode)
    pub(crate) redis_client: Option<::redis::Client>,
    /// Local Moka caches (Present/Used if in Local mode)
    /// We keep these initialized but empty if in Redis mode to simplify struct
    pub(crate) embeddings_cache: Cache<String, serde_json::Value>,
    pub(crate) search_results_cache: Cache<String, serde_json::Value>,
    pub(crate) metadata_cache: Cache<String, serde_json::Value>,
    pub(crate) provider_responses_cache: Cache<String, serde_json::Value>,
    pub(crate) sync_batches_cache: Cache<String, serde_json::Value>,

    pub(crate) stats_hits: Arc<AtomicU64>,
    pub(crate) stats_misses: Arc<AtomicU64>,
    pub(crate) stats_evictions: Arc<AtomicU64>,
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
            match ::redis::Client::open(config.redis_url.clone()) {
                Ok(client) => {
                    tracing::debug!("Redis Client open success");
                    // Test connection
                    tracing::debug!("Testing redis connection...");
                    match Self::test_redis_connection(&client).await {
                        Ok(_) => {
                            tracing::debug!("Redis connection established");
                            tracing::info!("Redis cache connection established (Remote Mode)");
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
            tracing::info!("Redis not configured, using Local Moka Cache (Local Mode)");
        } else {
            tracing::info!("Caching disabled");
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
    async fn test_cache_manager_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await?;
        assert!(!manager.is_enabled());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_local_cache_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(), // Explicitly empty for Local mode
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await?;

        // Test set and get
        manager
            .set("test", "key1", "value1".to_string())
            .await?;

        let result: CacheResult<String> = manager.get("test", "key1").await;
        assert!(result.is_hit());
        let data = result.data().ok_or("Expected data in cache hit")?;
        assert_eq!(data, "value1");

        // Test miss
        let result: CacheResult<String> = manager.get("test", "nonexistent").await;
        assert!(result.is_miss());

        // Test delete
        manager.delete("test", "key1").await?;
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
        Ok(())
    }

    #[tokio::test]
    async fn test_namespace_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(),
            ..Default::default()
        };

        let manager = CacheManager::new(config, None).await?;

        // Set values in different namespaces
        manager
            .set("ns1", "key1", "value1".to_string())
            .await?;
        manager
            .set("ns2", "key1", "value2".to_string())
            .await?;

        // Get values
        let result1: CacheResult<String> = manager.get("ns1", "key1").await;
        let result2: CacheResult<String> = manager.get("ns2", "key1").await;

        assert!(result1.is_hit());
        assert!(result2.is_hit());
        let data1 = result1.data().ok_or("Expected data in ns1 cache hit")?;
        let data2 = result2.data().ok_or("Expected data in ns2 cache hit")?;
        assert_eq!(data1, "value1");
        assert_eq!(data2, "value2");

        // Clear namespace
        manager.clear_namespace("ns1").await?;

        manager.get_stats().await;

        let result1: CacheResult<String> = manager.get("ns1", "key1").await;
        let result2: CacheResult<String> = manager.get("ns2", "key1").await;

        assert!(result1.is_miss());
        assert!(result2.is_hit());
        Ok(())
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
    async fn test_cache_manager_handles_disabled_cache_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig {
            redis_url: "".to_string(),
            default_ttl_seconds: 300,
            max_size: 0, // Disabled
            enabled: false,
            namespaces: CacheNamespacesConfig::default(),
        };

        let manager = CacheManager::new(config, None).await?;

        // Operations on disabled cache should not panic
        let set_result = manager.set("test", "key", "value".to_string()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "key").await;
        assert!(!matches!(get_result, CacheResult::Error(_)));
        Ok(())
    }

    #[tokio::test]
    async fn test_cache_manager_handles_large_data_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let config = CacheConfig::default(); // Local mode
        let manager = CacheManager::new(config, None).await?;

        // Test with large data
        let large_data = "x".repeat(1024 * 1024); // 1MB string

        let set_result = manager.set("test", "large_key", large_data.clone()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "large_key").await;
        match get_result {
            CacheResult::Hit(data) => assert_eq!(data, large_data),
            CacheResult::Miss => return Err("Expected cache hit".into()),
            CacheResult::Error(e) => return Err(format!("Expected no error, got: {}", e).into()),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_namespace_limits() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut config = CacheConfig {
            enabled: true,
            redis_url: "".to_string(),
            ..Default::default()
        };
        // Configure metadata namespace with small limit
        config.namespaces.metadata.max_entries = 2;

        let manager = CacheManager::new(config, None).await?;

        // Add 3 items to metadata namespace
        manager.set("meta", "k1", "v1".to_string()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        manager.set("meta", "k2", "v2".to_string()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        manager.set("meta", "k3", "v3".to_string()).await?;

        // Stats should show total entries <= 2 (might be 2)
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert!(stats.evictions >= 1);
        Ok(())
    }
}
