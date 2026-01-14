//! DI Container Bootstrap
//!
//! This module provides the bootstrap function for creating the complete
//! Shaku module hierarchy. Use this instead of manual component construction.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use mcp_context_browser::infrastructure::di::bootstrap::DiContainer;
//!
//! // Default build (null providers - for tests)
//! let container = DiContainer::build()?;
//!
//! // Build with config (production - uses actual providers)
//! let container = DiContainer::build_with_config(&config, http_client).await?;
//!
//! // Resolve components from the container
//! let http_client: Arc<dyn HttpClientProvider> = container.resolve();
//! let chunk_repository: Arc<dyn ChunkRepository> = container.resolve();
//! ```

use std::sync::Arc;

use crate::adapters::http_client::HttpClientProvider;
use crate::domain::error::Result;
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::config::Config;
use shaku::Interface;

use super::dispatch::{create_embedding_provider_boxed, create_vector_store_provider_boxed};
use super::modules::{
    AdaptersModule, AdaptersModuleImpl, AdminModule, AdminModuleImpl, ApplicationModule,
    ApplicationModuleImpl, InfrastructureModule, InfrastructureModuleImpl, ServerModule,
    ServerModuleImpl,
};

/// DI Container holding the module hierarchy
///
/// This container builds and holds Shaku modules, providing a single
/// point for resolving registered components.
///
/// ## Provider Strategy
///
/// By default, null providers are used (for testing). In production,
/// use `build_with_config()` to inject actual providers based on config.
///
/// ## Future Modules
///
/// Analysis, Quality, and Git modules are prepared for v0.3.0+ feature additions.
/// They are feature-gated and optional - initialized only when enabled.
pub struct DiContainer {
    /// Adapters module (HTTP clients, providers, repositories)
    pub adapters_module: Arc<dyn AdaptersModule>,
    /// Infrastructure module (metrics, service provider, events, auth)
    pub infrastructure_module: Arc<dyn InfrastructureModule>,
    /// Server module (performance, indexing operations)
    pub server_module: Arc<dyn ServerModule>,
    /// Admin module (admin service with dependencies on all modules)
    pub admin_module: Arc<dyn AdminModule>,
    /// Application module (business logic services)
    pub application_module: Arc<dyn ApplicationModule>,
}

impl DiContainer {
    /// Build the DI container with null providers (for testing)
    ///
    /// This builds with default null providers, suitable for unit tests.
    /// For production, use `build_with_config()` instead.
    ///
    /// Future modules (analysis, quality, git) are initialized as None in v0.2.0.
    /// They will be populated when those features are implemented (v0.3.0+).
    pub fn build() -> Result<Self> {
        // Build leaf modules with null providers (defaults)
        let adapters_module: Arc<dyn AdaptersModule> =
            Arc::new(AdaptersModuleImpl::builder().build());
        let infrastructure_module: Arc<dyn InfrastructureModule> =
            Arc::new(InfrastructureModuleImpl::builder().build());
        let server_module: Arc<dyn ServerModule> = Arc::new(ServerModuleImpl::builder().build());
        // Application module depends on adapters module for repositories
        let application_module: Arc<dyn ApplicationModule> =
            Arc::new(ApplicationModuleImpl::builder(Arc::clone(&adapters_module)).build());
        // Admin module depends on infrastructure, server, adapters, and application modules
        let admin_module: Arc<dyn AdminModule> = Arc::new(
            AdminModuleImpl::builder(
                Arc::clone(&infrastructure_module),
                Arc::clone(&server_module),
                Arc::clone(&adapters_module),
                Arc::clone(&application_module),
            )
            .build(),
        );

        Ok(Self {
            adapters_module,
            infrastructure_module,
            server_module,
            admin_module,
            application_module,
        })
    }

    /// Build the DI container with config-based providers (for production)
    ///
    /// Creates providers based on configuration and injects them into the
    /// module hierarchy using Shaku's component override mechanism.
    ///
    /// Future modules (analysis, quality, git) are initialized as None in v0.2.0.
    /// They will be populated from config when those features are implemented (v0.3.0+).
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration with provider settings
    /// * `http_client` - HTTP client for providers that need network access
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = Config::load()?;
    /// let http_client: Arc<dyn HttpClientProvider> = /* ... */;
    /// let container = DiContainer::build_with_config(&config, http_client).await?;
    /// ```
    pub async fn build_with_config(
        config: &Config,
        http_client: Arc<dyn HttpClientProvider>,
    ) -> Result<Self> {
        // Create providers based on config (returns Box for Shaku override)
        let embedding_provider =
            create_embedding_provider_boxed(&config.providers.embedding, Arc::clone(&http_client))
                .await?;

        let vector_store_provider =
            create_vector_store_provider_boxed(&config.providers.vector_store).await?;

        // Build adapters module with config-based provider overrides
        let adapters_module: Arc<dyn AdaptersModule> = Arc::new(
            AdaptersModuleImpl::builder()
                .with_component_override::<dyn EmbeddingProvider>(embedding_provider)
                .with_component_override::<dyn VectorStoreProvider>(vector_store_provider)
                .build(),
        );

        // Build other modules normally
        let infrastructure_module: Arc<dyn InfrastructureModule> =
            Arc::new(InfrastructureModuleImpl::builder().build());
        let server_module: Arc<dyn ServerModule> = Arc::new(ServerModuleImpl::builder().build());
        // Application module depends on adapters module for repositories
        let application_module: Arc<dyn ApplicationModule> =
            Arc::new(ApplicationModuleImpl::builder(Arc::clone(&adapters_module)).build());
        // Admin module depends on infrastructure, server, adapters, and application modules
        let admin_module: Arc<dyn AdminModule> = Arc::new(
            AdminModuleImpl::builder(
                Arc::clone(&infrastructure_module),
                Arc::clone(&server_module),
                Arc::clone(&adapters_module),
                Arc::clone(&application_module),
            )
            .build(),
        );

        Ok(Self {
            adapters_module,
            infrastructure_module,
            server_module,
            admin_module,
            application_module,
        })
    }

    /// Convenience method to resolve any component
    ///
    /// Uses trait bounds to automatically select the right module.
    pub fn resolve<T: Interface + ?Sized + 'static>(&self) -> Arc<T>
    where
        Self: ComponentResolver<T>,
    {
        <Self as ComponentResolver<T>>::resolve(self)
    }
}

/// Trait for resolving components from the right module
pub trait ComponentResolver<T: Interface + ?Sized> {
    /// Resolve and return an instance of the component
    fn resolve(&self) -> Arc<T>;
}

// Implement resolvers for each component type
impl ComponentResolver<dyn crate::adapters::http_client::HttpClientProvider> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::adapters::http_client::HttpClientProvider> {
        self.adapters_module.resolve()
    }
}

impl ComponentResolver<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface>
    for DiContainer
{
    fn resolve(
        &self,
    ) -> Arc<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface> {
        self.infrastructure_module.resolve()
    }
}

impl ComponentResolver<dyn crate::infrastructure::di::factory::ServiceProviderInterface>
    for DiContainer
{
    fn resolve(&self) -> Arc<dyn crate::infrastructure::di::factory::ServiceProviderInterface> {
        self.infrastructure_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::PerformanceMetricsInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::PerformanceMetricsInterface> {
        self.server_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::IndexingOperationsInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::IndexingOperationsInterface> {
        self.server_module.resolve()
    }
}

impl ComponentResolver<dyn crate::infrastructure::events::EventBusProvider> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::infrastructure::events::EventBusProvider> {
        self.infrastructure_module.resolve()
    }
}

impl ComponentResolver<dyn crate::infrastructure::auth::AuthServiceInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::infrastructure::auth::AuthServiceInterface> {
        self.infrastructure_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::EmbeddingProvider> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::EmbeddingProvider> {
        self.adapters_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::VectorStoreProvider> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::VectorStoreProvider> {
        self.adapters_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::ChunkRepository> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::ChunkRepository> {
        self.adapters_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::SearchRepository> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::SearchRepository> {
        self.adapters_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::ContextServiceInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::ContextServiceInterface> {
        self.application_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::SearchServiceInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::SearchServiceInterface> {
        self.application_module.resolve()
    }
}

impl ComponentResolver<dyn crate::domain::ports::IndexingServiceInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::domain::ports::IndexingServiceInterface> {
        self.application_module.resolve()
    }
}

impl ComponentResolver<dyn crate::server::admin::service::AdminService> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::server::admin::service::AdminService> {
        self.admin_module.resolve()
    }
}
