//! Rate Limiting System
//!
//! Implements sliding window rate limiting with Redis backend for distributed
//! enforcement across multiple instances. Supports both IP-based and user-based
//! rate limiting for production security.

use crate::core::error::{Error, Result};
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Type alias for sliding window data structure
type SlidingWindowData = HashMap<String, VecDeque<(u64, u32)>>;

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Backend type: "memory" for single-node, "redis" for clustered
    pub backend: RateLimitBackend,
    /// Window duration in seconds
    pub window_seconds: u64,
    /// Maximum requests per window
    pub max_requests_per_window: u32,
    /// Burst allowance (additional requests beyond max)
    pub burst_allowance: u32,
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

/// Rate limiting backend types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RateLimitBackend {
    /// In-memory backend for single-node deployments
    #[serde(rename = "memory")]
    Memory {
        /// Maximum entries in memory cache (default: 10000)
        #[serde(default = "default_memory_max_entries")]
        max_entries: usize,
    },
    /// Redis backend for clustered deployments
    #[serde(rename = "redis")]
    Redis {
        /// Redis connection URL
        url: String,
    },
}

fn default_memory_max_entries() -> usize {
    10000
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            backend: RateLimitBackend::Memory {
                max_entries: default_memory_max_entries(),
            },
            window_seconds: 60,
            max_requests_per_window: 100,
            burst_allowance: 20,
            enabled: true,
        }
    }
}

/// Rate limit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining: u32,
    /// Seconds until window resets
    pub reset_in_seconds: u64,
    /// Current request count in window
    pub current_count: u32,
    /// Total limit (max_requests_per_window + burst_allowance)
    pub limit: u32,
}

/// Rate limit key types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateLimitKey {
    /// IP address based limiting
    Ip(String),
    /// User ID based limiting
    User(String),
    /// API key based limiting
    ApiKey(String),
    /// Endpoint specific limiting
    Endpoint(String),
}

impl std::fmt::Display for RateLimitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitKey::Ip(ip) => write!(f, "ip:{}", ip),
            RateLimitKey::User(user) => write!(f, "user:{}", user),
            RateLimitKey::ApiKey(key) => write!(f, "apikey:{}", key),
            RateLimitKey::Endpoint(endpoint) => write!(f, "endpoint:{}", endpoint),
        }
    }
}

/// Storage backends for rate limiting
#[derive(Clone)]
enum RateLimitStorage {
    /// In-memory storage for single-node deployments
    Memory {
        /// Sliding window data: key -> (timestamps, counts)
        windows: Arc<RwLock<SlidingWindowData>>,
        /// Maximum entries to prevent memory leaks
        max_entries: usize,
    },
    /// Redis storage for clustered deployments
    Redis { client: Arc<RwLock<Option<Client>>> },
}

/// Rate limiter with pluggable storage backends
#[derive(Clone)]
pub struct RateLimiter {
    storage: RateLimitStorage,
    config: RateLimitConfig,
    /// In-memory cache for faster lookups (works with both backends)
    memory_cache: Arc<RwLock<HashMap<String, (Instant, RateLimitResult)>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let storage = match &config.backend {
            RateLimitBackend::Memory { max_entries } => RateLimitStorage::Memory {
                windows: Arc::new(RwLock::new(HashMap::new())),
                max_entries: *max_entries,
            },
            RateLimitBackend::Redis { .. } => RateLimitStorage::Redis {
                client: Arc::new(RwLock::new(None)),
            },
        };

        Self {
            storage,
            config,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the storage backend
    pub async fn init(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        match &self.storage {
            RateLimitStorage::Memory { .. } => {
                // Memory backend doesn't need initialization
                Ok(())
            }
            RateLimitStorage::Redis { client, .. } => {
                let redis_config = match &self.config.backend {
                    RateLimitBackend::Redis { url, .. } => url,
                    _ => unreachable!(),
                };

                let redis_client = Client::open(redis_config.as_str()).map_err(|e| {
                    Error::internal(format!("Failed to create Redis client: {}", e))
                })?;

                // Test connection
                let mut conn = redis_client
                    .get_connection()
                    .map_err(|e| Error::internal(format!("Failed to connect to Redis: {}", e)))?;

                let _: String = redis::cmd("PING")
                    .query(&mut conn)
                    .map_err(|e| Error::internal(format!("Redis PING failed: {}", e)))?;

                *client.write().await = Some(redis_client);
                Ok(())
            }
        }
    }

    /// Check if request is allowed for given key
    pub async fn check_rate_limit(&self, key: &RateLimitKey) -> Result<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult {
                allowed: true,
                remaining: u32::MAX,
                reset_in_seconds: 0,
                current_count: 0,
                limit: self.config.max_requests_per_window + self.config.burst_allowance,
            });
        }

        // Check memory cache first for performance
        let cache_key = key.to_string();
        if let Some((cached_at, result)) = self.memory_cache.read().await.get(&cache_key) {
            if cached_at.elapsed() < Duration::from_secs(1) {
                return Ok(result.clone());
            }
        }

        let result = self.check_storage_rate_limit(key).await?;

        // Cache result for 1 second
        self.memory_cache
            .write()
            .await
            .insert(cache_key, (Instant::now(), result.clone()));

        Ok(result)
    }

    /// Check rate limit against storage backend
    async fn check_storage_rate_limit(&self, key: &RateLimitKey) -> Result<RateLimitResult> {
        match &self.storage {
            RateLimitStorage::Memory {
                windows,
                max_entries,
            } => {
                self.check_memory_rate_limit(key, windows, *max_entries)
                    .await
            }
            RateLimitStorage::Redis { client, .. } => {
                self.check_redis_rate_limit(key, client).await
            }
        }
    }

    /// Check rate limit against in-memory storage
    async fn check_memory_rate_limit(
        &self,
        key: &RateLimitKey,
        windows: &Arc<RwLock<SlidingWindowData>>,
        max_entries: usize,
    ) -> Result<RateLimitResult> {
        let storage_key = key.to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let window_start = now.saturating_sub(self.config.window_seconds);

        let mut windows_guard = windows.write().await;

        // Clean up old entries to prevent memory leaks
        if windows_guard.len() > max_entries {
            // Remove oldest entries when exceeding max_entries
            let keys_to_remove: Vec<_> = windows_guard.keys().cloned().collect();
            for key in keys_to_remove.into_iter().skip(max_entries / 2) {
                windows_guard.remove(&key);
            }
        }

        // Get or create window for this key
        let window = windows_guard
            .entry(storage_key.clone())
            .or_insert_with(VecDeque::new);

        // Remove old entries outside the window
        while let Some(&(timestamp, _)) = window.front() {
            if timestamp < window_start {
                window.pop_front();
            } else {
                break;
            }
        }

        // Count current requests in window
        let current_count = window.iter().map(|(_, count)| count).sum::<u32>();

        let max_allowed = self.config.max_requests_per_window + self.config.burst_allowance;
        let would_be_count = current_count + 1;
        let allowed = would_be_count <= max_allowed;
        let remaining = max_allowed.saturating_sub(would_be_count);

        // If allowed, add current request to window
        if allowed {
            window.push_back((now, 1));
        }

        // Calculate reset time (simplified for memory backend)
        let reset_in_seconds = self.config.window_seconds;

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_in_seconds,
            current_count,
            limit: self.config.max_requests_per_window + self.config.burst_allowance,
        })
    }

    /// Check rate limit against Redis with sliding window algorithm
    async fn check_redis_rate_limit(
        &self,
        key: &RateLimitKey,
        client: &Arc<RwLock<Option<Client>>>,
    ) -> Result<RateLimitResult> {
        let client_guard = client.read().await;
        let redis_client = client_guard
            .as_ref()
            .ok_or_else(|| Error::internal("Redis client not initialized"))?;

        let mut conn = redis_client
            .get_connection()
            .map_err(|e| Error::internal(format!("Redis connection failed: {}", e)))?;

        let redis_key = format!("ratelimit:{}", key);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let window_start = now - self.config.window_seconds as f64;

        // Remove old entries outside the window
        // Note: zremrangebyscore removes by score range, we want to keep recent entries
        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&redis_key)
            .arg("-inf")
            .arg(window_start)
            .query(&mut conn)
            .map_err(|e| Error::internal(format!("Redis ZREMRANGEBYSCORE failed: {}", e)))?;

        // Count current requests in window
        let current_count: u32 = redis::cmd("ZCARD")
            .arg(&redis_key)
            .query(&mut conn)
            .map_err(|e| Error::internal(format!("Redis ZCARD failed: {}", e)))?;

        let max_allowed = self.config.max_requests_per_window + self.config.burst_allowance;
        let allowed = current_count < max_allowed;
        let remaining = max_allowed.saturating_sub(current_count);

        // If allowed, add current request to window
        if allowed {
            let _: () = redis::cmd("ZADD")
                .arg(&redis_key)
                .arg(now)
                .arg(now)
                .query(&mut conn)
                .map_err(|e| Error::internal(format!("Redis ZADD failed: {}", e)))?;

            // Set expiration on the key (window + buffer)
            let _: () = redis::cmd("EXPIRE")
                .arg(&redis_key)
                .arg(self.config.window_seconds as i64 * 2)
                .query(&mut conn)
                .map_err(|e| Error::internal(format!("Redis EXPIRE failed: {}", e)))?;
        }

        // Calculate reset time (end of current window)
        let window_end = window_start + self.config.window_seconds as f64;
        let reset_in_seconds = if now < window_end {
            (window_end - now) as u64
        } else {
            self.config.window_seconds
        };

        Ok(RateLimitResult {
            allowed,
            remaining,
            reset_in_seconds,
            current_count,
            limit: self.config.max_requests_per_window + self.config.burst_allowance,
        })
    }

    /// Get current rate limit status for key
    pub async fn get_status(&self, key: &RateLimitKey) -> Result<RateLimitResult> {
        self.check_rate_limit(key).await
    }

    /// Get the backend type
    pub fn backend_type(&self) -> &str {
        match &self.config.backend {
            RateLimitBackend::Memory { .. } => "memory",
            RateLimitBackend::Redis { .. } => "redis",
        }
    }

    /// Reset rate limit for a key (admin function)
    pub async fn reset_key(&self, key: &RateLimitKey) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        match &self.storage {
            RateLimitStorage::Memory { windows, .. } => {
                let storage_key = key.to_string();
                windows.write().await.remove(&storage_key);
            }
            RateLimitStorage::Redis { client, .. } => {
                let client_guard = client.read().await;
                let redis_client = client_guard
                    .as_ref()
                    .ok_or_else(|| Error::internal("Redis client not initialized"))?;

                let mut conn = redis_client
                    .get_connection()
                    .map_err(|e| Error::internal(format!("Redis connection failed: {}", e)))?;

                let redis_key = format!("ratelimit:{}", key);
                let _: () = redis::cmd("DEL")
                    .arg(&redis_key)
                    .query(&mut conn)
                    .map_err(|e| Error::internal(format!("Redis DEL failed: {}", e)))?;
            }
        }

        // Clear from memory cache
        self.memory_cache.write().await.remove(&key.to_string());

        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_disabled() {
        let config = RateLimitConfig {
            enabled: false,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        let key = RateLimitKey::Ip("127.0.0.1".to_string());
        let result = limiter.check_rate_limit(&key).await.unwrap();

        assert!(result.allowed);
        assert_eq!(result.remaining, u32::MAX);
        assert_eq!(result.reset_in_seconds, 0);
    }

    #[tokio::test]
    async fn test_memory_backend_basic() {
        let config = RateLimitConfig {
            backend: RateLimitBackend::Memory { max_entries: 1000 },
            window_seconds: 60,
            max_requests_per_window: 10,
            burst_allowance: 2,
            enabled: true,
        };
        let limiter = RateLimiter::new(config);
        limiter.init().await.unwrap();

        // Clear cache to ensure fresh results for this test
        limiter.memory_cache.write().await.clear();

        let key = RateLimitKey::Ip("127.0.0.1".to_string());

        // First 12 requests should be allowed (10 + 2 burst)
        for i in 0..12 {
            let result = limiter.check_storage_rate_limit(&key).await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 13th request should be blocked
        let result = limiter.check_storage_rate_limit(&key).await.unwrap();
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[tokio::test]
    async fn test_backend_types() {
        let memory_config = RateLimitConfig {
            backend: RateLimitBackend::Memory { max_entries: 1000 },
            ..Default::default()
        };
        let memory_limiter = RateLimiter::new(memory_config);
        assert_eq!(memory_limiter.backend_type(), "memory");

        let redis_config = RateLimitConfig {
            backend: RateLimitBackend::Redis {
                url: "redis://localhost:6379".to_string(),
            },
            ..Default::default()
        };
        let redis_limiter = RateLimiter::new(redis_config);
        assert_eq!(redis_limiter.backend_type(), "redis");
    }

    #[tokio::test]
    async fn test_rate_limit_keys() {
        let ip_key = RateLimitKey::Ip("192.168.1.1".to_string());
        let user_key = RateLimitKey::User("user123".to_string());
        let api_key = RateLimitKey::ApiKey("key456".to_string());
        let endpoint_key = RateLimitKey::Endpoint("/api/search".to_string());

        assert_eq!(ip_key.to_string(), "ip:192.168.1.1");
        assert_eq!(user_key.to_string(), "user:user123");
        assert_eq!(api_key.to_string(), "apikey:key456");
        assert_eq!(endpoint_key.to_string(), "endpoint:/api/search");
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        match config.backend {
            RateLimitBackend::Memory { .. } => {} // Default is memory
            _ => panic!("Expected memory backend"),
        }
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.max_requests_per_window, 100);
        assert_eq!(config.burst_allowance, 20);
        assert!(config.enabled);
    }
}
