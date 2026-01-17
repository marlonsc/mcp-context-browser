//! DI Container Bootstrap - Shaku Strict Pattern
//!
//! Provides the composition root for the Shaku-based dependency injection system.
//! This follows the strict Shaku hierarchical module pattern with no manual wiring.
//!
//! ## Shaku Strict Architecture
//!
//! - **Module Hierarchy**: Uses `module!` macro with `use dyn ModuleTrait` for composition
//! - **No Manual Wiring**: All dependencies resolved through module interfaces
//! - **Provider Overrides**: Runtime component overrides for production configuration
//! - **Trait-based DI**: All dependencies injected as `Arc<dyn Trait>`
//!
//! ## Construction Pattern
//!
//! ```rust,ignore
//! // Build all modules (simplified - no dependencies between them)
//! let infrastructure = Arc::new(InfrastructureModuleImpl::builder().build());
//! let server = Arc::new(ServerModuleImpl::builder().build());
//! let adapters = Arc::new(AdaptersModuleImpl::builder().build());
//! let application = Arc::new(ApplicationModuleImpl::builder().build());
//! let admin = Arc::new(AdminModuleImpl::builder().build());
//!
//! // Build root container
//! let container = McpModule::builder(infrastructure, server, adapters, application, admin).build();
//! ```
//!
//! ## Note
//!
//! Most services are created at runtime via DomainServicesFactory (in domain_services.rs)
//! rather than through Shaku DI, because they require runtime configuration.
//! The Shaku modules primarily hold null providers as defaults.

use crate::config::AppConfig;
use crate::crypto::CryptoService;
use mcb_domain::error::Result;
use std::sync::Arc;
use tracing::info;

// Import all module implementations and traits
use super::modules::{
    AdaptersModuleImpl, AdminModuleImpl, ApplicationModuleImpl, InfrastructureModuleImpl,
    McpModule, ServerModuleImpl,
    traits::{AdaptersModule, AdminModule, ApplicationModule, InfrastructureModule, ServerModule},
};

// Import factories for provider overrides (production configuration)
use super::factory::{EmbeddingProviderFactory, VectorStoreProviderFactory};

/// Shaku-based DI Container following strict hierarchical pattern.
///
/// This container holds the root McpModule and provides access to all services
/// through the module resolution system. No manual component management.
///
/// ## Usage
///
/// ```rust,ignore
/// // Create container with production config
/// let container = DiContainer::build_with_config(config, http_client).await?;
///
/// // Resolve provider through trait-based access
/// let embedding_provider: Arc<dyn EmbeddingProvider> = container.resolve();
/// ```
pub type DiContainer = McpModule;

/// Provider configuration overrides for production setup.
///
/// This struct provides methods to create configured providers
/// that can be injected into the module hierarchy at runtime.
pub struct ProviderOverrides;

impl ProviderOverrides {
    /// Create embedding provider from configuration
    pub fn create_embedding_provider(config: &AppConfig) -> Result<Arc<dyn mcb_domain::ports::providers::EmbeddingProvider>> {
        if let Some((name, embedding_config)) = config.providers.embedding.iter().next() {
            info!(provider = name, "Creating embedding provider from config");
            EmbeddingProviderFactory::create(embedding_config, None)
        } else {
            info!("No embedding provider configured, using null provider");
            Ok(EmbeddingProviderFactory::create_null())
        }
    }

    /// Create vector store provider from configuration
    pub fn create_vector_store_provider(
        config: &AppConfig,
        crypto: &CryptoService,
    ) -> Result<Arc<dyn mcb_domain::ports::providers::VectorStoreProvider>> {
        if let Some((name, vector_config)) = config.providers.vector_store.iter().next() {
            info!(
                provider = name,
                "Creating vector store provider from config"
            );
            // Wrap CryptoService as Arc<dyn CryptoProvider> for DI
            let crypto_provider: Arc<dyn mcb_domain::ports::providers::CryptoProvider> = Arc::new(crypto.clone());
            VectorStoreProviderFactory::create(vector_config, Some(crypto_provider))
        } else {
            info!("No vector store provider configured, using in-memory provider");
            Ok(VectorStoreProviderFactory::create_in_memory())
        }
    }
}

/// Container builder for Shaku-based DI system.
///
/// Builds the hierarchical module structure following the strict Shaku pattern.
/// Provides both testing (null providers) and production (configured providers) setups.
pub struct DiContainerBuilder {
    #[allow(dead_code)]
    config: Option<AppConfig>,
    #[allow(dead_code)]
    embedding_override: Option<Arc<dyn mcb_domain::ports::providers::EmbeddingProvider>>,
    #[allow(dead_code)]
    vector_store_override: Option<Arc<dyn mcb_domain::ports::providers::VectorStoreProvider>>,
}

impl DiContainerBuilder {
    /// Create a new container builder for testing (null providers)
    pub fn new() -> Self {
        Self {
            config: None,
            embedding_override: None,
            vector_store_override: None,
        }
    }

    /// Create a container builder with production configuration
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config: Some(config),
            embedding_override: None,
            vector_store_override: None,
        }
    }

    /// Override the embedding provider (for production configuration)
    pub fn with_embedding_provider(
        mut self,
        provider: Arc<dyn mcb_domain::ports::providers::EmbeddingProvider>,
    ) -> Self {
        self.embedding_override = Some(provider);
        self
    }

    /// Override the vector store provider (for production configuration)
    pub fn with_vector_store_provider(
        mut self,
        provider: Arc<dyn mcb_domain::ports::providers::VectorStoreProvider>,
    ) -> Self {
        self.vector_store_override = Some(provider);
        self
    }

    /// Build the DI container with hierarchical module composition
    ///
    /// Note: Provider overrides are stored but not used by Shaku modules directly.
    /// Instead, use `ProviderOverrides` methods and pass providers to `DomainServicesFactory`
    /// at runtime for actual service creation.
    ///
    /// ApplicationModule and AdminModule are placeholders - their services are created
    /// at runtime via DomainServicesFactory, not through Shaku resolution.
    pub async fn build(self) -> Result<DiContainer> {
        // Build leaf modules (no dependencies)
        // Note: Shaku modules use null providers as defaults.
        // Real providers are created via DomainServicesFactory at runtime.
        let infrastructure: Arc<dyn InfrastructureModule> =
            Arc::new(InfrastructureModuleImpl::builder().build());
        let server: Arc<dyn ServerModule> = Arc::new(ServerModuleImpl::builder().build());
        let adapters: Arc<dyn AdaptersModule> = Arc::new(AdaptersModuleImpl::builder().build());

        // Build root container with core modules only
        // ApplicationModule and AdminModule are NOT included - they're runtime-only
        Ok(McpModule::builder(infrastructure, server, adapters).build())
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

/// Convenience function to create DI container with production configuration
pub async fn create_production_container(config: AppConfig) -> Result<DiContainer> {
    // Create configured providers
    let embedding_provider = ProviderOverrides::create_embedding_provider(&config)?;
    let crypto = CryptoService::new(
        if config.auth.jwt.secret.len() >= 32 {
            config.auth.jwt.secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        }
    )?;
    let vector_store_provider = ProviderOverrides::create_vector_store_provider(&config, &crypto)?;

    DiContainerBuilder::with_config(config)
        .with_embedding_provider(embedding_provider)
        .with_vector_store_provider(vector_store_provider)
        .build()
        .await
}

/// Legacy compatibility - creates full container for gradual migration
///
/// **DEPRECATED**: Use `create_production_container()` instead.
/// This will be removed in v0.2.0 when migration to strict Shaku pattern is complete.
#[deprecated(
    since = "0.1.0",
    note = "Use `create_production_container()` instead. This function will be removed in v0.2.0."
)]
pub async fn create_full_container(config: AppConfig) -> Result<DiContainer> {
    create_production_container(config).await
}

// ============================================================================
// Legacy Compatibility Types (for gradual migration)
// ============================================================================

/// Infrastructure components container for backward compatibility
///
/// **Note**: This exists for gradual migration from the old API.
/// New code should use `create_production_container()` and Shaku modules.
#[derive(Clone)]
pub struct InfrastructureComponents {
    /// Shared cache provider
    pub cache: crate::cache::provider::SharedCacheProvider,
    /// Crypto service
    pub crypto: CryptoService,
    /// Health registry
    pub health: crate::health::HealthRegistry,
}

impl InfrastructureComponents {
    /// Create infrastructure components from configuration
    pub async fn new(config: AppConfig) -> Result<Self> {
        InfrastructureContainerBuilder::new(config).build().await
    }
}

/// Builder for infrastructure components (backward compatibility)
///
/// **Note**: This exists for gradual migration from the old API.
pub struct InfrastructureContainerBuilder {
    config: AppConfig,
}

impl InfrastructureContainerBuilder {
    /// Create a new builder
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build infrastructure components
    pub async fn build(self) -> Result<InfrastructureComponents> {
        // Create cache provider
        let cache = crate::cache::factory::CacheProviderFactory::create_from_config(
            &self.config.system.infrastructure.cache,
        )
        .await?;

        // Create crypto service with 32-byte key
        let master_key = if self.config.auth.jwt.secret.len() >= 32 {
            self.config.auth.jwt.secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        };
        let crypto = CryptoService::new(master_key)?;

        // Create health registry
        let health = crate::health::HealthRegistry::new();
        let system_checker = crate::health::checkers::SystemHealthChecker::new();
        health.register_checker("system".to_string(), system_checker).await;

        Ok(InfrastructureComponents {
            cache,
            crypto,
            health,
        })
    }
}

// ============================================================================
// Legacy FullContainer (for mcb-server backward compatibility)
// ============================================================================

use super::modules::DomainServicesContainer;
use super::modules::DomainServicesFactory;
use mcb_domain::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};

/// Full container for backward compatibility with mcb-server.
///
/// **Note**: This exists for gradual migration from the old API.
/// It combines InfrastructureComponents with DomainServicesContainer.
#[derive(Clone)]
pub struct FullContainer {
    /// Infrastructure components
    pub infrastructure: InfrastructureComponents,
    /// Domain services
    pub services: DomainServicesContainer,
}

impl FullContainer {
    /// Create a full container from configuration
    pub async fn new(config: AppConfig) -> Result<Self> {
        // Create infrastructure components
        let infrastructure = InfrastructureComponents::new(config.clone()).await?;

        // Create embedding and vector store providers
        let embedding_provider = ProviderOverrides::create_embedding_provider(&config)?;
        let vector_store_provider = ProviderOverrides::create_vector_store_provider(
            &config,
            &infrastructure.crypto,
        )?;

        // Create domain services
        let services = DomainServicesFactory::create_services(
            infrastructure.cache.clone(),
            infrastructure.crypto.clone(),
            config,
            embedding_provider,
            vector_store_provider,
        )
        .await?;

        Ok(Self {
            infrastructure,
            services,
        })
    }

    /// Get the indexing service
    pub fn indexing_service(&self) -> Arc<dyn IndexingServiceInterface> {
        self.services.indexing_service.clone()
    }

    /// Get the context service
    pub fn context_service(&self) -> Arc<dyn ContextServiceInterface> {
        self.services.context_service.clone()
    }

    /// Get the search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        self.services.search_service.clone()
    }
}
