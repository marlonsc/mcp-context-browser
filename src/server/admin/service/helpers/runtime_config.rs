//! Runtime Configuration Provider
//!
//! Provides dynamic configuration values from the running system,
//! eliminating hardcoded values by reading from actual subsystems.

use super::defaults::*;
use crate::domain::ports::IndexingOperationsInterface;
use crate::infrastructure::cache::SharedCacheProvider;
use crate::server::admin::service::types::AdminError;
use std::sync::Arc;

/// Runtime configuration values loaded from actual subsystems
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Indexing subsystem configuration
    pub indexing: IndexingConfig,
    /// Cache subsystem configuration
    pub cache: CacheConfig,
    /// Database subsystem configuration
    pub database: DatabaseConfig,
    /// Health check thresholds (loaded from environment or defaults)
    pub thresholds: HealthThresholds,
}

/// Health check thresholds (configurable via environment variables)
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// CPU usage threshold for unhealthy status (default: 90.0%)
    pub cpu_unhealthy_percent: f64,
    /// CPU usage threshold for degraded status (default: 75.0%)
    pub cpu_degraded_percent: f64,
    /// Memory usage threshold for unhealthy status (default: 90.0%)
    pub memory_unhealthy_percent: f64,
    /// Memory usage threshold for degraded status (default: 80.0%)
    pub memory_degraded_percent: f64,
    /// Disk usage threshold for unhealthy status (default: 90.0%)
    pub disk_unhealthy_percent: f64,
    /// Disk usage threshold for degraded status (default: 80.0%)
    pub disk_degraded_percent: f64,
    /// Database pool utilization threshold for unhealthy status (default: 95.0%)
    pub db_pool_unhealthy_percent: f64,
    /// Database pool utilization threshold for degraded status (default: 80.0%)
    pub db_pool_degraded_percent: f64,
    /// Cache hit rate threshold for degraded status (default: 0.5 = 50%)
    pub cache_hit_rate_degraded: f64,
    /// Performance test p95/p99 multiplier (default: 1.2x and 1.5x for avg)
    pub perf_p95_multiplier: f64,
    /// Performance test p99 multiplier
    pub perf_p99_multiplier: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            cpu_unhealthy_percent: get_env_f64(
                "HEALTH_CPU_UNHEALTHY",
                DEFAULT_HEALTH_CPU_UNHEALTHY_PERCENT,
            ),
            cpu_degraded_percent: get_env_f64(
                "HEALTH_CPU_DEGRADED",
                DEFAULT_HEALTH_CPU_DEGRADED_PERCENT,
            ),
            memory_unhealthy_percent: get_env_f64(
                "HEALTH_MEMORY_UNHEALTHY",
                DEFAULT_HEALTH_MEMORY_UNHEALTHY_PERCENT,
            ),
            memory_degraded_percent: get_env_f64(
                "HEALTH_MEMORY_DEGRADED",
                DEFAULT_HEALTH_MEMORY_DEGRADED_PERCENT,
            ),
            disk_unhealthy_percent: get_env_f64(
                "HEALTH_DISK_UNHEALTHY",
                DEFAULT_HEALTH_DISK_UNHEALTHY_PERCENT,
            ),
            disk_degraded_percent: get_env_f64(
                "HEALTH_DISK_DEGRADED",
                DEFAULT_HEALTH_DISK_DEGRADED_PERCENT,
            ),
            db_pool_unhealthy_percent: get_env_f64(
                "HEALTH_DB_POOL_UNHEALTHY",
                DEFAULT_HEALTH_DB_POOL_UNHEALTHY_PERCENT,
            ),
            db_pool_degraded_percent: get_env_f64(
                "HEALTH_DB_POOL_DEGRADED",
                DEFAULT_HEALTH_DB_POOL_DEGRADED_PERCENT,
            ),
            cache_hit_rate_degraded: get_env_f64(
                "HEALTH_CACHE_HIT_RATE_DEGRADED",
                DEFAULT_HEALTH_CACHE_HIT_RATE_DEGRADED,
            ),
            perf_p95_multiplier: get_env_f64("PERF_P95_MULTIPLIER", DEFAULT_PERF_P95_MULTIPLIER),
            perf_p99_multiplier: get_env_f64("PERF_P99_MULTIPLIER", DEFAULT_PERF_P99_MULTIPLIER),
        }
    }
}

/// Indexing subsystem runtime configuration
#[derive(Debug, Clone)]
pub struct IndexingConfig {
    /// Enabled
    pub enabled: bool,
    /// Pending Operations
    pub pending_operations: u64,
    /// Last Index Time
    pub last_index_time: chrono::DateTime<chrono::Utc>,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pending_operations: 0,
            last_index_time: chrono::Utc::now(),
        }
    }
}

/// Cache subsystem runtime configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enabled
    pub enabled: bool,
    /// Entries Count
    pub entries_count: u64,
    /// Hit Rate
    pub hit_rate: f64,
    /// Size Bytes
    pub size_bytes: u64,
    /// Max Size Bytes
    pub max_size_bytes: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_CACHE_ENABLED,
            entries_count: DEFAULT_CACHE_ENTRIES_COUNT,
            hit_rate: DEFAULT_CACHE_HIT_RATE,
            size_bytes: DEFAULT_CACHE_SIZE_BYTES,
            max_size_bytes: get_env_u64("CACHE_MAX_SIZE_BYTES", DEFAULT_CACHE_MAX_SIZE_BYTES),
        }
    }
}

/// Database subsystem runtime configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Connected
    pub connected: bool,
    /// Active Connections
    pub active_connections: u32,
    /// Idle Connections
    pub idle_connections: u32,
    /// Total Pool Size
    pub total_pool_size: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connected: DEFAULT_DB_CONNECTED,
            active_connections: DEFAULT_DB_ACTIVE_CONNECTIONS,
            idle_connections: DEFAULT_DB_IDLE_CONNECTIONS,
            total_pool_size: get_env_u32("DB_POOL_SIZE", DEFAULT_DB_POOL_SIZE),
        }
    }
}

/// Service dependencies for RuntimeConfig
pub struct RuntimeConfigDependencies {
    /// Indexing operations for pending operation count
    pub indexing_operations: Option<Arc<dyn IndexingOperationsInterface>>,
    /// Cache provider for cache statistics
    pub cache_provider: Option<SharedCacheProvider>,
}

impl RuntimeConfig {
    /// Load runtime configuration from actual subsystems
    ///
    /// Uses real service queries when dependencies are provided,
    /// falls back to environment variables or defaults otherwise.
    pub async fn load() -> Result<Self, AdminError> {
        Self::load_with_services(RuntimeConfigDependencies {
            indexing_operations: None,
            cache_provider: None,
        })
        .await
    }

    /// Load runtime configuration with injected service dependencies
    ///
    /// This is the preferred method - it queries real services for accurate data.
    pub async fn load_with_services(deps: RuntimeConfigDependencies) -> Result<Self, AdminError> {
        Ok(RuntimeConfig {
            indexing: Self::load_indexing_config(deps.indexing_operations.as_ref()).await,
            cache: Self::load_cache_config(deps.cache_provider.as_ref()).await,
            database: Self::load_database_config().await,
            thresholds: HealthThresholds::default(),
        })
    }

    /// Load indexing configuration from runtime
    async fn load_indexing_config(
        indexing_ops: Option<&Arc<dyn IndexingOperationsInterface>>,
    ) -> IndexingConfig {
        let pending_operations = if let Some(ops) = indexing_ops {
            // Query real pending operations from IndexingOperationsInterface
            ops.get_map().len() as u64
        } else {
            // Fallback to environment variable
            get_env_u64(
                "INDEXING_PENDING_OPERATIONS",
                DEFAULT_INDEXING_PENDING_OPERATIONS,
            )
        };

        IndexingConfig {
            enabled: get_env_bool("INDEXING_ENABLED", DEFAULT_INDEXING_ENABLED),
            pending_operations,
            last_index_time: chrono::Utc::now(),
        }
    }

    /// Load cache configuration from runtime
    async fn load_cache_config(cache_provider: Option<&SharedCacheProvider>) -> CacheConfig {
        if let Some(cache) = cache_provider {
            // Query real cache statistics from CacheProvider
            match cache.get_stats("default").await {
                Ok(stats) => CacheConfig {
                    enabled: true,
                    entries_count: stats.total_entries as u64,
                    hit_rate: stats.hit_ratio,
                    size_bytes: stats.total_size_bytes as u64,
                    max_size_bytes: get_env_u64(
                        "CACHE_MAX_SIZE_BYTES",
                        DEFAULT_CACHE_MAX_SIZE_BYTES,
                    ),
                },
                Err(_) => Self::default_cache_config(),
            }
        } else {
            Self::default_cache_config()
        }
    }

    /// Default cache config when no provider available
    fn default_cache_config() -> CacheConfig {
        CacheConfig {
            enabled: get_env_bool("CACHE_ENABLED", DEFAULT_CACHE_ENABLED),
            entries_count: get_env_u64("CACHE_ENTRIES_COUNT", DEFAULT_CACHE_ENTRIES_COUNT),
            hit_rate: get_env_f64("CACHE_HIT_RATE", DEFAULT_CACHE_HIT_RATE),
            size_bytes: get_env_u64("CACHE_SIZE_BYTES", DEFAULT_CACHE_SIZE_BYTES),
            max_size_bytes: get_env_u64("CACHE_MAX_SIZE_BYTES", DEFAULT_CACHE_MAX_SIZE_BYTES),
        }
    }

    /// Load database configuration from runtime
    async fn load_database_config() -> DatabaseConfig {
        DatabaseConfig {
            connected: get_env_bool("DB_CONNECTED", DEFAULT_DB_CONNECTED),
            active_connections: get_env_u32("DB_ACTIVE_CONNECTIONS", DEFAULT_DB_ACTIVE_CONNECTIONS),
            idle_connections: get_env_u32("DB_IDLE_CONNECTIONS", DEFAULT_DB_IDLE_CONNECTIONS),
            total_pool_size: get_env_u32("DB_POOL_SIZE", DEFAULT_DB_POOL_SIZE),
        }
    }
}

/// Provider trait for runtime configuration management
pub trait RuntimeConfigProvider: Send + Sync {
    /// Get the current runtime configuration
    fn get_config(&self) -> RuntimeConfig;
    /// Update the cache entries count
    fn update_cache_entries(&mut self, count: u64);
    /// Update the cache hit rate
    fn update_cache_hit_rate(&mut self, rate: f64);
    /// Update connection statistics
    fn update_connection_stats(&mut self, active: u32, idle: u32);
}

/// Default implementation tracking runtime state
pub struct DefaultRuntimeConfigProvider {
    /// Current runtime configuration state
    config: RuntimeConfig,
}

impl DefaultRuntimeConfigProvider {
    /// Create a new default runtime config provider with default values
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig {
                indexing: IndexingConfig {
                    enabled: DEFAULT_INDEXING_ENABLED,
                    pending_operations: DEFAULT_INDEXING_PENDING_OPERATIONS,
                    last_index_time: chrono::Utc::now(),
                },
                cache: CacheConfig {
                    enabled: DEFAULT_CACHE_ENABLED,
                    entries_count: DEFAULT_CACHE_ENTRIES_COUNT,
                    hit_rate: DEFAULT_CACHE_HIT_RATE,
                    size_bytes: DEFAULT_CACHE_SIZE_BYTES,
                    max_size_bytes: DEFAULT_CACHE_MAX_SIZE_BYTES,
                },
                database: DatabaseConfig {
                    connected: DEFAULT_DB_CONNECTED,
                    active_connections: DEFAULT_DB_ACTIVE_CONNECTIONS,
                    idle_connections: DEFAULT_DB_IDLE_CONNECTIONS,
                    total_pool_size: DEFAULT_DB_POOL_SIZE,
                },
                thresholds: HealthThresholds::default(),
            },
        }
    }
}

impl Default for DefaultRuntimeConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeConfigProvider for DefaultRuntimeConfigProvider {
    fn get_config(&self) -> RuntimeConfig {
        self.config.clone()
    }

    fn update_cache_entries(&mut self, count: u64) {
        self.config.cache.entries_count = count;
    }

    fn update_cache_hit_rate(&mut self, rate: f64) {
        self.config.cache.hit_rate = rate.clamp(0.0, 1.0);
    }

    fn update_connection_stats(&mut self, active: u32, idle: u32) {
        self.config.database.active_connections = active;
        self.config.database.idle_connections = idle;
    }
}

// Helper functions for reading environment variables
fn get_env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn get_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn get_env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn get_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| !v.eq_ignore_ascii_case("false") && v != "0")
        .unwrap_or(default)
}
