//! Rate Limiting System
//!
//! Implements sliding window rate limiting with Redis backend for distributed
//! enforcement across multiple instances. Supports both IP-based and user-based
//! rate limiting for production security.

use crate::domain::error::{Error, Result};
use crate::infrastructure::constants::{
    RATE_LIMIT_BURST_ALLOWANCE, RATE_LIMIT_CACHE_MAX_ENTRIES, RATE_LIMIT_DEFAULT_MAX_REQUESTS,
    RATE_LIMIT_WINDOW_SECONDS,
};
use crate::infrastructure::utils::TimeUtils;
use arc_swap::ArcSwapOption;
use dashmap::DashMap;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    /// Cache TTL in seconds for rate limit results (default: 1)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

fn default_cache_ttl() -> u64 {
    1
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
    RATE_LIMIT_CACHE_MAX_ENTRIES
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
            window_seconds: RATE_LIMIT_WINDOW_SECONDS,
            max_requests_per_window: RATE_LIMIT_DEFAULT_MAX_REQUESTS,
            burst_allowance: RATE_LIMIT_BURST_ALLOWANCE,
            enabled: true,
            redis_timeout_seconds: default_redis_timeout(),
            cache_ttl_seconds: default_cache_ttl(),
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
                    _ => {
                        return Err(Error::internal(
                            "Redis storage configured but backend is not Redis",
                        ))
                    }
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
            if cached_at.elapsed() < Duration::from_secs(self.config.cache_ttl_seconds) {
                return Ok(result.clone());
            }
        }

        let result = self.check_storage_rate_limit(key).await?;

        // Cache result
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
        let now = TimeUtils::now_unix_secs();

        let window_start = now.saturating_sub(self.config.window_seconds);

        // Clean up old entries to prevent memory leaks if we exceeded max_entries
        if windows.len() > max_entries {
            // Remove entries that have no requests in the current window
            windows.retain(|_, window| {
                while let Some(&(timestamp, _)) = window.front() {
                    if timestamp < window_start {
                        window.pop_front();
                    } else {
                        break;
                    }
                }
                !window.is_empty()
            });

            // If still too many, remove oldest entries based on the first timestamp in their window
            if windows.len() > max_entries {
                let mut entries: Vec<_> = windows
                    .iter()
                    .map(|r| {
                        let first_ts = r.value().front().map(|(ts, _)| *ts).unwrap_or(0);
                        (r.key().clone(), first_ts)
                    })
                    .collect();

                // Sort by oldest first timestamp
                entries.sort_by_key(|(_, ts)| *ts);

                // Remove enough to get under the limit
                let to_remove = windows.len().saturating_sub(max_entries);
                for (key, _) in entries.iter().take(to_remove) {
                    windows.remove(key);
                }
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
        let now = TimeUtils::now_unix_secs() as f64;
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
