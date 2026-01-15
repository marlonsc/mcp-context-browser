//! Moka in-memory cache provider
//!
//! High-performance, concurrent in-memory cache implementation using Moka.

use crate::cache::config::{CacheEntryConfig, CacheStats};
use crate::cache::provider::CacheProvider;
use crate::constants::*;
use async_trait::async_trait;
use mcb_domain::error::Result;
use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

/// Moka-based in-memory cache provider
#[derive(Clone)]
pub struct MokaCacheProvider {
    cache: Cache<String, Vec<u8>>,
    max_size: usize,
}

impl MokaCacheProvider {
    /// Create a new Moka cache provider with default settings
    pub fn new() -> Self {
        Self::with_capacity(CACHE_DEFAULT_SIZE_LIMIT)
    }

    /// Create a new Moka cache provider with specified capacity
    pub fn with_capacity(max_size: usize) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_size as u64)
            .build();

        Self { cache, max_size }
    }

    /// Create a new Moka cache provider with custom configuration
    pub fn with_config(max_size: usize, time_to_live: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_size as u64)
            .time_to_live(time_to_live)
            .build();

        Self { cache, max_size }
    }

    /// Serialize a value to bytes
    fn serialize_value<V: Serialize>(value: &V) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| mcb_domain::error::Error::Cache {
            message: format!("Failed to serialize cache value: {}", e),
        })
    }

    /// Deserialize bytes to a value
    fn deserialize_value<V: DeserializeOwned>(bytes: &[u8]) -> Result<V> {
        serde_json::from_slice(bytes).map_err(|e| mcb_domain::error::Error::Cache {
            message: format!("Failed to deserialize cache value: {}", e),
        })
    }
}

#[async_trait::async_trait]
impl CacheProvider for MokaCacheProvider {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        if let Some(bytes) = self.cache.get(key).await {
            let json = String::from_utf8(bytes)
                .map_err(|e| mcb_domain::error::Error::Infrastructure {
                    message: format!("Invalid UTF-8 in cached value: {}", e),
                    source: Some(Box::new(e)),
                })?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    async fn set_json(&self, key: &str, value: &str, _config: CacheEntryConfig) -> Result<()> {
        let bytes = value.as_bytes();

        // Check if the value exceeds our size limit
        if bytes.len() > self.max_size {
            return Err(mcb_domain::error::Error::Infrastructure {
                message: format!(
                    "Cache value size {} exceeds maximum size {}",
                    bytes.len(),
                    self.max_size
                ),
                source: None,
            });
        }

        self.cache.insert(key.to_string(), bytes.to_vec()).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let existed = self.cache.contains_key(key);
        self.cache.invalidate(key).await;
        Ok(existed)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        // Moka doesn't provide detailed stats, so we return basic info
        let entries = self.cache.entry_count();

        Ok(CacheStats {
            hits: 0,    // Moka doesn't track hits/misses
            misses: 0,  // Moka doesn't track hits/misses
            entries,
            hit_rate: 0.0, // Unknown
            bytes_used: 0,  // Unknown
        })
    }

    async fn size(&self) -> Result<usize> {
        Ok(self.cache.entry_count() as usize)
    }
}

impl std::fmt::Debug for MokaCacheProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MokaCacheProvider")
            .field("max_size", &self.max_size)
            .field("entries", &self.cache.entry_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestValue {
        data: String,
        number: i32,
    }

    #[tokio::test]
    async fn test_moka_provider_basic_operations() {
        let provider = MokaCacheProvider::new();

        let value = TestValue {
            data: "test data".to_string(),
            number: 42,
        };

        // Test set and get
        provider.set("test_key", &value, CacheEntryConfig::default()).await.unwrap();

        let retrieved: Option<TestValue> = provider.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Test exists
        assert!(provider.exists("test_key").await.unwrap());

        // Test delete
        assert!(provider.delete("test_key").await.unwrap());
        assert!(!provider.exists("test_key").await.unwrap());

        // Test get after delete
        let retrieved: Option<TestValue> = provider.get("test_key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_moka_provider_nonexistent_key() {
        let provider = MokaCacheProvider::new();

        let retrieved: Option<TestValue> = provider.get("nonexistent").await.unwrap();
        assert!(retrieved.is_none());

        assert!(!provider.exists("nonexistent").await.unwrap());
        assert!(!provider.delete("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_moka_provider_clear() {
        let provider = MokaCacheProvider::new();

        // Add some entries
        provider.set("key1", "value1", CacheEntryConfig::default()).await.unwrap();
        provider.set("key2", "value2", CacheEntryConfig::default()).await.unwrap();

        assert_eq!(provider.size().await.unwrap(), 2);

        // Clear cache
        provider.clear().await.unwrap();

        assert_eq!(provider.size().await.unwrap(), 0);
        assert!(!provider.exists("key1").await.unwrap());
        assert!(!provider.exists("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_moka_provider_stats() {
        let provider = MokaCacheProvider::new();

        provider.set("key1", "value1", CacheEntryConfig::default()).await.unwrap();
        provider.set("key2", "value2", CacheEntryConfig::default()).await.unwrap();

        let stats = provider.stats().await.unwrap();
        assert_eq!(stats.entries, 2);
    }
}