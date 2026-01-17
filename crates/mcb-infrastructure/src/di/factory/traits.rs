//! DI Factory traits
//!
//! Defines factory interfaces for creating infrastructure components
//! that may require complex initialization or external dependencies.

use async_trait::async_trait;
use mcb_domain::error::Result;

/// Factory for creating infrastructure components
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::InfrastructureFactory;
///
/// let factory = DefaultInfrastructureFactory::new(config);
/// let components = factory.create_components().await?;
/// // Use components.cache, components.crypto, etc.
/// ```
#[async_trait]
pub trait InfrastructureFactory: Send + Sync {
    /// Create infrastructure components from configuration
    async fn create_components(&self) -> Result<crate::di::bootstrap::DiContainer>;
}

/// Factory for creating cache providers
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::CacheProviderFactory;
///
/// let factory = DefaultCacheProviderFactory::new(cache_config);
/// let cache = factory.create_cache_provider().await?;
/// cache.set_json("key", "{}", Default::default()).await?;
/// ```
#[async_trait]
pub trait CacheProviderFactory: Send + Sync {
    /// Create a cache provider
    async fn create_cache_provider(&self) -> Result<crate::cache::provider::SharedCacheProvider>;
}

/// Factory for creating crypto services
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::CryptoServiceFactory;
///
/// let factory = DefaultCryptoServiceFactory::new(crypto_config);
/// let crypto = factory.create_crypto_service().await?;
/// let hash = crypto.hash_password("secret")?;
/// ```
#[async_trait]
pub trait CryptoServiceFactory: Send + Sync {
    /// Create a crypto service
    async fn create_crypto_service(&self) -> Result<crate::crypto::CryptoService>;
}

/// Factory for creating health registries
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::HealthRegistryFactory;
///
/// let factory = DefaultHealthRegistryFactory::new();
/// let registry = factory.create_health_registry().await?;
/// let status = registry.check_all().await;
/// println!("Overall health: {:?}", status.overall);
/// ```
#[async_trait]
pub trait HealthRegistryFactory: Send + Sync {
    /// Create a health registry
    async fn create_health_registry(&self) -> Result<crate::health::HealthRegistry>;
}
