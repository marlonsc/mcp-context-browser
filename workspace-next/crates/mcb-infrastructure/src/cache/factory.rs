//! Cache provider factory
//!
//! Factory for creating cache provider instances based on configuration.

use crate::cache::config::*;
use crate::cache::provider::{SharedCacheProvider, CacheProviderType};
use crate::cache::providers::*;
use crate::config::data::*;
use mcb_domain::error::{Error, Result};

/// Cache provider factory
#[derive(Clone)]
pub struct CacheProviderFactory;

impl CacheProviderFactory {
    /// Create a cache provider from configuration
    pub async fn create_from_config(config: &CacheConfig) -> Result<SharedCacheProvider> {
        use crate::cache::provider::CacheProviderType;

        if !config.enabled {
            return Ok(SharedCacheProvider::new(CacheProviderType::Null(NullCacheProvider::new())));
        }

        let provider = match config.provider {
            crate::config::data::CacheProvider::Moka => {
                CacheProviderType::Moka(MokaCacheProvider::with_capacity(config.max_size))
            }
            crate::config::data::CacheProvider::Redis => {
                let redis_url = config.redis_url.as_deref()
                    .unwrap_or("redis://localhost:6379");
                CacheProviderType::Redis(RedisCacheProvider::new(redis_url)?)
            }
        };

        let mut shared_provider = SharedCacheProvider::new(provider);
        shared_provider.set_namespace(&config.namespace);

        Ok(shared_provider)
    }

    /// Create a Moka cache provider
    pub fn create_moka(max_size: usize) -> SharedCacheProvider {
        use crate::cache::provider::CacheProviderType;
        SharedCacheProvider::new(CacheProviderType::Moka(MokaCacheProvider::with_capacity(max_size)))
    }

    /// Create a Redis cache provider
    pub async fn create_redis(connection_string: &str) -> Result<SharedCacheProvider> {
        use crate::cache::provider::CacheProviderType;
        let provider = RedisCacheProvider::new(connection_string)?;
        Ok(SharedCacheProvider::new(CacheProviderType::Redis(provider)))
    }

    /// Create a null cache provider (for testing/disabling cache)
    pub fn create_null() -> SharedCacheProvider {
        use crate::cache::provider::CacheProviderType;
        SharedCacheProvider::new(CacheProviderType::Null(NullCacheProvider::new()))
    }

    /// Create a cache provider with specific namespace
    pub fn with_namespace(provider: SharedCacheProvider, namespace: &str) -> SharedCacheProvider {
        provider.namespaced(namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_null_provider() {
        let provider = CacheProviderFactory::create_null();

        // Test basic operations
        assert!(provider.set("test", "value", CacheEntryConfig::default()).await.is_ok());
        let result: Option<String> = provider.get("test").await.unwrap();
        assert!(result.is_none()); // Null provider always returns None
    }

    #[tokio::test]
    async fn test_factory_moka_provider() {
        let provider = CacheProviderFactory::create_moka(1024 * 1024); // 1MB

        // Test basic operations
        assert!(provider.set("test", "value", CacheEntryConfig::default()).await.is_ok());
        let result: Option<String> = provider.get("test").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_factory_redis_provider() {
        let provider = CacheProviderFactory::create_redis("redis://localhost:6379").await.unwrap();

        // Test basic operations
        assert!(provider.set("test", "value", CacheEntryConfig::default()).await.is_ok());
        let result: Option<String> = provider.get("test").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_factory_from_config_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };

        let provider = CacheProviderFactory::create_from_config(&config).await.unwrap();

        // Should be null provider when disabled
        let result: Option<String> = provider.get("test").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_factory_from_config_moka() {
        let config = CacheConfig {
            enabled: true,
            provider: crate::config::data::CacheProvider::Moka,
            max_size: 1024 * 1024,
            namespace: "test".to_string(),
            ..Default::default()
        };

        let provider = CacheProviderFactory::create_from_config(&config).await.unwrap();

        // Test that it works and has namespace
        assert!(provider.set("key", "value", CacheEntryConfig::default()).await.is_ok());
        let result: Option<String> = provider.get("key").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }
}