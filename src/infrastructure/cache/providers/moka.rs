//! Moka-based cache provider implementation
//!
//! Provides local in-memory caching using the Moka library.
//! Suitable for single-node deployments or testing.
//!
//! ## Features
//! - In-memory storage with per-namespace Moka caches
//! - Configurable TTL and max size per namespace
//! - Automatic eviction with statistics tracking
//! - No external dependencies required

use crate::domain::error::Result;
use crate::infrastructure::cache::config::CacheConfig;
use crate::infrastructure::cache::provider::{CacheProvider, CacheStats, HealthStatus};
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Moka-based cache provider for local in-memory caching
pub struct MokaCacheProvider {
    /// Cache for embedding vectors
    embeddings_cache: Cache<String, Vec<u8>>,
    /// Cache for search results
    search_results_cache: Cache<String, Vec<u8>>,
    /// Cache for metadata
    metadata_cache: Cache<String, Vec<u8>>,
    /// Cache for provider responses
    provider_responses_cache: Cache<String, Vec<u8>>,
    /// Cache for sync batches
    sync_batches_cache: Cache<String, Vec<u8>>,

    // Statistics tracking
    stats_hits: Arc<AtomicU64>,
    stats_misses: Arc<AtomicU64>,
    stats_evictions: Arc<AtomicU64>,
}

impl MokaCacheProvider {
    /// Create a new Moka cache provider
    pub fn new(config: CacheConfig) -> Result<Self> {
        tracing::info!("[CACHE] Initializing Moka provider (local mode)");

        let stats_evictions = Arc::new(AtomicU64::new(0));

        // Build embeddings cache
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

        // Build search results cache
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

        // Build metadata cache
        let evictions = stats_evictions.clone();
        let metadata_cache = Cache::builder()
            .max_capacity(config.namespaces.metadata.max_entries as u64)
            .time_to_live(Duration::from_secs(config.namespaces.metadata.ttl_seconds))
            .eviction_listener(move |_k, _v, _cause| {
                evictions.fetch_add(1, Ordering::Relaxed);
            })
            .support_invalidation_closures()
            .build();

        // Build provider responses cache
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

        // Build sync batches cache
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

        Ok(Self {
            embeddings_cache,
            search_results_cache,
            metadata_cache,
            provider_responses_cache,
            sync_batches_cache,
            stats_hits: Arc::new(AtomicU64::new(0)),
            stats_misses: Arc::new(AtomicU64::new(0)),
            stats_evictions,
        })
    }

    /// Get the cache for a specific namespace
    #[inline]
    fn get_cache(&self, namespace: &str) -> &Cache<String, Vec<u8>> {
        match namespace {
            "embeddings" => &self.embeddings_cache,
            "search_results" => &self.search_results_cache,
            "metadata" => &self.metadata_cache,
            "provider_responses" => &self.provider_responses_cache,
            "sync_batches" => &self.sync_batches_cache,
            _ => &self.metadata_cache, // Default to metadata for unknown namespaces
        }
    }

    /// Create a full cache key combining namespace and key
    #[inline]
    fn full_key(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace, key)
    }
}

#[async_trait]
impl CacheProvider for MokaCacheProvider {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let cache = self.get_cache(namespace);
        let full_key = Self::full_key(namespace, key);

        match cache.get(&full_key).await {
            Some(value) => {
                self.stats_hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(value))
            }
            None => {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn set(&self, namespace: &str, key: &str, value: Vec<u8>, _ttl: Duration) -> Result<()> {
        let cache = self.get_cache(namespace);
        let full_key = Self::full_key(namespace, key);
        cache.insert(full_key, value).await;
        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let cache = self.get_cache(namespace);
        let full_key = Self::full_key(namespace, key);
        cache.invalidate(&full_key).await;
        Ok(())
    }

    async fn clear(&self, namespace: Option<&str>) -> Result<()> {
        match namespace {
            Some(ns) => {
                let cache = self.get_cache(ns);
                let prefix = format!("{}:", ns);
                if let Err(e) = cache.invalidate_entries_if(move |k, _v| k.starts_with(&prefix)) {
                    tracing::warn!("[CACHE] Failed to invalidate namespace {}: {}", ns, e);
                }
            }
            None => {
                // Clear all namespaces
                for cache in [
                    &self.embeddings_cache,
                    &self.search_results_cache,
                    &self.metadata_cache,
                    &self.provider_responses_cache,
                    &self.sync_batches_cache,
                ] {
                    cache.invalidate_all();
                }
            }
        }
        Ok(())
    }

    async fn get_stats(&self, namespace: &str) -> Result<CacheStats> {
        let cache = self.get_cache(namespace);
        let hits = self.stats_hits.load(Ordering::Relaxed);
        let misses = self.stats_misses.load(Ordering::Relaxed);

        let hit_ratio = if hits + misses > 0 {
            hits as f64 / (hits + misses) as f64
        } else {
            0.0
        };

        // Note: Moka doesn't directly expose memory usage or entry count
        // This is a simplified implementation - production deployments might
        // want to track these metrics explicitly
        Ok(CacheStats {
            total_entries: cache.entry_count() as usize,
            total_size_bytes: 0, // Would need explicit tracking
            hits,
            misses,
            hit_ratio,
            evictions: self.stats_evictions.load(Ordering::Relaxed),
            avg_access_time_us: 0.0, // Would need explicit tracking
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        // Moka in-memory cache is always healthy unless it has too many evictions
        let evictions = self.stats_evictions.load(Ordering::Relaxed);
        let hits = self.stats_hits.load(Ordering::Relaxed);

        // If we have high eviction rate (more evictions than hits), cache might be too small
        if hits > 0 && evictions > hits * 2 {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Healthy)
        }
    }

    fn backend_type(&self) -> String {
        "moka".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CacheConfig {
        CacheConfig::default()
    }

    #[tokio::test]
    async fn test_moka_provider_set_and_get() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();
        let namespace = "test";
        let key = "test_key";
        let value = vec![1, 2, 3, 4, 5];

        provider
            .set(namespace, key, value.clone(), Duration::from_secs(60))
            .await
            .unwrap();

        let retrieved = provider.get(namespace, key).await.unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_moka_provider_delete() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();
        let namespace = "test";
        let key = "test_key";
        let value = vec![1, 2, 3];

        provider
            .set(namespace, key, value, Duration::from_secs(60))
            .await
            .unwrap();

        provider.delete(namespace, key).await.unwrap();

        let retrieved = provider.get(namespace, key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_moka_provider_clear_namespace() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();
        let namespace = "embeddings";

        provider
            .set(namespace, "key1", vec![1, 2], Duration::from_secs(60))
            .await
            .unwrap();
        provider
            .set(namespace, "key2", vec![3, 4], Duration::from_secs(60))
            .await
            .unwrap();

        provider.clear(Some(namespace)).await.unwrap();

        assert_eq!(provider.get(namespace, "key1").await.unwrap(), None);
        assert_eq!(provider.get(namespace, "key2").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_moka_provider_clear_all() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();

        provider
            .set("embeddings", "key1", vec![1], Duration::from_secs(60))
            .await
            .unwrap();
        provider
            .set("search_results", "key2", vec![2], Duration::from_secs(60))
            .await
            .unwrap();

        provider.clear(None).await.unwrap();

        assert_eq!(provider.get("embeddings", "key1").await.unwrap(), None);
        assert_eq!(provider.get("search_results", "key2").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_moka_provider_health_check() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();
        let health = provider.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_moka_provider_stats() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();

        provider
            .set("test", "key1", vec![1, 2, 3], Duration::from_secs(60))
            .await
            .unwrap();
        provider.get("test", "key1").await.unwrap(); // Hit
        provider.get("test", "nonexistent").await.unwrap(); // Miss

        let stats = provider.get_stats("test").await.unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_ratio > 0.0);
    }

    #[tokio::test]
    async fn test_moka_provider_backend_type() {
        let provider = MokaCacheProvider::new(test_config()).unwrap();
        assert_eq!(provider.backend_type(), "moka".to_string());
    }
}
