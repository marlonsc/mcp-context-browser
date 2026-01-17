//! Main application configuration

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export all config types
pub use super::{
    auth::{AuthConfig, JwtConfig, PasswordAlgorithm},
    backup::BackupConfig,
    cache::{CacheConfig, CacheProvider},
    daemon::DaemonConfig,
    limits::LimitsConfig,
    logging::LoggingConfig,
    metrics::MetricsConfig,
    operations::OperationsConfig,
    resilience::ResilienceConfig,
    server::{
        ServerConfig, ServerCorsConfig, ServerNetworkConfig, ServerSslConfig,
        ServerTimeoutConfig, TransportMode,
    },
    snapshot::SnapshotConfig,
    sync::SyncConfig,
};

/// Provider configurations (embedding and vector store)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    /// Embedding provider configurations
    pub embedding: HashMap<String, EmbeddingConfig>,

    /// Vector store provider configurations
    pub vector_store: HashMap<String, VectorStoreConfig>,
}

/// Infrastructure configurations (cache, metrics, resilience, limits)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfrastructureConfig {
    /// Cache configuration
    pub cache: CacheConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Resilience configuration
    pub resilience: ResilienceConfig,

    /// Limits configuration
    pub limits: LimitsConfig,
}

/// Data management configurations (snapshot, sync, backup)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DataConfig {
    /// Snapshot configuration
    pub snapshot: SnapshotConfig,

    /// Sync configuration
    pub sync: SyncConfig,

    /// Backup configuration
    pub backup: BackupConfig,
}

/// System infrastructure and data configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Infrastructure configurations
    pub infrastructure: InfrastructureConfig,

    /// Data management configurations
    pub data: DataConfig,
}

/// Operations and daemon configurations combined
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationsDaemonConfig {
    /// Daemon configuration
    pub daemon: DaemonConfig,

    /// Operations configuration
    pub operations: OperationsConfig,
}

/// Main application configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Provider configurations
    pub providers: ProvidersConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// System configurations (infrastructure and data)
    pub system: SystemConfig,

    /// Operations and daemon configurations
    pub operations_daemon: OperationsDaemonConfig,
}