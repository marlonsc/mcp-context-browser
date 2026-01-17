//! Cache provider factory
//!
//! Factory for creating cache provider instances based on configuration.
//! Uses cache provider implementations from mcb-providers crate.
//!
//! **ARCHITECTURE**: Creates providers directly, not wrapped in enum.
//! Follows OCP - new providers can be added without modifying existing code.

use crate::cache::provider::SharedCacheProvider;
use crate::config::data::*;
use mcb_domain::error::Result;
use mcb_providers::cache::{MokaCacheProvider, NullCacheProvider, RedisCacheProvider};

/// Cache provider factory
#[derive(Clone)]
pub struct CacheProviderFactory;

impl CacheProviderFactory {
    /// Create a cache provider from configuration
    pub async fn create_from_config(config: &CacheConfig) -> Result<SharedCacheProvider> {
        if !config.enabled {
            return Ok(SharedCacheProvider::new(NullCacheProvider::new()));
        }

        let mut shared_provider = match config.provider {
            crate::config::data::CacheProvider::Moka => {
                SharedCacheProvider::new(MokaCacheProvider::with_capacity(config.max_size))
            }
            crate::config::data::CacheProvider::Redis => {
                let redis_url = config
                    .redis_url
                    .as_deref()
                    .unwrap_or("redis://localhost:6379");
                SharedCacheProvider::new(RedisCacheProvider::new(redis_url)?)
            }
        };

        shared_provider.set_namespace(&config.namespace);
        Ok(shared_provider)
    }

    /// Create a Moka cache provider
    pub fn create_moka(max_size: usize) -> SharedCacheProvider {
        SharedCacheProvider::new(MokaCacheProvider::with_capacity(max_size))
    }

    /// Create a Redis cache provider
    pub async fn create_redis(connection_string: &str) -> Result<SharedCacheProvider> {
        let provider = RedisCacheProvider::new(connection_string)?;
        Ok(SharedCacheProvider::new(provider))
    }

    /// Create a null cache provider (for testing/disabling cache)
    pub fn create_null() -> SharedCacheProvider {
        SharedCacheProvider::new(NullCacheProvider::new())
    }

    /// Create a cache provider with specific namespace
    pub fn with_namespace(
        mut provider: SharedCacheProvider,
        namespace: &str,
    ) -> SharedCacheProvider {
        provider.set_namespace(namespace);
        provider
    }
}
