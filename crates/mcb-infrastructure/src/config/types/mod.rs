//! Configuration types module
//!
//! Consolidated configuration types organized by domain:
//! - `app` - Main application configuration and containers
//! - `server` - Server transport and network configuration
//! - `mode` - Operating mode configuration
//! - `infrastructure` - Logging, limits, cache, metrics, resilience
//! - `system` - Auth, event_bus, backup, sync, snapshot, daemon, operations

pub mod app;
pub mod infrastructure;
pub mod mode;
pub mod server;
pub mod system;

// Re-export main types
pub use app::*;
pub use infrastructure::{
    CacheProvider, CacheSystemConfig, LimitsConfig, LoggingConfig, MetricsConfig, ResilienceConfig,
};
pub use mode::{ModeConfig, OperatingMode};
pub use server::{
    ServerConfig, ServerCorsConfig, ServerNetworkConfig, ServerSslConfig, ServerTimeoutConfig,
    TransportMode,
};
pub use system::{
    AdminApiKeyConfig, ApiKeyConfig, AuthConfig, BackupConfig, DaemonConfig, EventBusConfig,
    EventBusProvider, JwtConfig, OperationsConfig, PasswordAlgorithm, SnapshotConfig, SyncConfig,
};
