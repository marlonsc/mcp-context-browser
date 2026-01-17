//! DI Factory implementations
//!
//! Concrete implementations of factory traits for creating
//! infrastructure components.

use crate::cache::factory::CacheProviderFactory as CacheFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::di::bootstrap::{DiContainer, DiContainerBuilder};
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
    async fn create_components(&self) -> Result<DiContainer> {
        DiContainerBuilder::with_config(self.config.clone())
            .build()
            .await
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
        CacheFactory::create_from_config(&self.config).await
    }
}

/// Default crypto service factory
pub struct DefaultCryptoServiceFactory {
    master_key: Option<Vec<u8>>,
}

impl Default for DefaultCryptoServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultCryptoServiceFactory {
    /// Create a new crypto service factory
    pub fn new() -> Self {
        Self { master_key: None }
    }

    /// Create a factory with a specific master key
    pub fn with_master_key(master_key: Vec<u8>) -> Self {
        Self {
            master_key: Some(master_key),
        }
    }
}

#[async_trait]
impl CryptoServiceFactory for DefaultCryptoServiceFactory {
    async fn create_crypto_service(&self) -> Result<CryptoService> {
        let master_key = self
            .master_key
            .clone()
            .unwrap_or_else(CryptoService::generate_master_key);

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
        registry
            .register_checker("system".to_string(), system_checker)
            .await;

        Ok(registry)
    }
}

impl Default for DefaultHealthRegistryFactory {
    fn default() -> Self {
        Self::new()
    }
}
