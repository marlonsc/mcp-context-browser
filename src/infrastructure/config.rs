pub mod loader;
pub mod metrics;
pub mod providers;
pub mod server;
pub mod types;
pub mod watcher;

// Re-export types
pub use loader::ConfigLoader;
pub use metrics::MetricsConfig;
pub use providers::{EmbeddingProviderConfig, VectorStoreProviderConfig};
pub use server::ServerConfig;
pub use types::{Config, GlobalConfig, GlobalProviderConfig, ProviderConfig};
pub use watcher::ConfigWatcher;
