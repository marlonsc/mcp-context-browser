//! Null cache provider for testing
//!
//! A cache provider implementation that doesn't store anything.
//! Useful for testing and disabling caching.

use crate::cache::config::{CacheEntryConfig, CacheStats};
use crate::cache::provider::CacheProvider;
use async_trait::async_trait;
use mcb_domain::error::Result;

/// Null cache provider that doesn't store anything
///
/// This provider always returns None for gets and accepts all sets
/// without storing the data. Useful for testing and disabling caching.
#[derive(Debug, Clone)]
pub struct NullCacheProvider;

impl NullCacheProvider {
    /// Create a new null cache provider
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl CacheProvider for NullCacheProvider {
    async fn get_json(&self, _key: &str) -> Result<Option<String>> {
        // Always return None (cache miss)
        Ok(None)
    }

    async fn set_json(&self, _key: &str, _value: &str, _config: CacheEntryConfig) -> Result<()> {
        // Accept the set operation but don't store anything
        Ok(())
    }

    async fn delete(&self, _key: &str) -> Result<bool> {
        // Return false (key didn't exist)
        Ok(false)
    }

    async fn exists(&self, _key: &str) -> Result<bool> {
        // Always return false (key doesn't exist)
        Ok(false)
    }

    async fn clear(&self) -> Result<()> {
        // Nothing to clear
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        // Return empty stats
        Ok(CacheStats::new())
    }

    async fn size(&self) -> Result<usize> {
        // Always empty
        Ok(0)
    }
}

impl Default for NullCacheProvider {
    fn default() -> Self {
        Self::new()
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
    async fn test_null_provider_operations() {
        let provider = NullCacheProvider::new();

        // Test get (should always return None)
        let result: Option<TestValue> = provider.get("test_key").await.unwrap();
        assert!(result.is_none());

        // Test set (should succeed)
        let value = TestValue {
            data: "test".to_string(),
            number: 42,
        };
        assert!(provider.set("test_key", value, CacheEntryConfig::default()).await.is_ok());

        // Test exists (should always return false)
        assert_eq!(provider.exists("test_key").await.unwrap(), false);

        // Test delete (should return false)
        assert_eq!(provider.delete("test_key").await.unwrap(), false);

        // Test clear (should succeed)
        assert!(provider.clear().await.is_ok());

        // Test stats (should be empty)
        let stats = provider.stats().await.unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.entries, 0);

        // Test size (should be 0)
        assert_eq!(provider.size().await.unwrap(), 0);
    }
}