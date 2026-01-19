//! Redis distributed cache provider
//!
//! Distributed cache implementation using Redis as the backend.
//! Suitable for multi-instance deployments.
//!
//! ## Features
//!
//! - Distributed caching for multiple instances
//! - TTL support for automatic expiration
//! - Connection pooling via multiplexed connection
//!
//! ## Example
//!
//! ```ignore
//! use mcb_providers::cache::RedisCacheProvider;
//!
//! let provider = RedisCacheProvider::new("redis://localhost:6379")?;
//! // Or with host/port
//! let provider = RedisCacheProvider::with_host_port("localhost", 6379)?;
//! ```

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::cache::{CacheEntryConfig, CacheProvider, CacheStats};
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};
use std::sync::{Arc, RwLock};

/// Redis cache provider
///
/// Distributed cache implementation using Redis.
/// Uses multiplexed connections for efficient connection reuse.
#[derive(Clone)]
pub struct RedisCacheProvider {
    client: Client,
    stats: Arc<RwLock<CacheStats>>,
}

impl RedisCacheProvider {
    /// Create a new Redis cache provider with connection string
    ///
    /// # Arguments
    ///
    /// * `connection_string` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let provider = RedisCacheProvider::new("redis://localhost:6379")?;
    /// ```
    pub fn new(connection_string: &str) -> Result<Self> {
        let client = Client::open(connection_string).map_err(|e| Error::Infrastructure {
            message: format!("Failed to create Redis client: {}", e),
            source: Some(Box::new(e)),
        })?;

        Ok(Self {
            client,
            stats: Arc::new(RwLock::new(CacheStats::new())),
        })
    }

    /// Create a new Redis cache provider with host and port
    ///
    /// # Arguments
    ///
    /// * `host` - Redis server hostname
    /// * `port` - Redis server port
    pub fn with_host_port(host: &str, port: u16) -> Result<Self> {
        Self::new(&format!("redis://{}:{}", host, port))
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to get Redis connection: {}", e),
                source: Some(Box::new(e)),
            })
    }

    /// Record a cache hit
    fn record_hit(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.hits += 1;
            let total = stats.hits + stats.misses;
            stats.hit_rate = if total > 0 {
                stats.hits as f64 / total as f64
            } else {
                0.0
            };
        }
    }

    /// Record a cache miss
    fn record_miss(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.misses += 1;
            let total = stats.hits + stats.misses;
            stats.hit_rate = if total > 0 {
                stats.hits as f64 / total as f64
            } else {
                0.0
            };
        }
    }

    /// Get the Redis server address description
    pub fn server_address(&self) -> String {
        "redis-server".to_string()
    }

    /// Check if the Redis connection uses TLS
    pub fn is_tls(&self) -> bool {
        false
    }
}

#[async_trait]
impl CacheProvider for RedisCacheProvider {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(value)) => {
                self.record_hit();
                Ok(Some(value))
            }
            Ok(None) => {
                self.record_miss();
                Ok(None)
            }
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis GET failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let ttl_seconds = config.effective_ttl().as_secs();

        let result: redis::RedisResult<()> = if ttl_seconds > 0 {
            conn.set_ex(key, value, ttl_seconds).await
        } else {
            conn.set(key, value).await
        };

        result.map_err(|e| Error::Infrastructure {
            message: format!("Redis SET failed: {}", e),
            source: Some(Box::new(e)),
        })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let deleted: redis::RedisResult<i32> = conn.del(key).await;
        match deleted {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis DEL failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let exists: redis::RedisResult<i32> = conn.exists(key).await;
        match exists {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis EXISTS failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn clear(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Redis FLUSHDB failed: {}", e),
                source: Some(Box::new(e)),
            })
    }

    async fn stats(&self) -> Result<CacheStats> {
        let mut conn = self.get_connection().await?;

        // Get basic Redis stats using DBSIZE command
        let dbsize: redis::RedisResult<usize> = redis::cmd("DBSIZE").query_async(&mut conn).await;
        let dbsize = dbsize.unwrap_or(0);

        // Get our internal stats
        let mut internal_stats = self
            .stats
            .read()
            .map_err(|_| Error::Infrastructure {
                message: "Failed to read cache stats".to_string(),
                source: None,
            })?
            .clone();

        internal_stats.entries = dbsize as u64;

        Ok(internal_stats)
    }

    async fn size(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;

        let dbsize: redis::RedisResult<usize> = redis::cmd("DBSIZE").query_async(&mut conn).await;
        dbsize.map_err(|e| Error::Infrastructure {
            message: format!("Redis DBSIZE failed: {}", e),
            source: Some(Box::new(e)),
        })
    }

    fn provider_name(&self) -> &str {
        "redis"
    }
}

impl std::fmt::Debug for RedisCacheProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCacheProvider")
            .field("server", &self.server_address())
            .field("tls", &self.is_tls())
            .finish()
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_application::ports::registry::{CACHE_PROVIDERS, CacheProviderConfig, CacheProviderEntry};

/// Factory function for creating Redis cache provider instances.
fn redis_cache_factory(
    config: &CacheProviderConfig,
) -> std::result::Result<Arc<dyn CacheProvider>, String> {
    let uri = config
        .uri
        .clone()
        .unwrap_or_else(|| "redis://localhost:6379".to_string());

    let provider = RedisCacheProvider::new(&uri)
        .map_err(|e| format!("Failed to create Redis provider: {e}"))?;

    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(CACHE_PROVIDERS)]
static REDIS_PROVIDER: CacheProviderEntry = CacheProviderEntry {
    name: "redis",
    description: "Redis distributed cache",
    factory: redis_cache_factory,
};
