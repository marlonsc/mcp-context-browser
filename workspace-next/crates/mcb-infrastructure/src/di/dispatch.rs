//! DI Component Dispatch
//!
//! Coordinates the initialization and dispatch of infrastructure components,
//! ensuring proper dependency order and lifecycle management.

use crate::config::AppConfig;
use crate::di::bootstrap::{DiContainer, DiContainerBuilder};
use crate::di::factory::implementation::*;
use crate::di::factory::traits::{
    CacheProviderFactory, CryptoServiceFactory, HealthRegistryFactory,
};
use mcb_domain::error::Result;

/// Component dispatcher for infrastructure initialization
pub struct ComponentDispatcher {
    config: AppConfig,
}

impl ComponentDispatcher {
    /// Create a new component dispatcher
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Dispatch and initialize all infrastructure components
    pub async fn dispatch(&self) -> Result<DiContainer> {
        DiContainerBuilder::with_config(self.config.clone()).build().await
    }

    /// Create cache provider
    #[allow(dead_code)]
    async fn create_cache_provider(&self) -> Result<crate::cache::provider::SharedCacheProvider> {
        let factory = DefaultCacheProviderFactory::new(self.config.system.infrastructure.cache.clone());
        factory.create_cache_provider().await
    }

    /// Create crypto service
    #[allow(dead_code)]
    async fn create_crypto_service(&self) -> Result<crate::crypto::CryptoService> {
        // AES-GCM requires exactly 32 bytes for the key
        let master_key = if self.config.auth.jwt.secret.len() >= 32 {
            // Use first 32 bytes of the JWT secret as the master key
            self.config.auth.jwt.secret.as_bytes()[..32].to_vec()
        } else {
            crate::crypto::CryptoService::generate_master_key()
        };

        let factory = DefaultCryptoServiceFactory::with_master_key(master_key);
        factory.create_crypto_service().await
    }

    /// Create health registry
    #[allow(dead_code)]
    async fn create_health_registry(&self) -> Result<crate::health::HealthRegistry> {
        let factory = DefaultHealthRegistryFactory::new();
        factory.create_health_registry().await
    }
}

/// Infrastructure component initializer
pub struct InfrastructureInitializer {
    dispatcher: ComponentDispatcher,
}

impl InfrastructureInitializer {
    /// Create a new infrastructure initializer
    pub fn new(config: AppConfig) -> Self {
        Self {
            dispatcher: ComponentDispatcher::new(config),
        }
    }

    /// Initialize all infrastructure components
    pub async fn initialize(&self) -> Result<DiContainer> {
        // Initialize logging first
        self.initialize_logging()?;

        // Initialize configuration watching if enabled
        self.initialize_config_watching().await?;

        // Dispatch all components
        let container = self.dispatcher.dispatch().await?;

        // Log successful initialization
        tracing::info!("Infrastructure components initialized successfully");

        Ok(container)
    }

    /// Initialize logging system
    fn initialize_logging(&self) -> Result<()> {
        crate::logging::init_logging(self.dispatcher.config.logging.clone()).map_err(|e| {
            mcb_domain::error::Error::Infrastructure {
                message: format!("Failed to initialize logging: {}", e),
                source: Some(Box::new(e)),
            }
        })
    }

    /// Initialize configuration watching if enabled
    async fn initialize_config_watching(&self) -> Result<()> {
        // Configuration watching would be initialized here if needed
        // For now, this is a placeholder for future implementation
        Ok(())
    }
}
