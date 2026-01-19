//! Cache Provider Port
//!
//! Port for cache backend providers. Supports multiple backends including
//! in-memory (Moka), distributed (Redis), and null providers for testing.
//!
//! ## Provider Pattern
//!
//! This port follows the same pattern as [`EmbeddingProvider`] and
//! [`VectorStoreProvider`], enabling consistent provider registration
//! and factory-based creation.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default TTL for cache entries (5 minutes)
pub const DEFAULT_CACHE_TTL_SECS: u64 = 300;

/// Cache Entry Configuration
///
/// Configures how a cache entry should be stored, including TTL
/// and optional namespace isolation.
///
/// # Example
///
/// ```ignore
/// use mcb_application::ports::providers::cache::CacheEntryConfig;
/// use std::time::Duration;
///
/// let config = CacheEntryConfig::default()
///     .with_ttl(Duration::from_secs(600))
///     .with_namespace("embeddings");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntryConfig {
    /// Time to live for the cache entry
    pub ttl: Option<Duration>,
    /// Namespace for the cache entry
    pub namespace: Option<String>,
}

impl CacheEntryConfig {
    /// Create a new cache entry config with default TTL
    pub fn new() -> Self {
        Self {
            ttl: Some(Duration::from_secs(DEFAULT_CACHE_TTL_SECS)),
            namespace: None,
        }
    }

    /// Set the TTL for the cache entry
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Set TTL in seconds
    pub fn with_ttl_secs(mut self, secs: u64) -> Self {
        self.ttl = Some(Duration::from_secs(secs));
        self
    }

    /// Set the namespace for the cache entry
    pub fn with_namespace<S: Into<String>>(mut self, namespace: S) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Get the effective TTL, falling back to default
    pub fn effective_ttl(&self) -> Duration {
        self.ttl
            .unwrap_or(Duration::from_secs(DEFAULT_CACHE_TTL_SECS))
    }

    /// Get the effective namespace, falling back to default
    pub fn effective_namespace(&self) -> String {
        self.namespace
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }
}

impl Default for CacheEntryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache Operation Statistics
///
/// Tracks cache performance metrics including hits, misses, and hit rate.
///
/// # Example
///
/// ```ignore
/// use mcb_application::ports::providers::cache::CacheStats;
///
/// let stats = CacheStats::default();
/// println!("Hit rate: {:.1}%", stats.hit_rate * 100.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache entries
    pub entries: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Total bytes used by cache
    pub bytes_used: u64,
}

impl CacheStats {
    /// Create empty cache statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate hit rate from hits and misses
    pub fn calculate_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Cache Provider Port
///
/// Defines the contract for cache backend providers. Implementations
/// must provide JSON-based storage with TTL support.
///
/// # Implementations
///
/// - **Moka**: In-memory cache with configurable TTL
/// - **Redis**: Distributed cache for multi-instance deployments
/// - **Null**: No-op provider for testing
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::CacheProvider;
///
/// // Store JSON in cache with TTL
/// let config = CacheEntryConfig::default().with_ttl_secs(300);
/// cache.set_json("user:123", &user_json, config).await?;
///
/// // Retrieve from cache
/// if let Some(json) = cache.get_json("user:123").await? {
///     let user: User = serde_json::from_str(&json)?;
/// }
/// ```
#[async_trait]
pub trait CacheProvider: Send + Sync + std::fmt::Debug {
    /// Get a value from the cache as JSON string
    ///
    /// # Arguments
    /// * `key` - The cache key
    ///
    /// # Returns
    /// The cached JSON string if present, None if not found or expired
    async fn get_json(&self, key: &str) -> Result<Option<String>>;

    /// Set a value in the cache from JSON string
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The JSON string to cache
    /// * `config` - Entry configuration (TTL, namespace)
    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()>;

    /// Delete a value from the cache
    ///
    /// # Arguments
    /// * `key` - The cache key to delete
    ///
    /// # Returns
    /// True if the key was deleted, false if it didn't exist
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Check if a key exists in the cache
    ///
    /// # Arguments
    /// * `key` - The cache key to check
    ///
    /// # Returns
    /// True if the key exists and hasn't expired
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Clear all values from the cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;

    /// Get the cache size (number of entries)
    async fn size(&self) -> Result<usize>;

    /// Get the name/identifier of this provider implementation
    ///
    /// # Returns
    /// A string identifier for the provider (e.g., "moka", "redis", "null")
    fn provider_name(&self) -> &str;
}

/// Cache provider factory interface for dependency injection
///
/// This port defines the contract for creating cache providers from configuration.
/// Used by the dependency injection container to create cache provider instances.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::CacheProviderFactoryInterface;
///
/// // Create cache provider from configuration
/// let cache = factory.create_from_config(&cache_config).await?;
/// cache.set("key", &my_value, None).await?;
///
/// // Create null cache for testing
/// let test_cache = factory.create_null();
/// ```
#[async_trait]
pub trait CacheProviderFactoryInterface: Send + Sync {
    /// Create a cache provider from configuration
    async fn create_from_config(
        &self,
        config: &crate::value_objects::config::CacheConfig,
    ) -> Result<std::sync::Arc<dyn CacheProvider>>;

    /// Create a null cache provider for testing
    fn create_null(&self) -> std::sync::Arc<dyn CacheProvider>;
}
