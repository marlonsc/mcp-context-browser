//! DI Container Bootstrap
//!
//! Provides the main dependency injection container that wires together
//! all infrastructure components following Clean Architecture principles.

use crate::cache::provider::SharedCacheProvider;
use crate::cache::CacheProviderFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::{HealthRegistry, checkers};
use mcb_domain::error::Result;
use std::sync::Arc;

/// Core infrastructure components container
#[derive(Clone)]
pub struct InfrastructureComponents {
    pub cache: SharedCacheProvider,
    pub crypto: CryptoService,
    pub health: HealthRegistry,
    pub config: AppConfig,
}

impl InfrastructureComponents {
    /// Create a new infrastructure components container
    pub async fn new(config: AppConfig) -> Result<Self> {
        // Create cache provider based on configuration
        let cache = if config.cache.enabled {
            CacheProviderFactory::create_from_config(&config.cache)?
        } else {
            CacheProviderFactory::create_null()
        };

        // Create crypto service with master key from config or generate one
        let master_key = if let Some(key) = &config.auth.jwt_secret {
            if key.len() >= 32 {
                key.clone().into_bytes()
            } else {
                CryptoService::generate_master_key()
            }
        } else {
            CryptoService::generate_master_key()
        };

        let crypto = CryptoService::new(master_key)?;

        // Create health registry and register built-in checkers
        let health = HealthRegistry::new();

        // Register system health checker
        let system_checker = checkers::SystemHealthChecker::new();
        health.register_checker("system".to_string(), system_checker).await;

        // Register database health checker if configured
        // (This would be expanded based on actual database configuration)

        Ok(Self {
            cache,
            crypto,
            health,
            config,
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
pub async fn create_infrastructure_components(config: AppConfig) -> Result<InfrastructureComponents> {
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
        let infrastructure = InfrastructureContainerBuilder::new(config.clone()).build().await?;
        let domain_services = super::modules::DomainServicesFactory::create_services(
            infrastructure.cache.clone(),
            infrastructure.crypto.clone(),
            infrastructure.health.clone(),
            config,
        ).await?;

        Ok(Self {
            infrastructure,
            domain_services,
        })
    }

    /// Get indexing service
    pub fn indexing_service(&self) -> Arc<dyn mcb_domain::domain_services::search::IndexingServiceInterface> {
        self.domain_services.indexing_service.clone()
    }

    /// Get context service
    pub fn context_service(&self) -> Arc<dyn mcb_domain::domain_services::search::ContextServiceInterface> {
        self.domain_services.context_service.clone()
    }

    /// Get search service
    pub fn search_service(&self) -> Arc<dyn mcb_domain::domain_services::search::SearchServiceInterface> {
        self.domain_services.search_service.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigBuilder;

    #[tokio::test]
    async fn test_infrastructure_container_creation() {
        let config = ConfigBuilder::new().build();
        let result = InfrastructureContainerBuilder::new(config).build().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_infrastructure_components() {
        let config = ConfigBuilder::new().build();
        let components = InfrastructureContainerBuilder::new(config).build().await.unwrap();

        // Test that components are accessible
        assert!(components.cache().get::<_, String>("test").await.unwrap().is_none());
        assert!(components.health().list_checks().await.contains(&"system"));
    }
}