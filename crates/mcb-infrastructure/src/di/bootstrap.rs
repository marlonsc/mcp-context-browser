//! DI Container Bootstrap - Clean Architecture Composition Root
//!
//! Provides the composition root for the dependency injection system following
//! Clean Architecture principles with Shaku modules.
//!
//! ## Architecture (Shaku DI)
//!
//! All providers are registered as Shaku Components in their respective modules.
//! The bootstrap composes modules with their default Components (NullProviders).
//!
//! For production providers, use `with_component_override` when building modules
//! in the server layer (mcb-server/init.rs).
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create AppContainer with default providers (NullProviders for testing)
//! let app_container = init_app(AppConfig::default()).await?;
//!
//! // Resolve providers via Shaku DI
//! let embedding: Arc<dyn EmbeddingProvider> = app_container.embedding.resolve();
//! let vector_store: Arc<dyn VectorStoreProvider> = app_container.data.resolve();
//! ```

use crate::config::AppConfig;
use mcb_domain::error::Result;
use tracing::info;

// Import Shaku module implementations
use super::modules::{
    admin::AdminModuleImpl, cache_module::CacheModuleImpl, data_module::DataModuleImpl,
    embedding_module::EmbeddingModuleImpl, infrastructure::InfrastructureModuleImpl,
    language_module::LanguageModuleImpl, server::ServerModuleImpl,
};

/// Type alias for the root DI container (AppContainer with Shaku modules).
pub type DiContainer = AppContainer;

/// Container builder for Shaku-based DI system.
///
/// Builds the hierarchical module structure with null providers as defaults.
/// For production usage, create providers via factories and pass to DomainServicesFactory.
///
/// ## Example (Testing)
/// ```rust,ignore
/// let container = DiContainerBuilder::new().build().await?;
/// // Uses null providers from Shaku modules
/// ```
///
/// ## Example (Production)
/// See `mcb_server::init::run_server` for the complete production flow.
pub struct DiContainerBuilder {
    config: Option<AppConfig>,
}

impl DiContainerBuilder {
    /// Create a new container builder (null providers by default)
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Create a container builder with configuration
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    /// Build the DI container with hierarchical module composition
    pub async fn build(self) -> Result<DiContainer> {
        let config = self.config.unwrap_or_default();
        init_app(config).await
    }
}

impl Default for DiContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create DI container for testing
pub async fn create_test_container() -> Result<DiContainer> {
    DiContainerBuilder::new().build().await
}

// ============================================================================
// Clean Architecture App Module (Hierarchical Composition)
// ============================================================================

/// Application container using Clean Architecture modules
///
/// Contains only the essential modules following Clean Architecture:
/// - Context modules: cache, embedding, data, language (provider implementations)
/// - Infrastructure modules: infrastructure (cross-cutting), server (MCP), admin
pub struct AppContainer {
    /// Cache provider module (NullCacheProvider by default)
    pub cache: CacheModuleImpl,
    /// Embedding provider module (NullEmbeddingProvider by default)
    pub embedding: EmbeddingModuleImpl,
    /// Data/vector store provider module (NullVectorStoreProvider by default)
    pub data: DataModuleImpl,
    /// Language chunking module (UniversalLanguageChunkingProvider)
    pub language: LanguageModuleImpl,
    /// Core infrastructure services (auth, events, metrics, sync, snapshot)
    pub infrastructure: InfrastructureModuleImpl,
    /// MCP server components (performance metrics, indexing operations)
    pub server: ServerModuleImpl,
    /// Admin services (performance metrics, shutdown coordination)
    pub admin: AdminModuleImpl,
}

/// Initialize the application with Shaku DI modules
///
/// All providers are resolved via Shaku - uses default Components (NullProviders).
/// For production providers, build modules with `with_component_override` in mcb-server.
///
/// ## Example
///
/// ```ignore
/// // Uses default NullProviders
/// let container = init_app(AppConfig::default()).await?;
/// ```
pub async fn init_app(_config: AppConfig) -> Result<AppContainer> {
    info!("Initializing Shaku DI modules");

    // Build all modules with Shaku - uses default Components (NullProviders)
    let cache = CacheModuleImpl::builder().build();
    let embedding = EmbeddingModuleImpl::builder().build();
    let data = DataModuleImpl::builder().build();
    let language = LanguageModuleImpl::builder().build();
    let infrastructure = InfrastructureModuleImpl::builder().build();
    let server = ServerModuleImpl::builder().build();
    let admin = AdminModuleImpl::builder().build();

    let app_container = AppContainer {
        cache,
        embedding,
        data,
        language,
        infrastructure,
        server,
        admin,
    };

    info!("Shaku DI modules initialized");
    Ok(app_container)
}
