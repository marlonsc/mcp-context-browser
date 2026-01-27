//! Main application configuration

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export all config types from consolidated modules
pub use super::infrastructure::{
    CacheProvider, CacheSystemConfig, LimitsConfig, LoggingConfig, MetricsConfig, ResilienceConfig,
};
pub use super::mode::{ModeConfig, OperatingMode};
pub use super::server::{
    ServerConfig, ServerCorsConfig, ServerNetworkConfig, ServerSslConfig, ServerTimeoutConfig,
    TransportMode,
};
pub use super::system::{
    AdminApiKeyConfig, ApiKeyConfig, AuthConfig, BackupConfig, DaemonConfig, EventBusConfig,
    EventBusProvider, JwtConfig, OperationsConfig, PasswordAlgorithm, SnapshotConfig, SyncConfig,
};

/// Embedding configuration container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingConfigContainer {
    /// Provider name
    pub provider: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Base URL for API
    pub base_url: Option<String>,
    /// API key
    pub api_key: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// Named configs for TOML format
    #[serde(default)]
    pub configs: HashMap<String, EmbeddingConfig>,
}

/// Vector store configuration container
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorStoreConfigContainer {
    /// Provider name
    pub provider: Option<String>,
    /// Server address
    pub address: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// Collection name
    pub collection: Option<String>,
    /// Named configs for TOML format
    #[serde(default)]
    pub configs: HashMap<String, VectorStoreConfig>,
}

/// Provider configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    /// Embedding provider configuration
    #[serde(default)]
    pub embedding: EmbeddingConfigContainer,
    /// Vector store provider configuration
    #[serde(default)]
    pub vector_store: VectorStoreConfigContainer,
}

/// Infrastructure configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfrastructureConfig {
    /// Cache system configuration
    pub cache: CacheSystemConfig,
    /// EventBus configuration
    pub event_bus: EventBusConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
    /// Resilience configuration
    pub resilience: ResilienceConfig,
    /// Limits configuration
    pub limits: LimitsConfig,
}

/// Data management configurations
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
    /// Operating mode configuration
    #[serde(default)]
    pub mode: ModeConfig,
    /// Server configuration
    pub server: ServerConfig,
    /// Provider configurations
    pub providers: ProvidersConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// System configurations
    pub system: SystemConfig,
    /// Operations and daemon configurations
    pub operations_daemon: OperationsDaemonConfig,
}
