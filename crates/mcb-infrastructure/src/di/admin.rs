//! Provider Admin Services - Runtime provider switching via API
//!
//! These components allow switching providers at runtime without restarting
//! the application. Used by admin API endpoints.
//!
//! ## Pattern
//!
//! ```text
//! Admin API → AdminService.switch_provider() → Resolver → Handle.set()
//! ```

use super::handle::Handle;
use super::handles::{CacheHandleExt, EmbeddingHandleExt};
use super::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use mcb_application::ports::registry::{
    CacheProviderConfig, EmbeddingProviderConfig, LanguageProviderConfig, VectorStoreProviderConfig,
};
use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use std::sync::Arc;

// ============================================================================
// Common Types
// ============================================================================

/// Information about an available provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider name (used in config)
    pub name: String,
    /// Human-readable description
    pub description: String,
}

impl ProviderInfo {
    /// Create new provider info
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

// ============================================================================
// Resolver Trait - Common Interface for All Provider Resolvers
// ============================================================================

/// Common interface for provider resolvers
///
/// This trait abstracts the resolution logic so AdminService can work
/// with any resolver type generically.
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::ProviderResolver;
///
/// fn list_providers<R, P, C>(resolver: &R) -> Vec<String>
/// where
///     R: ProviderResolver<P, C>,
///     P: ?Sized + Send + Sync,
/// {
///     resolver.list_available()
///         .into_iter()
///         .map(|(name, _)| name.to_string())
///         .collect()
/// }
/// ```
pub trait ProviderResolver<P: ?Sized + Send + Sync, C>: Send + Sync {
    /// Resolve provider from current application config
    fn resolve_from_config(&self) -> Result<Arc<P>, String>;
    /// Resolve provider from override config (for admin API)
    fn resolve_from_override(&self, config: &C) -> Result<Arc<P>, String>;
    /// List available providers
    fn list_available(&self) -> Vec<(&'static str, &'static str)>;
}

// ============================================================================
// Resolver Trait Implementations
// ============================================================================

impl ProviderResolver<dyn EmbeddingProvider, EmbeddingProviderConfig>
    for EmbeddingProviderResolver
{
    fn resolve_from_config(&self) -> Result<Arc<dyn EmbeddingProvider>, String> {
        EmbeddingProviderResolver::resolve_from_config(self)
    }

    fn resolve_from_override(
        &self,
        config: &EmbeddingProviderConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>, String> {
        EmbeddingProviderResolver::resolve_from_override(self, config)
    }

    fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        EmbeddingProviderResolver::list_available(self)
    }
}

impl ProviderResolver<dyn VectorStoreProvider, VectorStoreProviderConfig>
    for VectorStoreProviderResolver
{
    fn resolve_from_config(&self) -> Result<Arc<dyn VectorStoreProvider>, String> {
        VectorStoreProviderResolver::resolve_from_config(self)
    }

    fn resolve_from_override(
        &self,
        config: &VectorStoreProviderConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>, String> {
        VectorStoreProviderResolver::resolve_from_override(self, config)
    }

    fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        VectorStoreProviderResolver::list_available(self)
    }
}

impl ProviderResolver<dyn CacheProvider, CacheProviderConfig> for CacheProviderResolver {
    fn resolve_from_config(&self) -> Result<Arc<dyn CacheProvider>, String> {
        CacheProviderResolver::resolve_from_config(self)
    }

    fn resolve_from_override(
        &self,
        config: &CacheProviderConfig,
    ) -> Result<Arc<dyn CacheProvider>, String> {
        CacheProviderResolver::resolve_from_override(self, config)
    }

    fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        CacheProviderResolver::list_available(self)
    }
}

impl ProviderResolver<dyn LanguageChunkingProvider, LanguageProviderConfig>
    for LanguageProviderResolver
{
    fn resolve_from_config(&self) -> Result<Arc<dyn LanguageChunkingProvider>, String> {
        LanguageProviderResolver::resolve_from_config(self)
    }

    fn resolve_from_override(
        &self,
        config: &LanguageProviderConfig,
    ) -> Result<Arc<dyn LanguageChunkingProvider>, String> {
        LanguageProviderResolver::resolve_from_override(self, config)
    }

    fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        LanguageProviderResolver::list_available(self)
    }
}

// ============================================================================
// Generic Admin Service
// ============================================================================

/// Generic admin service for managing providers at runtime
///
/// This struct provides the core admin functionality for any provider type.
/// Specific admin services are type aliases with the appropriate resolver
/// and provider types.
///
/// # Type Parameters
///
/// * `R` - Resolver type that implements `ProviderResolver<P, C>`
/// * `P` - Provider trait type (e.g., `dyn EmbeddingProvider`)
/// * `C` - Config type for the provider
pub struct AdminService<R, P: ?Sized + Send + Sync, C> {
    resolver: Arc<R>,
    handle: Arc<Handle<P>>,
    _config_marker: std::marker::PhantomData<C>,
}

impl<R, P, C> AdminService<R, P, C>
where
    R: ProviderResolver<P, C>,
    P: ?Sized + Send + Sync,
{
    /// Create a new admin service
    pub fn new(resolver: Arc<R>, handle: Arc<Handle<P>>) -> Self {
        Self {
            resolver,
            handle,
            _config_marker: std::marker::PhantomData,
        }
    }

    /// List all available providers
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.resolver
            .list_available()
            .into_iter()
            .map(|(name, description)| ProviderInfo {
                name: name.to_string(),
                description: description.to_string(),
            })
            .collect()
    }

    /// Switch to a different provider
    ///
    /// # Arguments
    /// * `config` - Configuration for the new provider
    ///
    /// # Returns
    /// * `Ok(())` - Provider switched successfully
    /// * `Err(String)` - Failed to switch (provider not found, config invalid, etc.)
    pub fn switch_provider(&self, config: &C) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(config)?;
        self.handle.set(new_provider);
        Ok(())
    }

    /// Reload provider from current application config
    pub fn reload_from_config(&self) -> Result<(), String> {
        let provider = self.resolver.resolve_from_config()?;
        self.handle.set(provider);
        Ok(())
    }
}

impl<R, P, C> std::fmt::Debug for AdminService<R, P, C>
where
    P: ?Sized + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminService").finish()
    }
}

// ============================================================================
// Admin Service Interfaces (Traits)
// ============================================================================

/// Interface for embedding provider admin operations
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::admin::{EmbeddingAdminInterface, ProviderInfo};
///
/// fn list_embedding_providers(admin: &dyn EmbeddingAdminInterface) -> Vec<String> {
///     admin.list_providers()
///         .into_iter()
///         .map(|info| info.name)
///         .collect()
/// }
/// ```
pub trait EmbeddingAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available embedding providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Get current provider name
    fn current_provider(&self) -> String;
    /// Switch to a different embedding provider
    fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for vector store provider admin operations
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::admin::{VectorStoreAdminInterface, ProviderInfo};
///
/// fn list_vector_providers(admin: &dyn VectorStoreAdminInterface) -> Vec<String> {
///     admin.list_providers()
///         .into_iter()
///         .map(|info| info.name)
///         .collect()
/// }
/// ```
pub trait VectorStoreAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available vector store providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Switch to a different vector store provider
    fn switch_provider(&self, config: VectorStoreProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for cache provider admin operations
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::admin::{CacheAdminInterface, ProviderInfo};
///
/// fn list_cache_providers(admin: &dyn CacheAdminInterface) -> Vec<String> {
///     admin.list_providers()
///         .into_iter()
///         .map(|info| info.name)
///         .collect()
/// }
/// ```
pub trait CacheAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available cache providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Get current provider name
    fn current_provider(&self) -> String;
    /// Switch to a different cache provider
    fn switch_provider(&self, config: CacheProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

/// Interface for language provider admin operations
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::admin::{LanguageAdminInterface, ProviderInfo};
///
/// fn list_language_providers(admin: &dyn LanguageAdminInterface) -> Vec<String> {
///     admin.list_providers()
///         .into_iter()
///         .map(|info| info.name)
///         .collect()
/// }
/// ```
pub trait LanguageAdminInterface: Send + Sync + std::fmt::Debug {
    /// List all available language providers
    fn list_providers(&self) -> Vec<ProviderInfo>;
    /// Switch to a different language provider
    fn switch_provider(&self, config: LanguageProviderConfig) -> Result<(), String>;
    /// Reload provider from current application config
    fn reload_from_config(&self) -> Result<(), String>;
}

// ============================================================================
// Type Aliases (Backward Compatibility)
// ============================================================================

/// Admin service for managing embedding providers at runtime
pub type EmbeddingAdminService =
    AdminService<EmbeddingProviderResolver, dyn EmbeddingProvider, EmbeddingProviderConfig>;

/// Admin service for managing vector store providers at runtime
pub type VectorStoreAdminService =
    AdminService<VectorStoreProviderResolver, dyn VectorStoreProvider, VectorStoreProviderConfig>;

/// Admin service for managing cache providers at runtime
pub type CacheAdminService =
    AdminService<CacheProviderResolver, dyn CacheProvider, CacheProviderConfig>;

/// Admin service for managing language chunking providers at runtime
pub type LanguageAdminService =
    AdminService<LanguageProviderResolver, dyn LanguageChunkingProvider, LanguageProviderConfig>;

// ============================================================================
// Trait Implementations for Specific Admin Services
// ============================================================================

impl EmbeddingAdminInterface for EmbeddingAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        AdminService::list_providers(self)
    }

    fn current_provider(&self) -> String {
        self.handle.provider_name()
    }

    fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String> {
        AdminService::switch_provider(self, &config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        AdminService::reload_from_config(self)
    }
}

impl VectorStoreAdminInterface for VectorStoreAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        AdminService::list_providers(self)
    }

    fn switch_provider(&self, config: VectorStoreProviderConfig) -> Result<(), String> {
        AdminService::switch_provider(self, &config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        AdminService::reload_from_config(self)
    }
}

impl CacheAdminInterface for CacheAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        AdminService::list_providers(self)
    }

    fn current_provider(&self) -> String {
        self.handle.provider_name()
    }

    fn switch_provider(&self, config: CacheProviderConfig) -> Result<(), String> {
        AdminService::switch_provider(self, &config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        AdminService::reload_from_config(self)
    }
}

impl LanguageAdminInterface for LanguageAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        AdminService::list_providers(self)
    }

    fn switch_provider(&self, config: LanguageProviderConfig) -> Result<(), String> {
        AdminService::switch_provider(self, &config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        AdminService::reload_from_config(self)
    }
}
