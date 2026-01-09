//! Rate Limiting System
//!
//! Implements sliding window rate limiting with Redis backend for distributed
//! enforcement across multiple instances. Supports both IP-based and user-based
//! rate limiting for production security.

use crate::core::error::{Error, Result};
use arc_swap::ArcSwapOption;
use dashmap::DashMap;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::task::spawn_blocking;
use tokio::time::timeout;
use validator::Validate;

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
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
    /// Connection timeout in seconds for Redis operations
    #[serde(default = "default_redis_timeout")]
    pub redis_timeout_seconds: u64,
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

impl Validate for RateLimitBackend {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();
        match self {
            RateLimitBackend::Redis { url } => {
                if url.is_empty() {
                    errors.add("url", validator::ValidationError::new("length"));
                }
            }
            RateLimitBackend::Memory { max_entries } => {
                if *max_entries == 0 {
                    errors.add("max_entries", validator::ValidationError::new("range"));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn default_memory_max_entries() -> usize {
    10000
}

fn default_redis_timeout() -> u64 {
    5 // 5 seconds default timeout
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
            redis_timeout_seconds: default_redis_timeout(),
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

/// Sliding window data structure
type SlidingWindowData = DashMap<String, VecDeque<(u64, u32)>>;

/// Storage backends for rate limiting
#[derive(Clone)]
enum RateLimitStorage {
    /// In-memory storage for single-node deployments
    Memory {
        /// Sliding window data: key -> (timestamps, counts)
        windows: Arc<SlidingWindowData>,
        /// Maximum entries to prevent memory leaks
        max_entries: usize,
    },
    /// Redis storage for clustered deployments
    Redis { client: Arc<ArcSwapOption<Client>> },
}

/// Rate limiter with pluggable storage backends
#[derive(Clone)]
pub struct RateLimiter {
    storage: RateLimitStorage,
    config: RateLimitConfig,
    /// In-memory cache for faster lookups (works with both backends)
    memory_cache: Arc<DashMap<String, (Instant, RateLimitResult)>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        let storage = match &config.backend {
            RateLimitBackend::Memory { max_entries } => RateLimitStorage::Memory {
                windows: Arc::new(DashMap::new()),
                max_entries: *max_entries,
            },
            RateLimitBackend::Redis { .. } => RateLimitStorage::Redis {
                client: Arc::new(ArcSwapOption::new(None)),
            },
        };

        Self {
            storage,
            config,
            memory_cache: Arc::new(DashMap::new()),
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

                // Test connection with timeout using blocking task
                let timeout_duration = Duration::from_secs(self.config.redis_timeout_seconds);
                let client_clone = redis_client.clone();

                let conn_result = timeout(
                    timeout_duration,
                    spawn_blocking(move || client_clone.get_connection()),
                )
                .await;

                let mut conn = match conn_result {
                    Ok(Ok(Ok(conn))) => conn,
                    Ok(Ok(Err(e))) => {
                        return Err(Error::internal(format!(
                            "Failed to connect to Redis: {}",
                            e
                        )));
                    }
                    Ok(Err(join_err)) => {
                        return Err(Error::internal(format!(
                            "Connection task panicked: {}",
                            join_err
                        )));
                    }
                    Err(_) => {
                        return Err(Error::internal(format!(
                            "Redis connection timeout after {} seconds",
                            self.config.redis_timeout_seconds
                        )));
                    }
                };

                let ping_result = timeout(
                    timeout_duration,
                    spawn_blocking(move || redis::cmd("PING").query::<String>(&mut conn)),
                )
                .await;

                match ping_result {
                    Ok(Ok(Ok(_))) => {} // PING successful
                    Ok(Ok(Err(e))) => {
                        return Err(Error::internal(format!("Redis PING failed: {}", e)));
                    }
                    Ok(Err(join_err)) => {
                        return Err(Error::internal(format!("PING task panicked: {}", join_err)));
                    }
                    Err(_) => {
                        return Err(Error::internal(format!(
                            "Redis PING timeout after {} seconds",
                            self.config.redis_timeout_seconds
                        )));
                    }
                }

                client.store(Some(Arc::new(redis_client)));
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
        if let Some(entry) = self.memory_cache.get(&cache_key) {
            let (cached_at, result) = entry.value();
            if cached_at.elapsed() < Duration::from_secs(1) {
                return Ok(result.clone());
            }
        }

        let result = self.check_storage_rate_limit(key).await?;

        // Cache result for 1 second
        self.memory_cache
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
        windows: &Arc<SlidingWindowData>,
        max_entries: usize,
    ) -> Result<RateLimitResult> {
        let storage_key = key.to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let window_start = now.saturating_sub(self.config.window_seconds);

        // Clean up old entries to prevent memory leaks if we exceeded max_entries
        if windows.len() > max_entries {
            // Very simple cleanup: remove about half
            let keys_to_remove: Vec<_> = windows
                .iter()
                .take(max_entries / 2)
                .map(|r| r.key().clone())
                .collect();
            for key in keys_to_remove {
                windows.remove(&key);
            }
        }

        // Get or create window for this key
        let mut window_entry = windows.entry(storage_key.clone()).or_default();
        let window = window_entry.value_mut();

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
        client: &Arc<ArcSwapOption<Client>>,
    ) -> Result<RateLimitResult> {
        let redis_client = client.load();
        let redis_client = redis_client
            .as_ref()
            .ok_or_else(|| Error::internal("Redis client not initialized"))?;

        // Get connection with timeout using blocking task
        let timeout_duration = Duration::from_secs(self.config.redis_timeout_seconds);
        let client_clone = redis_client.clone();

        let conn_result = timeout(
            timeout_duration,
            spawn_blocking(move || client_clone.get_connection()),
        )
        .await;

        let mut conn = match conn_result {
            Ok(Ok(Ok(conn))) => conn,
            Ok(Ok(Err(e))) => {
                return Err(Error::internal(format!("Redis connection failed: {}", e)));
            }
            Ok(Err(join_err)) => {
                return Err(Error::internal(format!(
                    "Connection task panicked: {}",
                    join_err
                )));
            }
            Err(_) => {
                return Err(Error::internal(format!(
                    "Redis connection timeout after {} seconds",
                    self.config.redis_timeout_seconds
                )));
            }
        };

        let redis_key = format!("ratelimit:{}", key);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64;

        let window_start = now - self.config.window_seconds as f64;

        // Remove old entries outside the window
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
                windows.remove(&storage_key);
            }
            RateLimitStorage::Redis { client, .. } => {
                let redis_client = client.load();
                let redis_client = redis_client
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
        self.memory_cache.remove(&key.to_string());

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
            redis_timeout_seconds: default_redis_timeout(),
        };
        let limiter = RateLimiter::new(config);
        limiter.init().await.unwrap();

        // Clear cache to ensure fresh results for this test
        limiter.memory_cache.clear();

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
            redis_timeout_seconds: 5,
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
        assert_eq!(config.redis_timeout_seconds, 5);
    }
}
