//! Cache configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Cache providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheProvider {
    /// In-memory cache (Moka)
    Moka,
    /// Distributed cache (Redis)
    Redis,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
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

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: CacheProvider::Moka,
            default_ttl_secs: CACHE_DEFAULT_TTL_SECS,
            max_size: CACHE_DEFAULT_SIZE_LIMIT,
            redis_url: None,
            redis_pool_size: REDIS_POOL_SIZE as u32,
            namespace: "mcb".to_string(),
        }
    }
}
