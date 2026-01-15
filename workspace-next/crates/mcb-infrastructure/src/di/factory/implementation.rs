//! DI Factory implementations
//!
//! Concrete implementations of factory traits for creating
//! infrastructure components.

use crate::cache::CacheProviderFactory as CacheFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::di::bootstrap::{InfrastructureComponents, InfrastructureContainerBuilder};
use crate::di::factory::traits::*;
use crate::health::HealthRegistry;
use async_trait::async_trait;
use mcb_domain::error::Result;

/// Default infrastructure factory implementation
pub struct DefaultInfrastructureFactory {
    config: AppConfig,
}

impl DefaultInfrastructureFactory {
    /// Create a new default infrastructure factory
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl InfrastructureFactory for DefaultInfrastructureFactory {
    async fn create_components(&self) -> Result<InfrastructureComponents> {
        InfrastructureContainerBuilder::new(self.config.clone()).build().await
    }
}

/// Default cache provider factory
pub struct DefaultCacheProviderFactory {
    config: crate::config::data::CacheConfig,
}

impl DefaultCacheProviderFactory {
    /// Create a new cache provider factory
    pub fn new(config: crate::config::data::CacheConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl CacheProviderFactory for DefaultCacheProviderFactory {
    async fn create_cache_provider(&self) -> Result<crate::cache::provider::SharedCacheProvider> {
        CacheFactory::create_from_config(&self.config)
    }
}

/// Default crypto service factory
pub struct DefaultCryptoServiceFactory {
    master_key: Option<Vec<u8>>,
}

impl DefaultCryptoServiceFactory {
    /// Create a new crypto service factory
    pub fn new() -> Self {
        Self { master_key: None }
    }

    /// Create a factory with a specific master key
    pub fn with_master_key(master_key: Vec<u8>) -> Self {
        Self { master_key: Some(master_key) }
    }
}

#[async_trait]
impl CryptoServiceFactory for DefaultCryptoServiceFactory {
    async fn create_crypto_service(&self) -> Result<CryptoService> {
        let master_key = self.master_key.clone()
            .unwrap_or_else(|| CryptoService::generate_master_key());

        CryptoService::new(master_key)
    }
}

/// Default health registry factory
pub struct DefaultHealthRegistryFactory;

impl DefaultHealthRegistryFactory {
    /// Create a new health registry factory
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthRegistryFactory for DefaultHealthRegistryFactory {
    async fn create_health_registry(&self) -> Result<HealthRegistry> {
        let registry = HealthRegistry::new();

        // Register default health checkers
        let system_checker = crate::health::checkers::SystemHealthChecker::new();
        registry.register_checker("system".to_string(), system_checker).await;

        Ok(registry)
    }
}

impl Default for DefaultHealthRegistryFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_service_factory() {
        let factory = DefaultCryptoServiceFactory::new();
        let service = factory.create_crypto_service().await.unwrap();

        // Test that the service can encrypt/decrypt
        let data = b"test data";
        let encrypted = service.encrypt(data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[tokio::test]
    async fn test_health_registry_factory() {
        let factory = DefaultHealthRegistryFactory::new();
        let registry = factory.create_health_registry().await.unwrap();

        let checks = registry.list_checks().await;
        assert!(checks.contains(&"system"));
    }
}