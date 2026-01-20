//! Provider Resolvers - Components for resolving providers from linkme registry
//!
//! These components wrap the linkme registry resolution and can be injected
//! into other services.
//!
//! ## Pattern
//!
//! ```text
//! AppConfig (injected) → Resolver → linkme registry → Arc<dyn Provider>
//! ```

use crate::config::AppConfig;
use mcb_application::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use mcb_application::ports::registry::{
    CacheProviderConfig, EmbeddingProviderConfig, LanguageProviderConfig,
    VectorStoreProviderConfig, resolve_cache_provider, resolve_embedding_provider,
    resolve_language_provider, resolve_vector_store_provider,
};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use std::sync::Arc;

// ============================================================================
// Embedding Provider Resolver
// ============================================================================

/// Resolver component for embedding providers
///
/// Uses the linkme registry to resolve embedding providers by name.
/// Can resolve from current config or from an override config.
pub struct EmbeddingProviderResolver {
    config: Arc<AppConfig>,
}

impl EmbeddingProviderResolver {
    /// Create a new resolver with config
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Resolve provider from current application config
    pub fn resolve_from_config(&self) -> Result<Arc<dyn EmbeddingProvider>, String> {
        // First, check direct config (flat env vars like MCP__PROVIDERS__EMBEDDING__PROVIDER)
        if let Some(ref provider_name) = self.config.providers.embedding.provider {
            let mut registry_config = EmbeddingProviderConfig::new(provider_name);
            if let Some(ref model) = self.config.providers.embedding.model {
                registry_config = registry_config.with_model(model);
            }
            if let Some(ref base_url) = self.config.providers.embedding.base_url {
                registry_config = registry_config.with_base_url(base_url);
            }
            if let Some(ref api_key) = self.config.providers.embedding.api_key {
                registry_config = registry_config.with_api_key(api_key);
            }
            if let Some(dimensions) = self.config.providers.embedding.dimensions {
                registry_config = registry_config.with_dimensions(dimensions);
            }
            return resolve_embedding_provider(&registry_config);
        }

        // Fallback to named config (TOML: [providers.embedding.default])
        if let Some(default_config) = self.config.providers.embedding.configs.get("default") {
            // If there's a specific config for this provider, use it
            if let Some(specific_config) = self
                .config
                .providers
                .embedding
                .configs
                .get(&default_config.provider.to_string())
            {
                let registry_config = embedding_config_to_registry(specific_config);
                resolve_embedding_provider(&registry_config)
            } else {
                // Use the default config directly
                let registry_config = embedding_config_to_registry(default_config);
                resolve_embedding_provider(&registry_config)
            }
        } else {
            // Fallback to null provider if no default configured
            resolve_embedding_provider(&EmbeddingProviderConfig::new("null"))
        }
    }

    /// Resolve provider from override config (for admin API)
    pub fn resolve_from_override(
        &self,
        override_config: &EmbeddingProviderConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>, String> {
        resolve_embedding_provider(override_config)
    }

    /// List available embedding providers
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_application::ports::registry::list_embedding_providers()
    }
}

impl std::fmt::Debug for EmbeddingProviderResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingProviderResolver").finish()
    }
}

// ============================================================================
// Vector Store Provider Resolver
// ============================================================================

/// Resolver component for vector store providers
///
/// Uses the linkme registry to resolve vector store providers by name.
/// Can resolve from current config or from an override config.
pub struct VectorStoreProviderResolver {
    config: Arc<AppConfig>,
}

impl VectorStoreProviderResolver {
    /// Create a new resolver with config
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Resolve provider from current application config
    pub fn resolve_from_config(&self) -> Result<Arc<dyn VectorStoreProvider>, String> {
        // First, check direct config (flat env vars like MCP__PROVIDERS__VECTOR_STORE__PROVIDER)
        if let Some(ref provider_name) = self.config.providers.vector_store.provider {
            let mut registry_config = VectorStoreProviderConfig::new(provider_name);
            if let Some(ref address) = self.config.providers.vector_store.address {
                registry_config = registry_config.with_uri(address);
            }
            if let Some(dimensions) = self.config.providers.vector_store.dimensions {
                registry_config = registry_config.with_dimensions(dimensions);
            }
            if let Some(ref collection) = self.config.providers.vector_store.collection {
                registry_config = registry_config.with_collection(collection);
            }
            return resolve_vector_store_provider(&registry_config);
        }

        // Fallback to named config (TOML: [providers.vector_store.default])
        if let Some(default_config) = self.config.providers.vector_store.configs.get("default") {
            // If there's a specific config for this provider, use it
            if let Some(specific_config) = self
                .config
                .providers
                .vector_store
                .configs
                .get(&default_config.provider.to_string())
            {
                let registry_config = vector_store_config_to_registry(specific_config);
                resolve_vector_store_provider(&registry_config)
            } else {
                // Use the default config directly
                let registry_config = vector_store_config_to_registry(default_config);
                resolve_vector_store_provider(&registry_config)
            }
        } else {
            // Fallback to memory provider if no default configured
            resolve_vector_store_provider(&VectorStoreProviderConfig::new("memory"))
        }
    }

    /// Resolve provider from override config (for admin API)
    pub fn resolve_from_override(
        &self,
        override_config: &VectorStoreProviderConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>, String> {
        resolve_vector_store_provider(override_config)
    }

    /// List available vector store providers
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_application::ports::registry::list_vector_store_providers()
    }
}

impl std::fmt::Debug for VectorStoreProviderResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorStoreProviderResolver").finish()
    }
}

// ============================================================================
// Cache Provider Resolver
// ============================================================================

/// Resolver component for cache providers
///
/// Uses the linkme registry to resolve cache providers by name.
/// Can resolve from current config or from an override config.
pub struct CacheProviderResolver {
    config: Arc<AppConfig>,
}

impl CacheProviderResolver {
    /// Create a new resolver with config
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Resolve provider from current application config
    pub fn resolve_from_config(&self) -> Result<Arc<dyn CacheProvider>, String> {
        let cache_provider_name = match &self.config.system.infrastructure.cache.provider {
            crate::config::CacheProvider::Moka => "moka",
            crate::config::CacheProvider::Redis => "redis",
        };

        let registry_config = CacheProviderConfig {
            provider: cache_provider_name.to_string(),
            uri: self.config.system.infrastructure.cache.redis_url.clone(),
            max_size: Some(self.config.system.infrastructure.cache.max_size),
            ttl_secs: Some(self.config.system.infrastructure.cache.default_ttl_secs),
            namespace: Some(self.config.system.infrastructure.cache.namespace.clone()),
            extra: Default::default(),
        };

        resolve_cache_provider(&registry_config)
    }

    /// Resolve provider from override config (for admin API)
    pub fn resolve_from_override(
        &self,
        override_config: &CacheProviderConfig,
    ) -> Result<Arc<dyn CacheProvider>, String> {
        resolve_cache_provider(override_config)
    }

    /// List available cache providers
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_application::ports::registry::list_cache_providers()
    }
}

impl std::fmt::Debug for CacheProviderResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheProviderResolver").finish()
    }
}

// ============================================================================
// Language Provider Resolver
// ============================================================================

/// Resolver component for language chunking providers
///
/// Uses the linkme registry to resolve language providers by name.
/// Can resolve from current config or from an override config.
pub struct LanguageProviderResolver {
    #[allow(dead_code)]
    config: Arc<AppConfig>,
}

impl LanguageProviderResolver {
    /// Create a new resolver with config
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Resolve provider from current application config
    pub fn resolve_from_config(&self) -> Result<Arc<dyn LanguageChunkingProvider>, String> {
        // Language provider is always "universal" for now
        let registry_config = LanguageProviderConfig::new("universal");
        resolve_language_provider(&registry_config)
    }

    /// Resolve provider from override config (for admin API)
    pub fn resolve_from_override(
        &self,
        override_config: &LanguageProviderConfig,
    ) -> Result<Arc<dyn LanguageChunkingProvider>, String> {
        resolve_language_provider(override_config)
    }

    /// List available language providers
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_application::ports::registry::list_language_providers()
    }
}

impl std::fmt::Debug for LanguageProviderResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageProviderResolver").finish()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert domain EmbeddingConfig to registry EmbeddingProviderConfig
fn embedding_config_to_registry(config: &EmbeddingConfig) -> EmbeddingProviderConfig {
    EmbeddingProviderConfig {
        provider: config.provider.to_string(),
        model: Some(config.model.clone()),
        api_key: config.api_key.clone(),
        base_url: config.base_url.clone(),
        dimensions: config.dimensions,
        extra: Default::default(),
    }
}

/// Convert domain VectorStoreConfig to registry VectorStoreProviderConfig
fn vector_store_config_to_registry(config: &VectorStoreConfig) -> VectorStoreProviderConfig {
    VectorStoreProviderConfig {
        provider: config.provider.to_string(),
        uri: config.address.clone(),
        collection: config.collection.clone(),
        dimensions: config.dimensions,
        api_key: config.token.clone(),
        encrypted: None,
        encryption_key: None,
        extra: Default::default(),
    }
}
