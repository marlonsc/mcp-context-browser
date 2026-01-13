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
    AdaptersModule, AdaptersModuleImpl, InfrastructureModule,
    InfrastructureModuleImpl, ServerModule, ServerModuleImpl,
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
pub struct DiContainer {
    /// Adapters module (HTTP clients, providers, repositories)
    pub adapters_module: Arc<dyn AdaptersModule>,
    /// Infrastructure module (metrics, service provider, events, auth)
    pub infrastructure_module: Arc<dyn InfrastructureModule>,
    /// Server module (performance, indexing operations)
    pub server_module: Arc<dyn ServerModule>,
}

impl DiContainer {
    /// Build the DI container with null providers (for testing)
    ///
    /// This builds with default null providers, suitable for unit tests.
    /// For production, use `build_with_config()` instead.
    pub fn build() -> Result<Self> {
        // Build leaf modules with null providers (defaults)
        let adapters_module: Arc<dyn AdaptersModule> =
            Arc::new(AdaptersModuleImpl::builder().build());
        let infrastructure_module: Arc<dyn InfrastructureModule> =
            Arc::new(InfrastructureModuleImpl::builder().build());
        let server_module: Arc<dyn ServerModule> =
            Arc::new(ServerModuleImpl::builder().build());

        Ok(Self {
            adapters_module,
            infrastructure_module,
            server_module,
        })
    }

    /// Build the DI container with config-based providers (for production)
    ///
    /// Creates providers based on configuration and injects them into the
    /// module hierarchy using Shaku's component override mechanism.
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
        let embedding_provider = create_embedding_provider_boxed(
            &config.providers.embedding,
            Arc::clone(&http_client),
        ).await?;

        let vector_store_provider = create_vector_store_provider_boxed(
            &config.providers.vector_store,
        ).await?;

        // Build adapters module with config-based provider overrides
        let adapters_module: Arc<dyn AdaptersModule> = Arc::new(
            AdaptersModuleImpl::builder()
                .with_component_override::<dyn EmbeddingProvider>(embedding_provider)
                .with_component_override::<dyn VectorStoreProvider>(vector_store_provider)
                .build()
        );

        // Build other modules normally
        let infrastructure_module: Arc<dyn InfrastructureModule> =
            Arc::new(InfrastructureModuleImpl::builder().build());
        let server_module: Arc<dyn ServerModule> =
            Arc::new(ServerModuleImpl::builder().build());

        Ok(Self {
            adapters_module,
            infrastructure_module,
            server_module,
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
    fn resolve(&self) -> Arc<dyn crate::infrastructure::metrics::system::SystemMetricsCollectorInterface> {
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

impl ComponentResolver<dyn crate::server::metrics::PerformanceMetricsInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::server::metrics::PerformanceMetricsInterface> {
        self.server_module.resolve()
    }
}

impl ComponentResolver<dyn crate::server::operations::IndexingOperationsInterface> for DiContainer {
    fn resolve(&self) -> Arc<dyn crate::server::operations::IndexingOperationsInterface> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::http_client::HttpClientProvider;
    use crate::infrastructure::di::factory::ServiceProviderInterface;
    use crate::infrastructure::events::EventBusProvider;
    use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
    use crate::server::metrics::PerformanceMetricsInterface;
    use crate::server::operations::IndexingOperationsInterface;

    #[tokio::test]
    async fn test_di_container_resolves_http_client() {
        let container = DiContainer::build().expect("DiContainer should build");
        let http_client: Arc<dyn HttpClientProvider> = container.resolve();
        assert!(Arc::strong_count(&http_client) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_performance_metrics() {
        let container = DiContainer::build().expect("DiContainer should build");
        let metrics: Arc<dyn PerformanceMetricsInterface> = container.resolve();
        assert!(Arc::strong_count(&metrics) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_indexing_operations() {
        let container = DiContainer::build().expect("DiContainer should build");
        let ops: Arc<dyn IndexingOperationsInterface> = container.resolve();
        assert!(Arc::strong_count(&ops) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_service_provider() {
        let container = DiContainer::build().expect("DiContainer should build");
        let provider: Arc<dyn ServiceProviderInterface> = container.resolve();
        assert!(Arc::strong_count(&provider) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_system_collector() {
        let container = DiContainer::build().expect("DiContainer should build");
        let collector: Arc<dyn SystemMetricsCollectorInterface> = container.resolve();
        assert!(Arc::strong_count(&collector) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_event_bus() {
        let container = DiContainer::build().expect("DiContainer should build");
        let event_bus: Arc<dyn EventBusProvider> = container.resolve();
        assert!(Arc::strong_count(&event_bus) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_auth_service() {
        use crate::infrastructure::auth::AuthServiceInterface;
        let container = DiContainer::build().expect("DiContainer should build");
        let auth_service: Arc<dyn AuthServiceInterface> = container.resolve();
        assert!(Arc::strong_count(&auth_service) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_embedding_provider() {
        use crate::domain::ports::EmbeddingProvider;
        let container = DiContainer::build().expect("DiContainer should build");
        let provider: Arc<dyn EmbeddingProvider> = container.resolve();
        assert!(Arc::strong_count(&provider) >= 1);
        // Verify we got a null provider
        assert_eq!(provider.provider_name(), "null");
    }

    #[tokio::test]
    async fn test_di_container_resolves_vector_store_provider() {
        use crate::domain::ports::VectorStoreProvider;
        let container = DiContainer::build().expect("DiContainer should build");
        let provider: Arc<dyn VectorStoreProvider> = container.resolve();
        assert!(Arc::strong_count(&provider) >= 1);
        // Verify we got a null provider
        assert_eq!(provider.provider_name(), "null");
    }

    #[tokio::test]
    async fn test_di_container_resolves_chunk_repository() {
        use crate::domain::ports::ChunkRepository;
        let container = DiContainer::build().expect("DiContainer should build");
        let repository: Arc<dyn ChunkRepository> = container.resolve();
        assert!(Arc::strong_count(&repository) >= 1);
    }

    #[tokio::test]
    async fn test_di_container_resolves_search_repository() {
        use crate::domain::ports::SearchRepository;
        let container = DiContainer::build().expect("DiContainer should build");
        let repository: Arc<dyn SearchRepository> = container.resolve();
        assert!(Arc::strong_count(&repository) >= 1);
    }

    // NOTE: AdminService resolution requires complex runtime dependencies
    // which is not yet complete. This will be enabled in Phase 3.
    // #[tokio::test]
    // async fn test_di_container_resolves_admin_service() {
    //     let container = DiContainer::build().expect("DiContainer should build");
    //     let admin_service: Arc<dyn AdminService> = container.resolve();
    //     assert!(Arc::strong_count(&admin_service) >= 1);
    // }
}
