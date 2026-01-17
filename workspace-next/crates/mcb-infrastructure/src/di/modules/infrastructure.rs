//! Infrastructure DI Module
//!
//! Defines how infrastructure components are wired together using Shaku.
//! This module follows Clean Architecture by keeping infrastructure
//! components isolated and injectable.

use crate::cache::provider::SharedCacheProvider;
use crate::cache::CacheProviderFactory;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use crate::health::{HealthRegistry, checkers};
use mcb_domain::error::Result;
use shaku::{module, Component, Interface, Module, ModuleBuildContext};
use std::sync::Arc;

/// Infrastructure components interface
#[derive(Component)]
#[shaku(interface = InfrastructureProvider)]
pub struct InfrastructureProviderImpl {
    pub cache_provider: SharedCacheProvider,
    pub crypto_service: CryptoService,
    pub health_registry: HealthRegistry,
    pub app_config: AppConfig,
}

/// Infrastructure provider interface
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::di::InfrastructureProvider;
///
/// // Get from DI container
/// let infra: Arc<dyn InfrastructureProvider> = container.resolve();
///
/// // Access services
/// let cache = infra.cache();
/// let crypto = infra.crypto();
/// let health = infra.health();
/// let config = infra.config();
/// ```
pub trait InfrastructureProvider: Interface {
    fn cache(&self) -> &SharedCacheProvider;
    fn crypto(&self) -> &CryptoService;
    fn health(&self) -> &HealthRegistry;
    fn config(&self) -> &AppConfig;
}

impl InfrastructureProvider for InfrastructureProviderImpl {
    fn cache(&self) -> &SharedCacheProvider {
        &self.cache_provider
    }

    fn crypto(&self) -> &CryptoService {
        &self.crypto_service
    }

    fn health(&self) -> &HealthRegistry {
        &self.health_registry
    }

    fn config(&self) -> &AppConfig {
        &self.app_config
    }
}

/// Infrastructure module
#[module]
pub struct InfrastructureModule {
    config: AppConfig,
}

impl InfrastructureModule {
    /// Create a new infrastructure module
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}

impl<Ctx: ModuleBuildContext> Module<Ctx> for InfrastructureModule {
    fn build_context(self, ctx: &mut Ctx) -> Result<(), shaku::Error> {
        // Create cache provider
        let cache_provider = if self.config.infrastructure.cache.enabled {
            CacheProviderFactory::create_from_config(&self.config.infrastructure.cache)
                .map_err(|e| shaku::Error::Other(format!("Cache provider creation failed: {}", e)))?
        } else {
            CacheProviderFactory::create_null()
        };

        // Create crypto service
        let secret = &self.config.auth.jwt.secret;
        let master_key = if secret.len() >= 32 {
            secret.as_bytes()[..32].to_vec()
        } else {
            CryptoService::generate_master_key()
        };

        let crypto_service = CryptoService::new(master_key)
            .map_err(|e| shaku::Error::Other(format!("Crypto service creation failed: {}", e)))?;

        // Create health registry
        let health_registry = HealthRegistry::new();

        // Register built-in health checkers
        let system_checker = checkers::SystemHealthChecker::new();
        ctx.block_on(async {
            health_registry.register_checker("system".to_string(), system_checker).await;
        });

        // Create infrastructure provider
        let provider = InfrastructureProviderImpl {
            cache_provider,
            crypto_service,
            health_registry,
            app_config: self.config,
        };

        ctx.build_component(provider)?;
        Ok(())
    }
}

