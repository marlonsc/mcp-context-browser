//! DI Factory traits
//!
//! Defines factory interfaces for creating infrastructure components
//! that may require complex initialization or external dependencies.

use async_trait::async_trait;
use mcb_domain::error::Result;

/// Factory for creating infrastructure components
#[async_trait]
pub trait InfrastructureFactory: Send + Sync {
    /// Create infrastructure components from configuration
    async fn create_components(&self) -> Result<crate::di::bootstrap::InfrastructureComponents>;
}

/// Factory for creating cache providers
#[async_trait]
pub trait CacheProviderFactory: Send + Sync {
    /// Create a cache provider
    async fn create_cache_provider(&self) -> Result<crate::cache::provider::SharedCacheProvider>;
}

/// Factory for creating crypto services
#[async_trait]
pub trait CryptoServiceFactory: Send + Sync {
    /// Create a crypto service
    async fn create_crypto_service(&self) -> Result<crate::crypto::CryptoService>;
}

/// Factory for creating health registries
#[async_trait]
pub trait HealthRegistryFactory: Send + Sync {
    /// Create a health registry
    async fn create_health_registry(&self) -> Result<crate::health::HealthRegistry>;
}