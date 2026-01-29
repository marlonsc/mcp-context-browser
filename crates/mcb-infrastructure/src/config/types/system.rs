//! System configuration types
//!
//! Consolidated configuration for system concerns:
//! auth, event_bus, backup, sync, snapshot, daemon, and operations.

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Authentication Configuration
// ============================================================================

/// Password hashing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordAlgorithm {
    Argon2,
    Bcrypt,
    Pbkdf2,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key (REQUIRED when auth enabled, min 32 chars)
    pub secret: String,
    /// JWT expiration time in seconds
    pub expiration_secs: u64,
    /// JWT refresh token expiration in seconds
    pub refresh_expiration_secs: u64,
}

/// Default JWT configuration using infrastructure constants.
///
/// - `secret`: empty (must be configured)
/// - `expiration_secs`: `JWT_DEFAULT_EXPIRATION_SECS`
/// - `refresh_expiration_secs`: `JWT_REFRESH_EXPIRATION_SECS`
impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            expiration_secs: JWT_DEFAULT_EXPIRATION_SECS,
            refresh_expiration_secs: JWT_REFRESH_EXPIRATION_SECS,
        }
    }
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key authentication enabled
    pub enabled: bool,
    /// API key header name
    pub header: String,
}

/// Default API key configuration using infrastructure constants.
///
/// - `enabled`: true
/// - `header`: `API_KEY_HEADER`
impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            header: API_KEY_HEADER.to_string(),
        }
    }
}

fn default_admin_key_header() -> String {
    DEFAULT_ADMIN_KEY_HEADER.to_string()
}

/// Admin API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiKeyConfig {
    /// Admin API key authentication enabled
    pub enabled: bool,
    /// Header name for admin API key
    #[serde(default = "default_admin_key_header")]
    pub header: String,
    /// The actual admin API key
    #[serde(default)]
    pub key: Option<String>,
}

/// Default admin API key configuration.
///
/// - `enabled`: false (admin API disabled by default)
/// - `header`: `DEFAULT_ADMIN_KEY_HEADER`
/// - `key`: None
impl Default for AdminApiKeyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            header: default_admin_key_header(),
            key: None,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// API key configuration
    pub api_key: ApiKeyConfig,
    /// Admin API key configuration
    #[serde(default)]
    pub admin: AdminApiKeyConfig,
    /// User database path
    pub user_db_path: Option<PathBuf>,
    /// Password hashing algorithm
    pub password_algorithm: PasswordAlgorithm,
}

/// Default authentication configuration.
///
/// - `enabled`: false (disabled by default for backwards compatibility)
/// - `jwt`: JwtConfig::default()
/// - `api_key`: ApiKeyConfig::default()
/// - `admin`: AdminApiKeyConfig::default()
/// - `password_algorithm`: Argon2
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jwt: JwtConfig::default(),
            api_key: ApiKeyConfig::default(),
            admin: AdminApiKeyConfig::default(),
            user_db_path: None,
            password_algorithm: PasswordAlgorithm::Argon2,
        }
    }
}

// ============================================================================
// EventBus Configuration
// ============================================================================

/// EventBus provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventBusProvider {
    /// In-process broadcast channel (Tokio)
    #[default]
    Tokio,
    /// Distributed message queue (NATS)
    Nats,
    /// No-op event bus for testing
    Null,
}

/// EventBus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusConfig {
    /// EventBus provider to use
    pub provider: EventBusProvider,
    /// Buffer capacity for in-process event bus
    pub capacity: usize,
    /// NATS server URL (for NATS provider)
    pub nats_url: Option<String>,
    /// NATS client name (for NATS provider)
    pub nats_client_name: Option<String>,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Reconnection attempts for distributed providers
    pub max_reconnect_attempts: u32,
}

/// Default event bus configuration.
///
/// - `provider`: Tokio (in-process)
/// - `capacity`: 1024 events
/// - `nats_client_name`: `DEFAULT_NATS_CLIENT_NAME`
/// - `connection_timeout_ms`: 5000
/// - `max_reconnect_attempts`: 5
impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            provider: EventBusProvider::Tokio,
            capacity: 1024,
            nats_url: None,
            nats_client_name: Some(DEFAULT_NATS_CLIENT_NAME.to_string()),
            connection_timeout_ms: 5000,
            max_reconnect_attempts: 5,
        }
    }
}

impl EventBusConfig {
    pub fn tokio() -> Self {
        Self::default()
    }

    pub fn tokio_with_capacity(capacity: usize) -> Self {
        Self {
            provider: EventBusProvider::Tokio,
            capacity,
            ..Default::default()
        }
    }

    pub fn nats(url: impl Into<String>) -> Self {
        Self {
            provider: EventBusProvider::Nats,
            nats_url: Some(url.into()),
            ..Default::default()
        }
    }

    pub fn null() -> Self {
        Self {
            provider: EventBusProvider::Null,
            ..Default::default()
        }
    }
}

// ============================================================================
// Backup Configuration
// ============================================================================

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

/// Default backup configuration.
///
/// - `enabled`: false
/// - `directory`: ./backups
/// - `interval_secs`: 86400 (24 hours)
/// - `max_backups`: 7
/// - `compress`: true
impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: PathBuf::from("./backups"),
            interval_secs: 86400,
            max_backups: 7,
            compress: true,
            encrypt: false,
            encryption_key: None,
        }
    }
}

// ============================================================================
// Sync Configuration
// ============================================================================

fn default_watching_enabled() -> bool {
    true
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync enabled
    pub enabled: bool,
    /// Enable file watching for hot-reload
    #[serde(default = "default_watching_enabled")]
    pub watching_enabled: bool,
    /// Sync batch size
    pub batch_size: usize,
    /// Sync debounce delay in milliseconds
    pub debounce_delay_ms: u64,
    /// Sync timeout in seconds
    pub timeout_secs: u64,
    /// Maximum concurrent sync operations
    pub max_concurrent: usize,
}

/// Default sync configuration using infrastructure constants.
///
/// - `enabled`: true
/// - `watching_enabled`: true
/// - `batch_size`: `SYNC_BATCH_SIZE`
/// - `debounce_delay_ms`: `SYNC_DEBOUNCE_DELAY_MS`
/// - `timeout_secs`: `SYNC_TIMEOUT_SECS`
impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watching_enabled: default_watching_enabled(),
            batch_size: SYNC_BATCH_SIZE,
            debounce_delay_ms: SYNC_DEBOUNCE_DELAY_MS,
            timeout_secs: SYNC_TIMEOUT_SECS,
            max_concurrent: 10,
        }
    }
}

// ============================================================================
// Snapshot Configuration
// ============================================================================

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

/// Default snapshot configuration.
///
/// - `enabled`: true
/// - `directory`: ./snapshots
/// - `max_file_size`: `MAX_SNAPSHOT_FILE_SIZE`
/// - `compression_enabled`: true
/// - `change_detection_enabled`: true
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

// ============================================================================
// Daemon Configuration
// ============================================================================

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

/// Default daemon configuration using infrastructure constants.
///
/// - `enabled`: true
/// - `check_interval_secs`: `DAEMON_CHECK_INTERVAL_SECS`
/// - `restart_delay_secs`: `DAEMON_RESTART_DELAY_SECS`
/// - `max_restart_attempts`: `DAEMON_MAX_RESTART_ATTEMPTS`
/// - `auto_start`: true
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

// ============================================================================
// Operations Configuration
// ============================================================================

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

/// Default operations configuration using infrastructure constants.
///
/// - `tracking_enabled`: true
/// - `cleanup_interval_secs`: `OPERATIONS_CLEANUP_INTERVAL_SECS`
/// - `retention_secs`: `OPERATIONS_RETENTION_SECS`
/// - `max_operations_in_memory`: `OPERATIONS_MAX_IN_MEMORY`
impl Default for OperationsConfig {
    fn default() -> Self {
        Self {
            tracking_enabled: true,
            cleanup_interval_secs: OPERATIONS_CLEANUP_INTERVAL_SECS,
            retention_secs: OPERATIONS_RETENTION_SECS,
            max_operations_in_memory: OPERATIONS_MAX_IN_MEMORY,
        }
    }
}
