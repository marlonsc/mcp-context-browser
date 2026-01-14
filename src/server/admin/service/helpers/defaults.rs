//! Default configuration constants
//!
//! Centralized default values for runtime configuration thresholds and system metrics.
//! These values are used when environment variables are not configured.

use crate::infrastructure::constants::{
    HEALTH_CACHE_HIT_RATE_DEGRADED, HEALTH_CPU_DEGRADED_PERCENT, HEALTH_CPU_UNHEALTHY_PERCENT,
    HEALTH_DB_POOL_DEGRADED_PERCENT, HEALTH_DB_POOL_UNHEALTHY_PERCENT,
    HEALTH_DISK_DEGRADED_PERCENT, HEALTH_DISK_UNHEALTHY_PERCENT, HEALTH_MEMORY_DEGRADED_PERCENT,
    HEALTH_MEMORY_UNHEALTHY_PERCENT, PERF_P95_MULTIPLIER, PERF_P99_MULTIPLIER,
};

// Health Check Thresholds - CPU
/// CPU usage percentage considered unhealthy for health checks
pub const DEFAULT_HEALTH_CPU_UNHEALTHY_PERCENT: f64 = HEALTH_CPU_UNHEALTHY_PERCENT;
/// CPU usage percentage considered degraded for health checks
pub const DEFAULT_HEALTH_CPU_DEGRADED_PERCENT: f64 = HEALTH_CPU_DEGRADED_PERCENT;

// Health Check Thresholds - Memory
/// Memory usage percentage considered unhealthy for health checks
pub const DEFAULT_HEALTH_MEMORY_UNHEALTHY_PERCENT: f64 = HEALTH_MEMORY_UNHEALTHY_PERCENT;
/// Memory usage percentage considered degraded for health checks
pub const DEFAULT_HEALTH_MEMORY_DEGRADED_PERCENT: f64 = HEALTH_MEMORY_DEGRADED_PERCENT;

// Health Check Thresholds - Disk
/// Disk usage percentage considered unhealthy for health checks
pub const DEFAULT_HEALTH_DISK_UNHEALTHY_PERCENT: f64 = HEALTH_DISK_UNHEALTHY_PERCENT;
/// Disk usage percentage considered degraded for health checks
pub const DEFAULT_HEALTH_DISK_DEGRADED_PERCENT: f64 = HEALTH_DISK_DEGRADED_PERCENT;

// Health Check Thresholds - Database Pool
/// Database connection pool usage percentage considered unhealthy
pub const DEFAULT_HEALTH_DB_POOL_UNHEALTHY_PERCENT: f64 = HEALTH_DB_POOL_UNHEALTHY_PERCENT;
/// Database connection pool usage percentage considered degraded
pub const DEFAULT_HEALTH_DB_POOL_DEGRADED_PERCENT: f64 = HEALTH_DB_POOL_DEGRADED_PERCENT;

// Health Check Thresholds - Cache Hit Rate
/// Cache hit rate percentage considered degraded for health checks
pub const DEFAULT_HEALTH_CACHE_HIT_RATE_DEGRADED: f64 = HEALTH_CACHE_HIT_RATE_DEGRADED; // 50%

// Performance Test Multipliers
/// P95 latency multiplier for performance testing
pub const DEFAULT_PERF_P95_MULTIPLIER: f64 = PERF_P95_MULTIPLIER;
/// P99 latency multiplier for performance testing
pub const DEFAULT_PERF_P99_MULTIPLIER: f64 = PERF_P99_MULTIPLIER;

// Indexing Configuration
/// Default state for indexing enablement
pub const DEFAULT_INDEXING_ENABLED: bool = true;
/// Default number of pending indexing operations
pub const DEFAULT_INDEXING_PENDING_OPERATIONS: u64 = 0;

// Cache Configuration
/// Default state for cache enablement
pub const DEFAULT_CACHE_ENABLED: bool = true;
/// Default number of cache entries
pub const DEFAULT_CACHE_ENTRIES_COUNT: u64 = 0;
/// Default cache hit rate
pub const DEFAULT_CACHE_HIT_RATE: f64 = 0.0;
/// Default cache size in bytes
pub const DEFAULT_CACHE_SIZE_BYTES: u64 = 0;
/// Default maximum cache size in bytes (10GB)
pub const DEFAULT_CACHE_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024 * 1024; // 10GB

// Database Configuration
/// Default database connection state
pub const DEFAULT_DB_CONNECTED: bool = true;
/// Default number of active database connections
pub const DEFAULT_DB_ACTIVE_CONNECTIONS: u32 = 0;
/// Default number of idle database connections
pub const DEFAULT_DB_IDLE_CONNECTIONS: u32 = 0;
/// Default database connection pool size
pub const DEFAULT_DB_POOL_SIZE: u32 = 20;
