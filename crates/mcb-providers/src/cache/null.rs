//! Null cache provider for testing
//!
//! A cache provider implementation that doesn't store anything.
//! Useful for testing and disabling caching.

use async_trait::async_trait;
use mcb_application::ports::providers::cache::{CacheEntryConfig, CacheProvider, CacheStats};
use mcb_domain::error::Result;

/// Null cache provider that doesn't store anything
///
/// This provider always returns None for gets and accepts all sets
/// without storing the data. Useful for testing and disabling caching.
///
/// # Example
///
/// ```rust
/// use mcb_providers::cache::NullCacheProvider;
///
/// let provider = NullCacheProvider::new();
/// // All operations succeed but nothing is cached
/// ```
#[derive(Debug, Clone, shaku::Component)]
#[shaku(interface = CacheProvider)]
pub struct NullCacheProvider;

impl NullCacheProvider {
    /// Create a new null cache provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullCacheProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
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

    fn provider_name(&self) -> &str {
        "null"
    }
}
