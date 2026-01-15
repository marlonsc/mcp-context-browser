//! Infrastructure layer constants
//!
//! Contains constants that are part of the infrastructure implementation.
//! Domain-specific constants are defined in `mcb_domain::constants`.

// ============================================================================
// CONFIGURATION CONSTANTS
// ============================================================================

/// Default configuration file name
pub const DEFAULT_CONFIG_FILENAME: &str = "mcb.toml";

/// Default configuration directory name
pub const DEFAULT_CONFIG_DIR: &str = "mcb";

/// Environment variable prefix for configuration
pub const CONFIG_ENV_PREFIX: &str = "MCB";

// ============================================================================
// AUTHENTICATION CONSTANTS
// ============================================================================

/// JWT default expiration time in seconds (24 hours)
pub const JWT_DEFAULT_EXPIRATION_SECS: u64 = 86400;

/// JWT refresh token expiration time in seconds (7 days)
pub const JWT_REFRESH_EXPIRATION_SECS: u64 = 604800;

/// Default bcrypt cost for password hashing
pub const BCRYPT_DEFAULT_COST: u32 = 12;

/// API key header name
pub const API_KEY_HEADER: &str = "x-api-key";

/// Authorization header name
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// Bearer token prefix
pub const BEARER_PREFIX: &str = "Bearer ";

// ============================================================================
// CACHE CONSTANTS
// ============================================================================

/// Default cache TTL in seconds (1 hour)
pub const CACHE_DEFAULT_TTL_SECS: u64 = 3600;

/// Default cache size limit in bytes (100MB)
pub const CACHE_DEFAULT_SIZE_LIMIT: usize = 100 * 1024 * 1024;

/// Redis connection pool size
pub const REDIS_POOL_SIZE: usize = 10;

/// Cache namespace separator
pub const CACHE_NAMESPACE_SEPARATOR: &str = ":";

// ============================================================================
// HTTP SERVER CONSTANTS
// ============================================================================

/// Default HTTP server port
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default HTTPS server port
pub const DEFAULT_HTTPS_PORT: u16 = 8443;

/// Default server host
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// Request timeout in seconds
pub const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Connection timeout in seconds
pub const CONNECTION_TIMEOUT_SECS: u64 = 10;

/// Maximum request body size in bytes (10MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Health check endpoint path
pub const HEALTH_CHECK_PATH: &str = "/health";

/// Metrics endpoint path
pub const METRICS_PATH: &str = "/metrics";

// ============================================================================
// RESILIENCE CONSTANTS
// ============================================================================

/// Circuit breaker failure threshold
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: u32 = 5;

/// Circuit breaker timeout in seconds
pub const CIRCUIT_BREAKER_TIMEOUT_SECS: u64 = 60;

/// Circuit breaker success threshold
pub const CIRCUIT_BREAKER_SUCCESS_THRESHOLD: u32 = 3;

/// Rate limiter default requests per second
pub const RATE_LIMITER_DEFAULT_RPS: u32 = 100;

/// Rate limiter burst size
pub const RATE_LIMITER_DEFAULT_BURST: u32 = 200;

// ============================================================================
// METRICS CONSTANTS
// ============================================================================

/// Metrics collection interval in seconds
pub const METRICS_COLLECTION_INTERVAL_SECS: u64 = 60;

/// Prometheus metrics prefix
pub const METRICS_PREFIX: &str = "mcb";

// ============================================================================
// FILE SYSTEM CONSTANTS
// ============================================================================

/// Default file permissions (0o644)
pub const DEFAULT_FILE_PERMISSIONS: u32 = 0o644;

/// Default directory permissions (0o755)
pub const DEFAULT_DIR_PERMISSIONS: u32 = 0o755;

/// Maximum file size for snapshot operations in bytes (100MB)
pub const MAX_SNAPSHOT_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Backup file extension
pub const BACKUP_FILE_EXTENSION: &str = ".backup";

/// Temporary file prefix
pub const TEMP_FILE_PREFIX: &str = "mcb_temp_";

// ============================================================================
// DATABASE CONSTANTS
// ============================================================================

/// Default database connection pool size
pub const DB_POOL_SIZE: u32 = 10;

/// Database connection timeout in seconds
pub const DB_CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Database query timeout in seconds
pub const DB_QUERY_TIMEOUT_SECS: u64 = 60;

// ============================================================================
// EVENT BUS CONSTANTS
// ============================================================================

/// NATS default connection timeout in seconds
pub const NATS_CONNECT_TIMEOUT_SECS: u64 = 10;

/// NATS default request timeout in seconds
pub const NATS_REQUEST_TIMEOUT_SECS: u64 = 5;

/// Event bus buffer size
pub const EVENT_BUS_BUFFER_SIZE: usize = 1000;

// ============================================================================
// LOGGING CONSTANTS
// ============================================================================

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Log file rotation size in bytes (10MB)
pub const LOG_ROTATION_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum number of log files to keep
pub const LOG_MAX_FILES: usize = 5;

// ============================================================================
// DAEMON CONSTANTS
// ============================================================================

/// Daemon process check interval in seconds
pub const DAEMON_CHECK_INTERVAL_SECS: u64 = 30;

/// Daemon restart delay in seconds
pub const DAEMON_RESTART_DELAY_SECS: u64 = 5;

/// Maximum restart attempts
pub const DAEMON_MAX_RESTART_ATTEMPTS: u32 = 3;

// ============================================================================
// SHUTDOWN CONSTANTS
// ============================================================================

/// Graceful shutdown timeout in seconds
pub const GRACEFUL_SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// Force shutdown timeout in seconds
pub const FORCE_SHUTDOWN_TIMEOUT_SECS: u64 = 10;

// ============================================================================
// SIGNAL CONSTANTS
// ============================================================================

/// Signal handling poll interval in milliseconds
pub const SIGNAL_POLL_INTERVAL_MS: u64 = 100;

// ============================================================================
// CRYPTO CONSTANTS
// ============================================================================

/// AES-GCM key size in bytes
pub const AES_GCM_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// PBKDF2 iterations for key derivation
pub const PBKDF2_ITERATIONS: u32 = 100_000;

// ============================================================================
// SYNC CONSTANTS
// ============================================================================

/// Sync batch size
pub const SYNC_BATCH_SIZE: usize = 100;

/// Sync debounce delay in milliseconds
pub const SYNC_DEBOUNCE_DELAY_MS: u64 = 500;

/// Sync timeout in seconds
pub const SYNC_TIMEOUT_SECS: u64 = 300;

// ============================================================================
// LIMITS CONSTANTS
// ============================================================================

/// Default memory limit in bytes (1GB)
pub const DEFAULT_MEMORY_LIMIT: usize = 1 * 1024 * 1024 * 1024;

/// Default CPU limit (number of cores)
pub const DEFAULT_CPU_LIMIT: usize = 4;

/// Default disk I/O limit in bytes per second (100MB/s)
pub const DEFAULT_DISK_IO_LIMIT: u64 = 100 * 1024 * 1024;

// ============================================================================
// OPERATIONS CONSTANTS
// ============================================================================

/// Operations tracking cleanup interval in seconds
pub const OPERATIONS_CLEANUP_INTERVAL_SECS: u64 = 3600;

/// Operations tracking retention period in seconds (7 days)
pub const OPERATIONS_RETENTION_SECS: u64 = 604800;

// ============================================================================
// HEALTH CHECK CONSTANTS
// ============================================================================

/// Health check timeout in seconds
pub const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

/// Health check interval in seconds
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// Health check failure threshold
pub const HEALTH_CHECK_FAILURE_THRESHOLD: u32 = 3;

// Re-export domain constants for convenience
pub use mcb_domain::constants::*;