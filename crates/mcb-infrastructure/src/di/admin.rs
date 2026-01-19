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
use dill::{Singleton, component};
use mcb_application::ports::registry::{
    CacheProviderConfig, EmbeddingProviderConfig, LanguageProviderConfig, VectorStoreProviderConfig,
};
use std::sync::Arc;

// ============================================================================
// Embedding Admin Service
// ============================================================================

/// Admin service for managing embedding providers at runtime
///
/// Provides methods to:
/// - List available providers
/// - Switch to a different provider
/// - Reload from persisted config
#[component]
#[dill::scope(Singleton)]
pub struct EmbeddingAdminService {
    resolver: Arc<EmbeddingProviderResolver>,
    handle: Arc<EmbeddingProviderHandle>,
}

impl EmbeddingAdminService {
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

// ============================================================================
// Vector Store Admin Service
// ============================================================================

/// Admin service for managing vector store providers at runtime
#[component]
#[dill::scope(Singleton)]
pub struct VectorStoreAdminService {
    resolver: Arc<VectorStoreProviderResolver>,
    handle: Arc<VectorStoreProviderHandle>,
}

impl VectorStoreAdminService {
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

// ============================================================================
// Cache Admin Service
// ============================================================================

/// Admin service for managing cache providers at runtime
#[component]
#[dill::scope(Singleton)]
pub struct CacheAdminService {
    resolver: Arc<CacheProviderResolver>,
    handle: Arc<CacheProviderHandle>,
}

impl CacheAdminService {
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

// ============================================================================
// Language Admin Service
// ============================================================================

/// Admin service for managing language chunking providers at runtime
#[component]
#[dill::scope(Singleton)]
pub struct LanguageAdminService {
    resolver: Arc<LanguageProviderResolver>,
    handle: Arc<LanguageProviderHandle>,
}

impl LanguageAdminService {
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
