//! Cache system configuration types
//!
//! This module defines infrastructure-level cache configuration.
//!
//! **Note:** This `CacheSystemConfig` is distinct from
//! `mcb_domain::value_objects::CacheConfig` which configures
//! provider-specific cache settings.

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Cache providers for infrastructure caching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheProvider {
    /// In-memory cache (Moka)
    Moka,
    /// Distributed cache (Redis)
    Redis,
}

impl CacheProvider {
    /// Get the provider name as a string for registry lookup
    ///
    /// Used by DI resolver to query the provider registry without
    /// coupling to concrete enum variants.
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheProvider::Moka => "moka",
            CacheProvider::Redis => "redis",
        }
    }
}

/// Infrastructure cache system configuration
///
/// Configures the caching layer used by the infrastructure.
/// This is distinct from `mcb_domain::value_objects::CacheConfig`
/// which configures provider-specific cache behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSystemConfig {
    /// Cache enabled
    pub enabled: bool,

    /// Cache provider
    pub provider: CacheProvider,

    /// Default TTL in seconds
    pub default_ttl_secs: u64,

    /// Maximum cache size in bytes
    pub max_size: usize,

    /// Redis URL (for Redis provider)
    pub redis_url: Option<String>,

    /// Redis connection pool size
    pub redis_pool_size: u32,

    /// Namespace for cache keys
    pub namespace: String,
}

/// Returns default cache configuration with:
/// - Cache enabled with Moka in-memory provider
/// - TTL and size limits from infrastructure constants
/// - Default namespace: "mcb" for cache key isolation
impl Default for CacheSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: CacheProvider::Moka,
            default_ttl_secs: CACHE_DEFAULT_TTL_SECS,
            max_size: CACHE_DEFAULT_SIZE_LIMIT,
            redis_url: None,
            redis_pool_size: REDIS_POOL_SIZE as u32,
            namespace: DEFAULT_CACHE_NAMESPACE.to_string(),
        }
    }
}
