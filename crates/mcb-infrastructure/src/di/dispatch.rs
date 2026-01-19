//! DI Component Dispatch
//!
//! Coordinates the initialization and dispatch of infrastructure components,
//! ensuring proper dependency order and lifecycle management.
//!
//! All components are resolved via Shaku DI - no manual factories.

use crate::config::AppConfig;
use crate::di::bootstrap::{DiContainer, init_app};
use mcb_domain::error::Result;

/// Component dispatcher for infrastructure initialization
///
/// Uses Shaku DI to resolve all components - no manual factories.
pub struct ComponentDispatcher {
    config: AppConfig,
}

impl ComponentDispatcher {
    /// Create a new component dispatcher
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Dispatch and initialize all infrastructure components via Shaku DI
    pub async fn dispatch(&self) -> Result<DiContainer> {
        init_app(self.config.clone()).await
    }
}

/// Infrastructure component initializer
///
/// Uses Shaku DI for all component resolution.
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

    /// Get the config reference
    fn config(&self) -> &AppConfig {
        &self.dispatcher.config
    }

    /// Initialize all infrastructure components via Shaku DI
    pub async fn initialize(&self) -> Result<DiContainer> {
        // Initialize logging first
        self.initialize_logging()?;

        // Initialize configuration watching if enabled
        self.initialize_config_watching().await?;

        // Dispatch all components via Shaku DI
        let container = self.dispatcher.dispatch().await?;

        // Log successful initialization
        tracing::info!("Infrastructure components initialized successfully");

        Ok(container)
    }

    /// Initialize logging system
    fn initialize_logging(&self) -> Result<()> {
        crate::logging::init_logging(self.config().logging.clone()).map_err(|e| {
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
