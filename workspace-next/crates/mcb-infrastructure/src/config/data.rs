//! Configuration data structures
//!
//! Defines the main configuration structure and related data types
//! for the MCP Context Browser system.

use crate::constants::*;
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Embedding provider configurations
    pub embedding: HashMap<String, EmbeddingConfig>,

    /// Vector store provider configurations
    pub vector_store: HashMap<String, VectorStoreConfig>,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Resilience configuration
    pub resilience: ResilienceConfig,

    /// Limits configuration
    pub limits: LimitsConfig,

    /// Daemon configuration
    pub daemon: DaemonConfig,

    /// Backup configuration
    pub backup: BackupConfig,

    /// Snapshot configuration
    pub snapshot: SnapshotConfig,

    /// Sync configuration
    pub sync: SyncConfig,

    /// Operations configuration
    pub operations: OperationsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            embedding: HashMap::new(),
            vector_store: HashMap::new(),
            logging: LoggingConfig::default(),
            auth: AuthConfig::default(),
            cache: CacheConfig::default(),
            metrics: MetricsConfig::default(),
            resilience: ResilienceConfig::default(),
            limits: LimitsConfig::default(),
            daemon: DaemonConfig::default(),
            backup: BackupConfig::default(),
            snapshot: SnapshotConfig::default(),
            sync: SyncConfig::default(),
            operations: OperationsConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,

    /// HTTPS enabled
    pub https: bool,

    /// SSL certificate path (if HTTPS enabled)
    pub ssl_cert_path: Option<PathBuf>,

    /// SSL key path (if HTTPS enabled)
    pub ssl_key_path: Option<PathBuf>,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Maximum request body size in bytes
    pub max_request_body_size: usize,

    /// Enable CORS
    pub cors_enabled: bool,

    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: DEFAULT_HTTP_PORT,
            https: false,
            ssl_cert_path: None,
            ssl_key_path: None,
            request_timeout_secs: REQUEST_TIMEOUT_SECS,
            connection_timeout_secs: CONNECTION_TIMEOUT_SECS,
            max_request_body_size: MAX_REQUEST_BODY_SIZE,
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}

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

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,

    /// JWT secret key
    pub jwt_secret: String,

    /// JWT expiration time in seconds
    pub jwt_expiration_secs: u64,

    /// JWT refresh token expiration in seconds
    pub jwt_refresh_expiration_secs: u64,

    /// API key authentication enabled
    pub api_key_enabled: bool,

    /// API key header name
    pub api_key_header: String,

    /// User database path
    pub user_db_path: Option<PathBuf>,

    /// Password hashing algorithm
    pub password_algorithm: PasswordAlgorithm,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: crate::crypto::TokenGenerator::generate_secure_token(32),
            jwt_expiration_secs: JWT_DEFAULT_EXPIRATION_SECS,
            jwt_refresh_expiration_secs: JWT_REFRESH_EXPIRATION_SECS,
            api_key_enabled: true,
            api_key_header: API_KEY_HEADER.to_string(),
            user_db_path: None,
            password_algorithm: PasswordAlgorithm::Argon2,
        }
    }
}

/// Password hashing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordAlgorithm {
    /// Argon2id (recommended)
    Argon2,
    /// bcrypt
    Bcrypt,
    /// PBKDF2
    Pbkdf2,
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

/// Cache providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheProvider {
    /// In-memory cache (Moka)
    Moka,
    /// Redis distributed cache
    Redis,
}

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

/// Daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Daemon enabled
    pub enabled: bool,

    /// Process check interval in seconds
    pub check_interval_secs: u64,

    /// Restart delay in seconds
    pub restart_delay_secs: u64,

    /// Maximum restart attempts
    pub max_restart_attempts: u32,

    /// Auto-start daemon
    pub auto_start: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DAEMON_CHECK_INTERVAL_SECS,
            restart_delay_secs: DAEMON_RESTART_DELAY_SECS,
            max_restart_attempts: DAEMON_MAX_RESTART_ATTEMPTS,
            auto_start: true,
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup enabled
    pub enabled: bool,

    /// Backup directory
    pub directory: PathBuf,

    /// Backup interval in seconds
    pub interval_secs: u64,

    /// Maximum number of backups to keep
    pub max_backups: usize,

    /// Compress backups
    pub compress: bool,

    /// Encrypt backups
    pub encrypt: bool,

    /// Backup encryption key (if encryption enabled)
    pub encryption_key: Option<String>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: PathBuf::from("./backups"),
            interval_secs: 86400, // 24 hours
            max_backups: 7,
            compress: true,
            encrypt: false,
            encryption_key: None,
        }
    }
}

/// Snapshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Snapshot enabled
    pub enabled: bool,

    /// Snapshot directory
    pub directory: PathBuf,

    /// Maximum file size for snapshot operations
    pub max_file_size: usize,

    /// Snapshot compression enabled
    pub compression_enabled: bool,

    /// Change detection enabled
    pub change_detection_enabled: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: PathBuf::from("./snapshots"),
            max_file_size: MAX_SNAPSHOT_FILE_SIZE,
            compression_enabled: true,
            change_detection_enabled: true,
        }
    }
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,

    /// Sync batch size
    pub batch_size: usize,

    /// Sync debounce delay in milliseconds
    pub debounce_delay_ms: u64,

    /// Sync timeout in seconds
    pub timeout_secs: u64,

    /// Maximum concurrent sync operations
    pub max_concurrent: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: SYNC_BATCH_SIZE,
            debounce_delay_ms: SYNC_DEBOUNCE_DELAY_MS,
            timeout_secs: SYNC_TIMEOUT_SECS,
            max_concurrent: 10,
        }
    }
}

/// Operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationsConfig {
    /// Operations tracking enabled
    pub tracking_enabled: bool,

    /// Operations cleanup interval in seconds
    pub cleanup_interval_secs: u64,

    /// Operations retention period in seconds
    pub retention_secs: u64,

    /// Maximum operations to keep in memory
    pub max_operations_in_memory: usize,
}

impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            cleanup_interval_secs: OPERATIONS_CLEANUP_INTERVAL_SECS,
            retention_secs: OPERATIONS_RETENTION_SECS,
            max_operations_in_memory: 10000,
        }
    }
}