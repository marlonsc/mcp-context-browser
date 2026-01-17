//! Shared cache provider wrapper
//!
//! Infrastructure wrapper for cache providers from mcb-providers.
//! Types come from mcb-domain (SOURCE OF TRUTH).
//!
//! **ARCHITECTURE**: Uses `Arc<dyn CacheProvider>` directly to follow OCP.
//! No enum wrapper - new cache providers can be added without modification.

use mcb_application::ports::providers::cache::{CacheEntryConfig, CacheProvider, CacheStats};
use mcb_domain::error::Result;
use std::fmt;
use std::sync::Arc;

/// Shared cache provider wrapper
///
/// This provides thread-safe access to a cache provider and includes
/// additional features like metrics collection and error handling.
///
/// **Architecture Note**: Uses `Arc<dyn CacheProvider>` directly
/// to follow OCP - new providers can be added without modification.
#[derive(Clone)]
pub struct SharedCacheProvider {
    provider: Arc<dyn CacheProvider>,
    namespace: Option<String>,
}

// Construction and Configuration Methods
impl SharedCacheProvider {
    /// Create a new shared cache provider
    ///
    /// Accepts any type implementing CacheProvider trait.
    pub fn new<P: CacheProvider + 'static>(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            namespace: None,
        }
    }

    /// Create a new shared cache provider from Arc
    pub fn from_arc(provider: Arc<dyn CacheProvider>) -> Self {
        Self {
            provider,
            namespace: None,
        }
    }

    /// Create a new shared cache provider with a default namespace
    pub fn with_namespace<P: CacheProvider + 'static, S: Into<String>>(
        provider: P,
        namespace: S,
    ) -> Self {
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

    /// Get the underlying cache provider as an Arc
    pub fn as_provider(&self) -> Arc<dyn CacheProvider> {
        self.provider.clone()
    }

    /// Get a namespaced key
    fn namespaced_key<K: AsRef<str>>(&self, key: K) -> String {
        if let Some(ns) = &self.namespace {
            format!("{}:{}", ns, key.as_ref())
        } else {
            key.as_ref().to_string()
        }
    }
}

// Cache Operations Methods
impl SharedCacheProvider {
    /// Get a typed value from the cache
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned + Send,
    {
        let namespaced_key = self.namespaced_key(key);
        match self.provider.get_json(&namespaced_key).await? {
            Some(json) => {
                let value: T = serde_json::from_str(&json).map_err(|e| {
                    mcb_domain::error::Error::Infrastructure {
                        message: format!("Failed to deserialize cached value: {}", e),
                        source: Some(Box::new(e)),
                    }
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
        let json =
            serde_json::to_string(value).map_err(|e| mcb_domain::error::Error::Infrastructure {
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
                let value: T = serde_json::from_str(&json).map_err(|e| {
                    mcb_domain::error::Error::Infrastructure {
                        message: format!("Failed to deserialize cached value: {}", e),
                        source: Some(Box::new(e)),
                    }
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
        let json =
            serde_json::to_string(value).map_err(|e| mcb_domain::error::Error::Infrastructure {
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

    /// Get the inner cache provider for DI injection
    pub fn inner(&self) -> Arc<dyn CacheProvider> {
        self.provider.clone()
    }
}

impl From<SharedCacheProvider> for Arc<dyn CacheProvider> {
    fn from(shared: SharedCacheProvider) -> Self {
        shared.provider
    }
}
