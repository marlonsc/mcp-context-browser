//! Infrastructure configuration types
//!
//! Consolidated configuration for infrastructure concerns:
//! logging, limits, cache, metrics, and resilience.

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Logging Configuration
// ============================================================================

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Enable JSON output format
    pub json_format: bool,
    /// Log to file in addition to stdout
    pub file_output: Option<PathBuf>,
    /// Maximum file size before rotation (bytes)
    pub max_file_size: u64,
    /// Maximum number of rotated files to keep
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: DEFAULT_LOG_LEVEL.to_string(),
            json_format: false,
            file_output: None,
            max_file_size: LOG_ROTATION_SIZE,
            max_files: LOG_MAX_FILES,
        }
    }
}

// ============================================================================
// Resource Limits Configuration
// ============================================================================

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Memory limit in bytes
    pub memory_limit: usize,
    /// CPU limit (number of cores)
    pub cpu_limit: usize,
    /// Disk I/O limit in bytes per second
    pub disk_io_limit: u64,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Maximum concurrent requests per connection
    pub max_requests_per_connection: u32,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            cpu_limit: DEFAULT_CPU_LIMIT,
            disk_io_limit: DEFAULT_DISK_IO_LIMIT,
            max_connections: 1000,
            max_requests_per_connection: 100,
        }
    }
}

// ============================================================================
// Cache Configuration
// ============================================================================

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
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheProvider::Moka => "moka",
            CacheProvider::Redis => "redis",
        }
    }
}

/// Infrastructure cache system configuration
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

// ============================================================================
// Metrics Configuration
// ============================================================================

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metrics enabled
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Prometheus metrics prefix
    pub prefix: String,
    /// Metrics endpoint enabled
    pub endpoint_enabled: bool,
    /// Metrics endpoint path
    pub endpoint_path: String,
    /// External metrics exporter URL
    pub exporter_url: Option<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: METRICS_COLLECTION_INTERVAL_SECS,
            prefix: METRICS_PREFIX.to_string(),
            endpoint_enabled: true,
            endpoint_path: METRICS_PATH.to_string(),
            exporter_url: None,
        }
    }
}

// ============================================================================
// Resilience Configuration
// ============================================================================

/// Resilience configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_secs: u64,
    /// Circuit breaker success threshold
    pub circuit_breaker_success_threshold: u32,
    /// Rate limiter requests per second
    pub rate_limiter_rps: u32,
    /// Rate limiter burst size
    pub rate_limiter_burst: u32,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_failure_threshold: CIRCUIT_BREAKER_FAILURE_THRESHOLD,
            circuit_breaker_timeout_secs: CIRCUIT_BREAKER_TIMEOUT_SECS,
            circuit_breaker_success_threshold: CIRCUIT_BREAKER_SUCCESS_THRESHOLD,
            rate_limiter_rps: RATE_LIMITER_DEFAULT_RPS,
            rate_limiter_burst: RATE_LIMITER_DEFAULT_BURST,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}
