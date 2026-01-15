//! Cache provider interface and shared cache provider
//!
//! Defines the cache provider trait and shared cache provider implementation
//! for unified cache access across the application.

use crate::cache::config::{CacheEntryConfig, CacheStats};
use async_trait::async_trait;
use mcb_domain::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// Cache provider types
#[derive(Clone)]
pub enum CacheProviderType {
    /// Moka in-memory cache
    Moka(MokaCacheProvider),
    /// Redis distributed cache
    Redis(RedisCacheProvider),
    /// Null cache (no-op)
    Null(NullCacheProvider),
}

#[async_trait::async_trait]
impl CacheProvider for CacheProviderType {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        match self {
            CacheProviderType::Moka(provider) => provider.get_json(key).await,
            CacheProviderType::Redis(provider) => provider.get_json(key).await,
            CacheProviderType::Null(provider) => provider.get_json(key).await,
        }
    }

    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        match self {
            CacheProviderType::Moka(provider) => provider.set_json(key, value, config).await,
            CacheProviderType::Redis(provider) => provider.set_json(key, value, config).await,
            CacheProviderType::Null(provider) => provider.set_json(key, value, config).await,
        }
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        match self {
            CacheProviderType::Moka(provider) => provider.delete(key).await,
            CacheProviderType::Redis(provider) => provider.delete(key).await,
            CacheProviderType::Null(provider) => provider.delete(key).await,
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self {
            CacheProviderType::Moka(provider) => provider.exists(key).await,
            CacheProviderType::Redis(provider) => provider.exists(key).await,
            CacheProviderType::Null(provider) => provider.exists(key).await,
        }
    }

    async fn clear(&self) -> Result<()> {
        match self {
            CacheProviderType::Moka(provider) => provider.clear().await,
            CacheProviderType::Redis(provider) => provider.clear().await,
            CacheProviderType::Null(provider) => provider.clear().await,
        }
    }

    async fn stats(&self) -> Result<CacheStats> {
        match self {
            CacheProviderType::Moka(provider) => provider.stats().await,
            CacheProviderType::Redis(provider) => provider.stats().await,
            CacheProviderType::Null(provider) => provider.stats().await,
        }
    }

    async fn size(&self) -> Result<usize> {
        match self {
            CacheProviderType::Moka(provider) => provider.size().await,
            CacheProviderType::Redis(provider) => provider.size().await,
            CacheProviderType::Null(provider) => provider.size().await,
        }
    }
}

impl std::fmt::Debug for CacheProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheProviderType::Moka(_) => write!(f, "CacheProviderType::Moka"),
            CacheProviderType::Redis(_) => write!(f, "CacheProviderType::Redis"),
            CacheProviderType::Null(_) => write!(f, "CacheProviderType::Null"),
        }
    }
}

/// Cache provider trait (legacy - use CacheProviderType enum instead)
#[async_trait::async_trait]
pub trait CacheProvider: Send + Sync + std::fmt::Debug {
    /// Get a value from the cache as JSON string
    async fn get_json(&self, key: &str) -> Result<Option<String>>;

    /// Set a value in the cache from JSON string
    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Clear all values from the cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;

    /// Get the cache size (number of entries)
    async fn size(&self) -> Result<usize>;
}

/// Shared cache provider wrapper
///
/// This provides thread-safe access to a cache provider and includes
/// additional features like metrics collection and error handling.
#[derive(Clone)]
pub struct SharedCacheProvider {
    provider: Arc<dyn CacheProvider>,
    namespace: Option<String>,
}

impl SharedCacheProvider {
    /// Create a new shared cache provider
    pub fn new(provider: CacheProviderType) -> Self {
        Self {
            provider: Arc::new(provider),
            namespace: None,
        }
    }

    /// Create a new shared cache provider with a default namespace
    pub fn with_namespace<S: Into<String>>(provider: CacheProviderType, namespace: S) -> Self {
        Self {
            provider: Arc::new(provider),
            namespace: Some(namespace.into()),
        }
    }

    /// Set the default namespace for this provider
    pub fn set_namespace<S: Into<String>>(&mut self, namespace: S) {
        self.namespace = Some(namespace.into());
    }

    /// Clear the default namespace
    pub fn clear_namespace(&mut self) {
        self.namespace = None;
    }

    /// Get a namespaced key
    fn namespaced_key<K: AsRef<str>>(&self, key: K) -> String {
        if let Some(ns) = &self.namespace {
            format!("{}:{}", ns, key.as_ref())
        } else {
            key.as_ref().to_string()
        }
    }

    /// Get a typed value from the cache
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        let namespaced_key = self.namespaced_key(key);
        match self.provider.get_json(&namespaced_key).await? {
            Some(json) => {
                let value: T = serde_json::from_str(&json)
                    .map_err(|e| mcb_domain::error::Error::Infrastructure {
                        message: format!("Failed to deserialize cached value: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a typed value in the cache
    pub async fn set<T>(&self, key: &str, value: &T, config: CacheEntryConfig) -> Result<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let namespaced_key = self.namespaced_key(key);
        let json = serde_json::to_string(value)
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Failed to serialize value for cache: {}", e),
                source: Some(Box::new(e)),
            })?;
        self.provider.set_json(&namespaced_key, &json, config).await
    }

    /// Set a JSON value in the cache
    pub async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        let namespaced_key = self.namespaced_key(key);
        self.provider.set_json(&namespaced_key, value, config).await
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let namespaced_key = self.namespaced_key(key);
        self.provider.delete(&namespaced_key).await
    }

    /// Check if a key exists in the cache
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let namespaced_key = self.namespaced_key(key);
        self.provider.exists(&namespaced_key).await
    }

    /// Clear all values from the cache
    pub async fn clear(&self) -> Result<()> {
        self.provider.clear().await
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        self.provider.stats().await
    }

    /// Get the cache size
    pub async fn size(&self) -> Result<usize> {
        self.provider.size().await
    }


    /// Create a namespaced view of this cache provider
    pub fn namespaced<S: Into<String>>(&self, namespace: S) -> NamespacedCacheProvider {
        NamespacedCacheProvider {
            provider: Arc::clone(&self.provider),
            namespace: namespace.into(),
        }
    }
}

impl fmt::Debug for SharedCacheProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedCacheProvider")
            .field("namespace", &self.namespace)
            .finish()
    }
}

/// Namespaced cache provider view
///
/// Provides access to a cache provider within a specific namespace.
#[derive(Clone)]
pub struct NamespacedCacheProvider {
    provider: Arc<dyn CacheProvider>,
    namespace: String,
}

impl NamespacedCacheProvider {
    /// Get a typed value from the cache within this namespace
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        let namespaced_key = format!("{}:{}", self.namespace, key);
        match self.provider.get_json(&namespaced_key).await? {
            Some(json) => {
                let value: T = serde_json::from_str(&json)
                    .map_err(|e| mcb_domain::error::Error::Infrastructure {
                        message: format!("Failed to deserialize cached value: {}", e),
                        source: Some(Box::new(e)),
                    })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a typed value in the cache within this namespace
    pub async fn set<T>(&self, key: &str, value: &T, config: CacheEntryConfig) -> Result<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let namespaced_key = format!("{}:{}", self.namespace, key);
        let json = serde_json::to_string(value)
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Failed to serialize value for cache: {}", e),
                source: Some(Box::new(e)),
            })?;
        self.provider.set_json(&namespaced_key, &json, config).await
    }

    /// Delete a value from the cache within this namespace
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let namespaced_key = format!("{}:{}", self.namespace, key);
        self.provider.delete(&namespaced_key).await
    }

    /// Check if a key exists in the cache within this namespace
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let namespaced_key = format!("{}:{}", self.namespace, key);
        self.provider.exists(&namespaced_key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::providers::NullCacheProvider;

    #[tokio::test]
    async fn test_shared_cache_provider_basic_operations() {
        let provider = SharedCacheProvider::new(NullCacheProvider::new());

        // Test basic operations (NullCacheProvider always returns None/Ok)
        assert_eq!(provider.get::<_, String>("test").await.unwrap(), None);
        assert!(provider.set("test", "value", CacheEntryConfig::default()).await.is_ok());
        assert_eq!(provider.exists("test").await.unwrap(), false);
        assert_eq!(provider.delete("test").await.unwrap(), false);
        assert!(provider.clear().await.is_ok());
    }

    #[tokio::test]
    async fn test_shared_cache_provider_namespacing() {
        let provider = SharedCacheProvider::new(NullCacheProvider::new());

        // Test namespaced operations
        let namespaced = provider.namespaced("test_ns");

        assert_eq!(namespaced.get::<_, String>("key").await.unwrap(), None);
        assert!(namespaced.set("key", "value", CacheEntryConfig::default()).await.is_ok());
        assert_eq!(namespaced.exists("key").await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_cache_entry_config() {
        let config = CacheEntryConfig::new()
            .with_ttl(Duration::from_secs(300))
            .with_namespace("test");

        assert_eq!(config.effective_ttl(), Duration::from_secs(300));
        assert_eq!(config.effective_namespace(), "test");
    }
}