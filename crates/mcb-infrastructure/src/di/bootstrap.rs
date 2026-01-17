//! DI Container Bootstrap - Clean Architecture Composition Root
//!
//! Provides the composition root for the dependency injection system following
//! Clean Architecture principles.
//!
//! ## Architecture
//!
//! External providers (embedding, vector_store, cache, language) are resolved
//! dynamically via the registry system. Internal infrastructure services use
//! Shaku DI modules.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create AppContext with resolved providers
//! let context = init_app(AppConfig::default()).await?;
//!
//! // Use providers
//! let embedding = context.providers.embedding.embed("hello").await?;
//! ```

use crate::config::AppConfig;
use crate::di::resolver::{resolve_providers, ResolvedProviders};
use mcb_domain::error::Result;
use tracing::info;

// Import Shaku module implementations for internal infrastructure
use super::modules::{
    admin::AdminModuleImpl, infrastructure::InfrastructureModuleImpl, server::ServerModuleImpl,
};

/// Application context with resolved providers and infrastructure modules
///
/// This is the composition root that combines:
/// - External providers resolved from registry (embedding, vector_store, cache, language)
/// - Internal infrastructure services via Shaku modules
pub struct AppContext {
    /// Application configuration
    pub config: AppConfig,
    /// Resolved external providers
    pub providers: ResolvedProviders,
    /// Infrastructure module (auth, events, metrics, sync, snapshot)
    pub infrastructure: InfrastructureModuleImpl,
    /// Server module (performance metrics, indexing operations)
    pub server: ServerModuleImpl,
    /// Admin module (marker for future admin services)
    pub admin: AdminModuleImpl,
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("providers", &self.providers)
            .finish_non_exhaustive()
    }
}

/// Initialize application context with resolved providers
///
/// Resolves external providers from registry and builds internal Shaku modules.
///
/// ## Example
///
/// ```ignore
/// let context = init_app(config).await?;
/// let embedding = context.providers.embedding;
/// ```
pub async fn init_app(config: AppConfig) -> Result<AppContext> {
    info!("Initializing application context");

    // Resolve external providers from registry
    let providers = resolve_providers(&config)?;
    info!("Resolved providers: {:?}", providers);

    // Build internal infrastructure modules via Shaku
    let infrastructure = InfrastructureModuleImpl::builder().build();
    let server = ServerModuleImpl::builder().build();
    let admin = AdminModuleImpl::builder().build();

    info!("Application context initialized");

    Ok(AppContext {
        config,
        providers,
        infrastructure,
        server,
        admin,
    })
}

/// Initialize application for testing
///
/// Uses null/test providers by default.
pub async fn init_test_app() -> Result<AppContext> {
    let config = AppConfig::default();
    init_app(config).await
}

// ============================================================================
// Legacy Types (for backward compatibility during migration)
// ============================================================================

/// Type alias for backward compatibility
pub type DiContainer = AppContext;

/// Legacy container builder (deprecated, use init_app directly)
#[deprecated(note = "Use init_app() directly")]
pub struct DiContainerBuilder {
    config: Option<AppConfig>,
}

#[allow(deprecated)]
impl DiContainerBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    pub async fn build(self) -> Result<AppContext> {
        let config = self.config.unwrap_or_default();
        init_app(config).await
    }
}

#[allow(deprecated)]
impl Default for DiContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create context for testing
pub async fn create_test_container() -> Result<AppContext> {
    init_test_app().await
}
