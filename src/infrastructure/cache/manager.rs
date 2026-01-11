//! Advanced distributed cache manager implementation

use crate::domain::error::{Error, Result};
use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use moka::future::Cache;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use super::config::CacheConfig;

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
