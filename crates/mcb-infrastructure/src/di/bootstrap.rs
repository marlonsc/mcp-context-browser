//! DI Container Bootstrap - Clean Architecture Composition Root
//!
//! Provides the composition root for the dependency injection system following
//! Clean Architecture principles with Shaku modules for infrastructure services.
//!
//! ## Architecture Strategy (see ADR-010)
//!
//! The DI system uses a two-layer approach:
//!
//! 1. **Shaku Modules** (Infrastructure Layer): Provide null implementations as defaults
//!    for infrastructure services (auth, events, metrics, sync, snapshot).
//!    These are resolved via trait-based DI at compile time.
//!
//! 2. **Runtime Factory** (Application Layer): Creates domain services with production
//!    providers configured from `AppConfig`. This is necessary because:
//!    - Provider implementations require runtime configuration (API keys, URLs)
//!    - Async initialization patterns don't fit Shaku's static composition
//!    - Different providers need different construction parameters
//!
//! ## Production Flow
//!
//! ```rust,ignore
//! // 1. Create AppContainer with Shaku modules (null providers as defaults)
//! let app_container = init_app(config).await?;
//!
//! // 2. Create production providers from config
//! let embedding = EmbeddingProviderFactory::create(&config.embedding, None)?;
//! let vector_store = VectorStoreProviderFactory::create(&config.vector_store, crypto)?;
//!
//! // 3. Create domain services with production providers
//! let services = DomainServicesFactory::create_services(
//!     cache, crypto, config, embedding, vector_store, chunker
//! ).await?;
//! ```
//!
//! ## Testing Flow
//!
//! ```rust,ignore
//! // Use null providers from Shaku modules
//! let app_container = init_app(AppConfig::default()).await?;
//! let services = DomainServicesFactory::create_context_service(&app_container).await?;
//! ```

use crate::config::AppConfig;
use mcb_domain::error::Result;
use tracing::info;

// Import module implementations (Clean Architecture - no empty placeholder modules)
use super::modules::{
    admin::AdminModuleImpl, cache_module::CacheModuleImpl, data_module::DataModuleImpl,
    embedding_module::EmbeddingModuleImpl, infrastructure::InfrastructureModuleImpl,
    language_module::LanguageModuleImpl, server::ServerModuleImpl,
};

// Re-export factories for production provider creation (used by mcb-server)
pub use super::factory::{EmbeddingProviderFactory, VectorStoreProviderFactory};

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

/// Initialize the application using Clean Architecture modules
///
/// This replaces the old FullContainer approach with pure Shaku DI.
/// Uses hierarchical modules following the Clean Architecture pattern.
pub async fn init_app(_config: AppConfig) -> Result<AppContainer> {
    info!("Initializing Clean Architecture application modules");

    // Build context modules (provider implementations)
    let cache = CacheModuleImpl::builder().build();
    let embedding = EmbeddingModuleImpl::builder().build();
    let data = DataModuleImpl::builder().build();
    let language = LanguageModuleImpl::builder().build();

    // Build infrastructure modules
    let infrastructure = InfrastructureModuleImpl::builder().build();
    let server = ServerModuleImpl::builder().build();
    let admin = AdminModuleImpl::builder().build();

    // Compose into final app container
    let app_container = AppContainer {
        cache,
        embedding,
        data,
        language,
        infrastructure,
        server,
        admin,
    };

    info!("Clean Architecture application initialized successfully");
    Ok(app_container)
}
