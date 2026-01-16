//! DI Container Bootstrap
//!
//! Provides the main dependency injection container that wires together
//! all infrastructure components following Clean Architecture principles.

use crate::adapters::providers::embedding::NullEmbeddingProvider;
use crate::adapters::providers::vector_store::InMemoryVectorStoreProvider;
use crate::cache::provider::SharedCacheProvider;
use crate::cache::CacheProviderFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::{checkers, HealthRegistry};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use std::sync::Arc;

/// Core infrastructure components container
#[derive(Clone)]
pub struct InfrastructureComponents {
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
    pub health: HealthRegistry,
    pub config: AppConfig,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl InfrastructureComponents {
    /// Create a new infrastructure components container
    pub async fn new(config: AppConfig) -> Result<Self> {
        // Create cache provider based on configuration
        let cache = if config.cache.enabled {
            CacheProviderFactory::create_from_config(&config.cache).await?
        } else {
            CacheProviderFactory::create_null()
        };

        // Create crypto service with master key from config or generate one
        // AES-GCM requires exactly 32 bytes for the key
        let master_key = if config.auth.jwt_secret.len() >= 32 {
            // Use first 32 bytes of the JWT secret as the master key
            config.auth.jwt_secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        };

        let crypto = CryptoService::new(master_key)?;

        // Create health registry and register built-in checkers
        let health = HealthRegistry::new();

        // Register system health checker
        let system_checker = checkers::SystemHealthChecker::new();
        health
            .register_checker("system".to_string(), system_checker)
            .await;

        // Register database health checker if configured
        // (This would be expanded based on actual database configuration)

        // Create embedding provider based on configuration
        // For now, use NullEmbeddingProvider as default (can be enhanced to
        // create different providers based on config.embedding.provider)
        let embedding_provider: Arc<dyn EmbeddingProvider> =
            Arc::new(NullEmbeddingProvider::new());

        // Create vector store provider based on configuration
        // For now, use InMemoryVectorStoreProvider as default (can be enhanced to
        // create different providers based on config.vector_store.provider)
        let vector_store_provider: Arc<dyn VectorStoreProvider> =
            Arc::new(InMemoryVectorStoreProvider::new());

        Ok(Self {
            cache,
            crypto,
            health,
            config,
            embedding_provider,
            vector_store_provider,
        })
    }

    /// Get the cache provider
    pub fn cache(&self) -> &SharedCacheProvider {
        &self.cache
    }

    /// Get the crypto service
    pub fn crypto(&self) -> &CryptoService {
        &self.crypto
    }

    /// Get the health registry
    pub fn health(&self) -> &HealthRegistry {
        &self.health
    }

    /// Get the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get the embedding provider
    pub fn embedding_provider(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.embedding_provider
    }

    /// Get the vector store provider
    pub fn vector_store_provider(&self) -> &Arc<dyn VectorStoreProvider> {
        &self.vector_store_provider
    }
}

/// Container builder for infrastructure components
pub struct InfrastructureContainerBuilder {
    config: AppConfig,
}

impl InfrastructureContainerBuilder {
    /// Create a new container builder
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build the infrastructure container
    pub async fn build(self) -> Result<InfrastructureComponents> {
        InfrastructureComponents::new(self.config).await
    }
}

/// Helper function to create infrastructure components
pub async fn create_infrastructure_components(
    config: AppConfig,
) -> Result<InfrastructureComponents> {
    InfrastructureContainerBuilder::new(config).build().await
}

/// Combined container with both infrastructure and domain services
#[derive(Clone)]
pub struct FullContainer {
    pub infrastructure: InfrastructureComponents,
    pub domain_services: super::modules::DomainServicesContainer,
}

impl FullContainer {
    /// Create a full container with both infrastructure and domain services
    pub async fn new(config: AppConfig) -> Result<Self> {
        let infrastructure = InfrastructureContainerBuilder::new(config.clone())
            .build()
            .await?;
        let domain_services = super::modules::DomainServicesFactory::create_services(
            infrastructure.cache.clone(),
            infrastructure.crypto.clone(),
            infrastructure.health.clone(),
            config,
            infrastructure.embedding_provider.clone(),
            infrastructure.vector_store_provider.clone(),
        )
        .await?;

        Ok(Self {
            infrastructure,
            domain_services,
        })
    }

    /// Get indexing service
    pub fn indexing_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::IndexingServiceInterface> {
        self.domain_services.indexing_service.clone()
    }

    /// Get context service
    pub fn context_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::ContextServiceInterface> {
        self.domain_services.context_service.clone()
    }

    /// Get search service
    pub fn search_service(
        &self,
    ) -> Arc<dyn mcb_domain::domain_services::search::SearchServiceInterface> {
        self.domain_services.search_service.clone()
    }
}
