//! Main application configuration

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export all config types
pub use super::{
    auth::{AuthConfig, JwtConfig, PasswordAlgorithm},
    backup::BackupConfig,
    cache::{CacheProvider, CacheSystemConfig},
    daemon::DaemonConfig,
    event_bus::{EventBusConfig, EventBusProvider},
    limits::LimitsConfig,
    logging::LoggingConfig,
    metrics::MetricsConfig,
    operations::OperationsConfig,
    resilience::ResilienceConfig,
    server::{
        ServerConfig, ServerCorsConfig, ServerNetworkConfig, ServerSslConfig, ServerTimeoutConfig,
        TransportMode,
    },
    snapshot::SnapshotConfig,
    sync::SyncConfig,
};

/// Embedding configuration container that supports both:
/// - Flat env vars: MCP__PROVIDERS__EMBEDDING__PROVIDER, MCP__PROVIDERS__EMBEDDING__MODEL
/// - TOML named configs: [providers.embedding.configs.default]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingConfigContainer {
    /// Provider name (from MCP__PROVIDERS__EMBEDDING__PROVIDER)
    pub provider: Option<String>,
    /// Model name (from MCP__PROVIDERS__EMBEDDING__MODEL)
    pub model: Option<String>,
    /// Base URL for API (from MCP__PROVIDERS__EMBEDDING__BASE_URL)
    pub base_url: Option<String>,
    /// API key (from MCP__PROVIDERS__EMBEDDING__API_KEY)
    pub api_key: Option<String>,
    /// Embedding dimensions (from MCP__PROVIDERS__EMBEDDING__DIMENSIONS)
    pub dimensions: Option<usize>,

    /// Named configs for TOML format (e.g., [providers.embedding.configs.default])
    #[serde(default)]
    pub configs: HashMap<String, EmbeddingConfig>,
}

/// Vector store configuration container that supports both:
/// - Flat env vars: MCP__PROVIDERS__VECTOR_STORE__PROVIDER, MCP__PROVIDERS__VECTOR_STORE__ADDRESS
/// - TOML named configs: [providers.vector_store.configs.default]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorStoreConfigContainer {
    /// Provider name (from MCP__PROVIDERS__VECTOR_STORE__PROVIDER)
    pub provider: Option<String>,
    /// Server address (from MCP__PROVIDERS__VECTOR_STORE__ADDRESS)
    pub address: Option<String>,
    /// Embedding dimensions (from MCP__PROVIDERS__VECTOR_STORE__DIMENSIONS)
    pub dimensions: Option<usize>,
    /// Collection name (from MCP__PROVIDERS__VECTOR_STORE__COLLECTION)
    pub collection: Option<String>,

    /// Named configs for TOML format (e.g., [providers.vector_store.configs.default])
    #[serde(default)]
    pub configs: HashMap<String, VectorStoreConfig>,
}

/// Provider configurations (embedding and vector store)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvidersConfig {
    /// Embedding provider configuration
    #[serde(default)]
    pub embedding: EmbeddingConfigContainer,

    /// Vector store provider configuration
    #[serde(default)]
    pub vector_store: VectorStoreConfigContainer,
}

/// Infrastructure configurations (cache, event_bus, metrics, resilience, limits)
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
