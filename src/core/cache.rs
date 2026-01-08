//! Advanced distributed caching system with Redis
//!
//! Provides high-performance caching for embeddings, search results, and metadata
//! with intelligent TTL management and cache invalidation strategies.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Default TTL for cache entries (seconds)
    pub default_ttl_seconds: u64,
    /// Maximum cache size (number of entries)
    pub max_size: usize,
    /// Whether caching is enabled
    pub enabled: bool,
    /// Cache namespaces configuration
    pub namespaces: CacheNamespacesConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_ttl_seconds: 3600, // 1 hour
            max_size: 10000,
            enabled: true,
            namespaces: CacheNamespacesConfig::default(),
        }
    }
}

/// Configuration for different cache namespaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheNamespacesConfig {
    /// Embedding cache settings
    pub embeddings: CacheNamespaceConfig,
    /// Search results cache settings
    pub search_results: CacheNamespaceConfig,
    /// Metadata cache settings
    pub metadata: CacheNamespaceConfig,
    /// Provider responses cache settings
    pub provider_responses: CacheNamespaceConfig,
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
        }
    }
}

/// Configuration for a specific cache namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheNamespaceConfig {
    /// TTL for entries in this namespace (seconds)
    pub ttl_seconds: u64,
    /// Maximum number of entries for this namespace
    pub max_entries: usize,
    /// Whether to compress entries
    pub compression: bool,
}

/// Cache entry with metadata
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
    redis_client: Option<redis::Client>,
    local_cache: Arc<RwLock<HashMap<String, CacheEntry<serde_json::Value>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: CacheConfig) -> Result<Self> {
        let redis_client = if config.enabled {
            match redis::Client::open(config.redis_url.clone()) {
                Ok(client) => {
                    // Test connection
                    match Self::test_redis_connection(&client).await {
                        Ok(_) => {
                            tracing::info!("✅ Redis cache connection established");
                            Some(client)
                        }
                        Err(e) => {
                            tracing::warn!(
                                "⚠️  Redis cache connection failed, falling back to local cache: {}",
                                e
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️  Redis client creation failed, using local cache: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("ℹ️  Caching disabled");
            None
        };

        let manager = Self {
            config,
            redis_client,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        };

        // Start background cleanup task
        if manager.config.enabled {
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                manager_clone.background_cleanup().await;
            });
        }

        Ok(manager)
    }

    /// Test Redis connection
    async fn test_redis_connection(client: &redis::Client) -> Result<()> {
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::generic(format!("Redis connection failed: {}", e)))?;

        redis::cmd("PING")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| Error::generic(format!("Redis PING failed: {}", e)))?;

        Ok(())
    }

    /// Get a value from cache
    pub async fn get<T>(&self, namespace: &str, key: &str) -> CacheResult<T>
    where
        T: for<'de> serde::Deserialize<'de> + Clone,
    {
        if !self.config.enabled {
            return CacheResult::Miss;
        }

        let start_time = Instant::now();
        let full_key = format!("{}:{}", namespace, key);

        // Try Redis first if available
        if self.redis_client.is_some() {
            match self.get_from_redis(&full_key).await {
                Ok(Some(data)) => match serde_json::from_value(data) {
                    Ok(deserialized) => {
                        self.update_stats(true, start_time.elapsed()).await;
                        return CacheResult::Hit(deserialized);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached data: {}", e);
                    }
                },
                Ok(None) => {} // Continue to local cache
                Err(e) => {
                    tracing::warn!("Redis get failed: {}", e);
                }
            }
        }

        // Try local cache
        match self.get_from_local(&full_key).await {
            Ok(Some(data)) => match serde_json::from_value(data) {
                Ok(deserialized) => {
                    self.update_stats(true, start_time.elapsed()).await;
                    CacheResult::Hit(deserialized)
                }
                Err(e) => {
                    self.update_stats(false, start_time.elapsed()).await;
                    CacheResult::Error(Error::generic(format!(
                        "Failed to deserialize cached data: {}",
                        e
                    )))
                }
            },
            Ok(None) => {
                self.update_stats(false, start_time.elapsed()).await;
                CacheResult::Miss
            }
            Err(e) => {
                self.update_stats(false, start_time.elapsed()).await;
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

        let size_bytes = serde_json::to_string(&data).map(|s| s.len()).unwrap_or(0);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::generic(format!("System time error: {}", e)))?
            .as_secs();

        let entry = CacheEntry {
            data: data.clone(),
            created_at: now,
            accessed_at: now,
            access_count: 1,
            size_bytes,
        };

        // Store in Redis if available
        if self.redis_client.is_some()
            && let Err(e) = self.set_in_redis(&full_key, &entry, ttl).await
        {
            tracing::warn!("Redis set failed: {}", e);
        }

        // Store in local cache
        self.set_in_local(&full_key, entry).await?;

        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Delete from Redis
        if self.redis_client.is_some()
            && let Err(e) = self.delete_from_redis(&full_key).await
        {
            tracing::warn!("Redis delete failed: {}", e);
        }

        // Delete from local cache
        self.delete_from_local(&full_key).await?;

        Ok(())
    }

    /// Clear entire namespace
    pub async fn clear_namespace(&self, namespace: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let pattern = format!("{}:*", namespace);

        // Clear from Redis
        if self.redis_client.is_some()
            && let Err(e) = self.clear_namespace_redis(&pattern).await
        {
            tracing::warn!("Redis namespace clear failed: {}", e);
        }

        // Clear from local cache
        self.clear_namespace_local(namespace).await?;

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
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
            _ => &self.config.namespaces.metadata, // Default to metadata config
        }
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

    async fn get_from_local(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let mut cache = self.local_cache.write().await;
        if let Some(entry) = cache.get_mut(key) {
            entry.accessed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Error::generic(format!("System time error: {}", e)))?
                .as_secs();
            entry.access_count += 1;
            return Ok(Some(entry.data.clone()));
        }
        Ok(None)
    }

    async fn set_in_local(&self, key: &str, entry: CacheEntry<serde_json::Value>) -> Result<()> {
        let mut cache = self.local_cache.write().await;

        // Check size limits
        if cache.len() >= self.config.max_size {
            self.evict_entries(&mut cache).await;
        }

        cache.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete_from_local(&self, key: &str) -> Result<()> {
        let mut cache = self.local_cache.write().await;
        cache.remove(key);
        Ok(())
    }

    async fn clear_namespace_local(&self, namespace: &str) -> Result<()> {
        let mut cache = self.local_cache.write().await;
        let prefix = format!("{}:", namespace);
        cache.retain(|key, _| !key.starts_with(&prefix));
        Ok(())
    }

    async fn evict_entries(&self, cache: &mut HashMap<String, CacheEntry<serde_json::Value>>) {
        // Simple LRU eviction: remove least recently accessed entries
        let mut entries: Vec<String> = cache.keys().cloned().collect();
        entries.sort_by_key(|key| {
            cache.get(key).map(|entry| entry.accessed_at).unwrap_or(0) // Fallback to 0 if key not found (shouldn't happen)
        });

        // Remove oldest 10% of entries
        let to_remove = (cache.len() / 10).max(1);
        for key in entries.into_iter().take(to_remove) {
            cache.remove(&key);
        }

        let mut stats = self.stats.write().await;
        stats.evictions += to_remove as u64;
    }

    async fn update_stats(&self, is_hit: bool, duration: Duration) {
        let mut stats = self.stats.write().await;
        if is_hit {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }

        let total_requests = stats.hits + stats.misses;
        if total_requests > 0 {
            stats.hit_ratio = stats.hits as f64 / total_requests as f64;
        }

        // Update average access time (exponential moving average)
        let access_time_us = duration.as_micros() as f64;
        if stats.hits + stats.misses == 1 {
            stats.avg_access_time_us = access_time_us;
        } else {
            stats.avg_access_time_us = 0.9 * stats.avg_access_time_us + 0.1 * access_time_us;
        }
    }

    async fn background_cleanup(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            interval.tick().await;

            if let Err(e) = self.cleanup_expired_entries().await {
                tracing::warn!("Cache cleanup failed: {}", e);
            }
        }
    }

    async fn cleanup_expired_entries(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::generic(format!("System time error: {}", e)))?
            .as_secs();

        let mut cache = self.local_cache.write().await;
        let mut to_remove = Vec::new();

        for (key, entry) in cache.iter() {
            let namespace = key.split(':').next().unwrap_or("");
            let namespace_config = self.get_namespace_config(namespace);
            let ttl = namespace_config.ttl_seconds;

            if now.saturating_sub(entry.created_at) > ttl {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            cache.remove(&key);
        }

        Ok(())
    }
}

/// Global cache manager instance
static CACHE_MANAGER: std::sync::OnceLock<CacheManager> = std::sync::OnceLock::new();

/// Initialize global cache manager
pub async fn init_global_cache_manager(config: CacheConfig) -> Result<()> {
    let manager = CacheManager::new(config).await?;
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
    }

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        let manager = CacheManager::new(config).await.unwrap();
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
            redis_url: "redis://nonexistent:6379".to_string(), // Force local cache
            ..Default::default()
        };

        let manager = CacheManager::new(config).await.unwrap();

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
            redis_url: "redis://nonexistent:6379".to_string(),
            ..Default::default()
        };

        let manager = CacheManager::new(config).await.unwrap();

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

        let result1: CacheResult<String> = manager.get("ns1", "key1").await;
        let result2: CacheResult<String> = manager.get("ns2", "key1").await;

        assert!(result1.is_miss());
        assert!(result2.is_hit());
    }

    #[tokio::test]
    async fn test_cache_manager_handles_connection_failures() {
        // Test with invalid Redis configuration
        let config = CacheConfig {
            redis_url: "redis://invalid:6379".to_string(),
            default_ttl_seconds: 300,
            max_size: 100,
            enabled: true,
            namespaces: CacheNamespacesConfig::default(),
        };

        // Should handle Redis connection failures gracefully
        let result = CacheManager::new(config).await;
        // The implementation should either succeed with fallback or return a proper error
        // We accept both outcomes as long as there's no panic
        match result {
            Ok(manager) => {
                // If it succeeds, it should have fallen back to local cache
                assert!(manager.is_enabled());
            }
            Err(e) => {
                // If it fails, it should be a proper error, not a panic
                assert!(matches!(
                    e,
                    crate::core::error::Error::Redis { .. } | crate::core::error::Error::Generic(_)
                ));
            }
        }
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

        let manager = CacheManager::new(config).await.unwrap();

        // Operations on disabled cache should not panic
        let set_result = manager.set("test", "key", "value".to_string()).await;
        assert!(set_result.is_ok()); // Should succeed (no-op) or return proper error

        let get_result: CacheResult<String> = manager.get("test", "key").await;
        // CacheResult doesn't have is_ok, check it's not an error
        assert!(!matches!(get_result, CacheResult::Error(_)));
    }

    #[tokio::test]
    async fn test_cache_manager_handles_namespace_operations() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        // These operations should not panic
        let clear_result = manager.clear_namespace("test_ns").await;
        assert!(clear_result.is_ok());

        let delete_result = manager.delete("test_ns", "key").await;
        assert!(delete_result.is_ok());

        // Note: get_namespace_stats doesn't exist, skip that test
    }

    #[tokio::test]
    async fn test_cache_manager_handles_large_data_operations() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config).await.unwrap();

        // Test with large data that might cause issues
        let large_data = "x".repeat(1024 * 1024); // 1MB string

        let set_result = manager.set("test", "large_key", large_data.clone()).await;
        assert!(set_result.is_ok());

        let get_result: CacheResult<String> = manager.get("test", "large_key").await;
        // Check it's not an error and contains the data
        match get_result {
            CacheResult::Hit(data) => assert_eq!(data, large_data),
            CacheResult::Miss => panic!("Expected cache hit"),
            CacheResult::Error(_) => panic!("Expected no error"),
        }
    }
}
