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

use super::handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
use super::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use mcb_application::ports::registry::{
    CacheProviderConfig, EmbeddingProviderConfig, LanguageProviderConfig, VectorStoreProviderConfig,
};
use std::sync::Arc;

// ============================================================================
// Admin Service Interfaces (Traits)
// ============================================================================

/// Interface for embedding provider admin operations
///
/// # Example
///
/// ```ignore
/// // List available providers
/// let providers = admin.list_providers();
/// println!("Available: {:?}", providers);
///
/// // Switch to a different provider
/// let config = EmbeddingProviderConfig::new("ollama");
/// admin.switch_provider(config)?;
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
/// ```ignore
/// // List available vector store providers
/// let providers = admin.list_providers();
/// for provider in &providers {
///     println!("{}: {}", provider.name, provider.description);
/// }
///
/// // Switch to Milvus vector store
/// let config = VectorStoreProviderConfig::new("milvus")
///     .with_uri("http://localhost:19530");
/// admin.switch_provider(config)?;
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
/// ```ignore
/// // Check current cache provider
/// let current = admin.current_provider();
/// println!("Current cache: {}", current);
///
/// // Switch to Redis cache
/// let config = CacheProviderConfig::new("redis")
///     .with_url("redis://localhost:6379");
/// admin.switch_provider(config)?;
///
/// // Reload from persisted config
/// admin.reload_from_config()?;
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
/// ```ignore
/// // List available language chunking providers
/// let providers = admin.list_providers();
/// println!("Available language providers:");
/// for p in providers {
///     println!("  - {}", p.name);
/// }
///
/// // Switch to tree-sitter based chunking
/// let config = LanguageProviderConfig::new("tree-sitter");
/// admin.switch_provider(config)?;
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
// Embedding Admin Service
// ============================================================================

/// Admin service for managing embedding providers at runtime
///
/// Provides methods to:
/// - List available providers
/// - Switch to a different provider
/// - Reload from persisted config
pub struct EmbeddingAdminService {
    resolver: Arc<EmbeddingProviderResolver>,
    handle: Arc<EmbeddingProviderHandle>,
}

impl EmbeddingAdminService {
    /// Create a new embedding admin service
    pub fn new(
        resolver: Arc<EmbeddingProviderResolver>,
        handle: Arc<EmbeddingProviderHandle>,
    ) -> Self {
        Self { resolver, handle }
    }
    /// List all available embedding providers
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

    /// Get current provider name
    pub fn current_provider(&self) -> String {
        self.handle.provider_name()
    }

    /// Switch to a different embedding provider
    ///
    /// # Arguments
    /// * `config` - Configuration for the new provider
    ///
    /// # Returns
    /// * `Ok(())` - Provider switched successfully
    /// * `Err(String)` - Failed to switch (provider not found, config invalid, etc.)
    pub fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(&config)?;
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

impl std::fmt::Debug for EmbeddingAdminService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingAdminService")
            .field("current_provider", &self.current_provider())
            .finish()
    }
}

impl EmbeddingAdminInterface for EmbeddingAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        EmbeddingAdminService::list_providers(self)
    }

    fn current_provider(&self) -> String {
        EmbeddingAdminService::current_provider(self)
    }

    fn switch_provider(&self, config: EmbeddingProviderConfig) -> Result<(), String> {
        EmbeddingAdminService::switch_provider(self, config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        EmbeddingAdminService::reload_from_config(self)
    }
}

// ============================================================================
// Vector Store Admin Service
// ============================================================================

/// Admin service for managing vector store providers at runtime
pub struct VectorStoreAdminService {
    resolver: Arc<VectorStoreProviderResolver>,
    handle: Arc<VectorStoreProviderHandle>,
}

impl VectorStoreAdminService {
    /// Create a new vector store admin service
    pub fn new(
        resolver: Arc<VectorStoreProviderResolver>,
        handle: Arc<VectorStoreProviderHandle>,
    ) -> Self {
        Self { resolver, handle }
    }
    /// List all available vector store providers
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

    /// Switch to a different vector store provider
    pub fn switch_provider(&self, config: VectorStoreProviderConfig) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(&config)?;
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

impl std::fmt::Debug for VectorStoreAdminService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStoreAdminService").finish()
    }
}

impl VectorStoreAdminInterface for VectorStoreAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        VectorStoreAdminService::list_providers(self)
    }

    fn switch_provider(&self, config: VectorStoreProviderConfig) -> Result<(), String> {
        VectorStoreAdminService::switch_provider(self, config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        VectorStoreAdminService::reload_from_config(self)
    }
}

// ============================================================================
// Cache Admin Service
// ============================================================================

/// Admin service for managing cache providers at runtime
pub struct CacheAdminService {
    resolver: Arc<CacheProviderResolver>,
    handle: Arc<CacheProviderHandle>,
}

impl CacheAdminService {
    /// Create a new cache admin service
    pub fn new(resolver: Arc<CacheProviderResolver>, handle: Arc<CacheProviderHandle>) -> Self {
        Self { resolver, handle }
    }
    /// List all available cache providers
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

    /// Get current provider name
    pub fn current_provider(&self) -> String {
        self.handle.provider_name()
    }

    /// Switch to a different cache provider
    pub fn switch_provider(&self, config: CacheProviderConfig) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(&config)?;
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

impl std::fmt::Debug for CacheAdminService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheAdminService")
            .field("current_provider", &self.current_provider())
            .finish()
    }
}

impl CacheAdminInterface for CacheAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        CacheAdminService::list_providers(self)
    }

    fn current_provider(&self) -> String {
        CacheAdminService::current_provider(self)
    }

    fn switch_provider(&self, config: CacheProviderConfig) -> Result<(), String> {
        CacheAdminService::switch_provider(self, config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        CacheAdminService::reload_from_config(self)
    }
}

// ============================================================================
// Language Admin Service
// ============================================================================

/// Admin service for managing language chunking providers at runtime
pub struct LanguageAdminService {
    resolver: Arc<LanguageProviderResolver>,
    handle: Arc<LanguageProviderHandle>,
}

impl LanguageAdminService {
    /// Create a new language admin service
    pub fn new(
        resolver: Arc<LanguageProviderResolver>,
        handle: Arc<LanguageProviderHandle>,
    ) -> Self {
        Self { resolver, handle }
    }

    /// List all available language providers
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

    /// Switch to a different language provider
    pub fn switch_provider(&self, config: LanguageProviderConfig) -> Result<(), String> {
        let new_provider = self.resolver.resolve_from_override(&config)?;
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

impl std::fmt::Debug for LanguageAdminService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageAdminService").finish()
    }
}

impl LanguageAdminInterface for LanguageAdminService {
    fn list_providers(&self) -> Vec<ProviderInfo> {
        LanguageAdminService::list_providers(self)
    }

    fn switch_provider(&self, config: LanguageProviderConfig) -> Result<(), String> {
        LanguageAdminService::switch_provider(self, config)
    }

    fn reload_from_config(&self) -> Result<(), String> {
        LanguageAdminService::reload_from_config(self)
    }
}

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
