//! Moka in-memory cache provider
//!
//! High-performance, concurrent in-memory cache implementation using Moka.
//!
//! ## Features
//!
//! - High-performance concurrent cache
//! - Configurable capacity and TTL
//! - Automatic eviction of expired entries
//!
//! ## Example
//!
//! ```ignore
//! use mcb_providers::cache::MokaCacheProvider;
//! use std::time::Duration;
//!
//! let provider = MokaCacheProvider::with_config(1000, Duration::from_secs(300));
//! ```

use crate::constants::CACHE_DEFAULT_SIZE_LIMIT;
use async_trait::async_trait;
use mcb_application::ports::providers::cache::{CacheEntryConfig, CacheProvider, CacheStats};
use mcb_domain::error::{Error, Result};
use moka::future::Cache;
use std::time::Duration;

/// Moka-based in-memory cache provider
///
/// Uses the Moka crate for high-performance concurrent caching.
/// Supports configurable capacity and TTL.
///
/// **Note**: This type is created at runtime via factory pattern, not through Shaku DI.
/// For DI testing, use `NullCacheProvider`.
#[derive(Clone)]
pub struct MokaCacheProvider {
    cache: Cache<String, Vec<u8>>,
    max_size: usize,
}

impl Default for MokaCacheProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MokaCacheProvider {
    /// Create a new Moka cache provider with default settings
    pub fn new() -> Self {
        Self::with_capacity(CACHE_DEFAULT_SIZE_LIMIT)
    }

    /// Create a new Moka cache provider with specified capacity
    pub fn with_capacity(max_size: usize) -> Self {
        let cache = Cache::builder().max_capacity(max_size as u64).build();

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

    /// Get the maximum capacity of the cache
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

#[async_trait]
impl CacheProvider for MokaCacheProvider {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        if let Some(bytes) = self.cache.get(key).await {
            let json = String::from_utf8(bytes).map_err(|e| Error::Infrastructure {
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
            return Err(Error::Infrastructure {
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
        // Run pending tasks to ensure entry_count is accurate
        self.cache.run_pending_tasks().await;
        let entries = self.cache.entry_count();

        Ok(CacheStats {
            hits: 0,   // Moka doesn't track hits/misses
            misses: 0, // Moka doesn't track hits/misses
            entries,
            hit_rate: 0.0, // Unknown
            bytes_used: 0, // Unknown
        })
    }

    async fn size(&self) -> Result<usize> {
        // Run pending tasks to ensure entry_count is accurate
        self.cache.run_pending_tasks().await;
        Ok(self.cache.entry_count() as usize)
    }

    fn provider_name(&self) -> &str {
        "moka"
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

// Shaku Component implementation for DI container
// This allows MokaCacheProvider to be used as a default in Shaku modules
impl<M: shaku::Module> shaku::Component<M> for MokaCacheProvider {
    type Interface = dyn CacheProvider;
    type Parameters = ();

    fn build(_: &mut shaku::ModuleBuildContext<M>, _: Self::Parameters) -> Box<Self::Interface> {
        Box::new(MokaCacheProvider::new())
    }
}

// ============================================================================
// Auto-registration via linkme
// ============================================================================

use mcb_application::ports::registry::{CacheProviderConfig, CacheProviderEntry, CACHE_PROVIDERS};

#[linkme::distributed_slice(CACHE_PROVIDERS)]
static MOKA_PROVIDER: CacheProviderEntry = CacheProviderEntry {
    name: "moka",
    description: "Moka high-performance in-memory cache",
    factory: |config: &CacheProviderConfig| {
        let provider = if let Some(max_size) = config.max_size {
            MokaCacheProvider::with_capacity(max_size)
        } else {
            MokaCacheProvider::new()
        };
        Ok(std::sync::Arc::new(provider))
    },
};
