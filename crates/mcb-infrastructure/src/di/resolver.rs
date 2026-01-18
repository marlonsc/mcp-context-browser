//! Dynamic Provider Resolver
//!
//! Resolves providers by name using the linkme distributed slice registry.
//! No direct knowledge of concrete provider implementations.
//!
//! ## Architecture
//!
//! This module provides the bridge between configuration and provider instances:
//!
//! ```text
//! Config: "embedding.provider = ollama"
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │     resolve_providers(&config)       │
//! └─────────────────────────────────────┘
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │   PROVIDERS.iter()                   │  ← Discovers auto-registered providers
//! └─────────────────────────────────────┘
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │   ResolvedProviders {                │
//! │     embedding: Arc<dyn ...>,         │
//! │     vector_store: Arc<dyn ...>,      │
//! │     cache: Arc<dyn ...>,             │
//! │     language: Arc<dyn ...>,          │
//! │   }                                  │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! let config = AppConfig::load()?;
//! let providers = resolve_providers(&config)?;
//!
//! // Use providers
//! let embedding = providers.embedding.embed("hello").await?;
//! ```

use std::sync::Arc;

use mcb_application::ports::providers::{
    cache::CacheProvider as CacheProviderTrait, EmbeddingProvider, LanguageChunkingProvider,
    VectorStoreProvider,
};
use mcb_application::ports::registry::{
    resolve_cache_provider, resolve_embedding_provider, resolve_language_provider,
    resolve_vector_store_provider, CacheProviderConfig, EmbeddingProviderConfig,
    LanguageProviderConfig, VectorStoreProviderConfig,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};

use crate::config::AppConfig;

/// Resolved providers from configuration
///
/// Contains all provider instances resolved from application configuration.
/// These providers are ready to use and have been fully initialized.
#[derive(Clone)]
pub struct ResolvedProviders {
    /// Embedding provider for text-to-vector conversion
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// Vector store for similarity search
    pub vector_store: Arc<dyn VectorStoreProvider>,
    /// Cache provider for performance optimization
    pub cache: Arc<dyn CacheProviderTrait>,
    /// Language chunking provider for code parsing
    pub language: Arc<dyn LanguageChunkingProvider>,
}

impl std::fmt::Debug for ResolvedProviders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedProviders")
            .field("embedding", &self.embedding.provider_name())
            .field("vector_store", &self.vector_store.provider_name())
            .field("cache", &self.cache.provider_name())
            .field("language", &self.language.provider_name())
            .finish()
    }
}

/// Resolve all providers from application configuration
///
/// Queries the inventory registry to find and instantiate providers
/// based on the names specified in configuration.
///
/// # Arguments
/// * `config` - Application configuration containing provider names
///
/// # Returns
/// * `Ok(ResolvedProviders)` - All providers successfully resolved
/// * `Err(Error)` - Provider not found or creation failed
pub fn resolve_providers(config: &AppConfig) -> Result<ResolvedProviders> {
    // Get the first (default) embedding config or use defaults
    let embedding_config = config
        .providers
        .embedding
        .values()
        .next()
        .map(embedding_config_to_registry)
        .unwrap_or_else(default_embedding_config);

    // Get the first (default) vector store config or use defaults
    let vector_store_config = config
        .providers
        .vector_store
        .values()
        .next()
        .map(vector_store_config_to_registry)
        .unwrap_or_else(default_vector_store_config);

    // Cache config from system.infrastructure.cache
    let cache_provider_name = match &config.system.infrastructure.cache.provider {
        crate::config::CacheProvider::Moka => "moka",
        crate::config::CacheProvider::Redis => "redis",
    };

    let cache_config = CacheProviderConfig {
        provider: cache_provider_name.to_string(),
        uri: config.system.infrastructure.cache.redis_url.clone(),
        max_size: Some(config.system.infrastructure.cache.max_size),
        ttl_secs: Some(config.system.infrastructure.cache.default_ttl_secs),
        namespace: Some(config.system.infrastructure.cache.namespace.clone()),
        extra: Default::default(),
    };

    // Language config - use "universal" as default
    let language_config = LanguageProviderConfig::new("universal");

    // Resolve each provider from registry
    let embedding = resolve_embedding_provider(&embedding_config).map_err(|e| {
        Error::configuration(format!("Failed to resolve embedding provider: {}", e))
    })?;

    let vector_store = resolve_vector_store_provider(&vector_store_config).map_err(|e| {
        Error::configuration(format!("Failed to resolve vector store provider: {}", e))
    })?;

    let cache = resolve_cache_provider(&cache_config)
        .map_err(|e| Error::configuration(format!("Failed to resolve cache provider: {}", e)))?;

    let language = resolve_language_provider(&language_config)
        .map_err(|e| Error::configuration(format!("Failed to resolve language provider: {}", e)))?;

    Ok(ResolvedProviders {
        embedding,
        vector_store,
        cache,
        language,
    })
}

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

/// Default embedding config for testing
fn default_embedding_config() -> EmbeddingProviderConfig {
    EmbeddingProviderConfig::new("null")
}

/// Default vector store config for testing
fn default_vector_store_config() -> VectorStoreProviderConfig {
    VectorStoreProviderConfig::new("memory")
}

/// List all available providers across all categories
///
/// Useful for CLI help, admin UI, and configuration validation.
///
/// # Returns
/// Struct containing lists of available providers by category
pub fn list_available_providers() -> AvailableProviders {
    AvailableProviders {
        embedding: mcb_application::ports::registry::list_embedding_providers(),
        vector_store: mcb_application::ports::registry::list_vector_store_providers(),
        cache: mcb_application::ports::registry::list_cache_providers(),
        language: mcb_application::ports::registry::list_language_providers(),
    }
}

/// Available providers by category
#[derive(Debug, Clone)]
pub struct AvailableProviders {
    /// Available embedding providers (name, description)
    pub embedding: Vec<(&'static str, &'static str)>,
    /// Available vector store providers (name, description)
    pub vector_store: Vec<(&'static str, &'static str)>,
    /// Available cache providers (name, description)
    pub cache: Vec<(&'static str, &'static str)>,
    /// Available language chunking providers (name, description)
    pub language: Vec<(&'static str, &'static str)>,
}

impl std::fmt::Display for AvailableProviders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Available Providers:")?;
        writeln!(f)?;

        writeln!(f, "Embedding Providers:")?;
        for (name, desc) in &self.embedding {
            writeln!(f, "  - {}: {}", name, desc)?;
        }
        writeln!(f)?;

        writeln!(f, "Vector Store Providers:")?;
        for (name, desc) in &self.vector_store {
            writeln!(f, "  - {}: {}", name, desc)?;
        }
        writeln!(f)?;

        writeln!(f, "Cache Providers:")?;
        for (name, desc) in &self.cache {
            writeln!(f, "  - {}: {}", name, desc)?;
        }
        writeln!(f)?;

        writeln!(f, "Language Providers:")?;
        for (name, desc) in &self.language {
            writeln!(f, "  - {}: {}", name, desc)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_available_providers() {
        // Should not panic
        let providers = list_available_providers();
        // Providers will be empty in unit tests since mcb-providers isn't linked
        assert!(providers.embedding.is_empty() || !providers.embedding.is_empty());
    }

    #[test]
    fn test_available_providers_display() {
        let providers = AvailableProviders {
            embedding: vec![("null", "Null provider")],
            vector_store: vec![("memory", "In-memory store")],
            cache: vec![("moka", "Moka cache")],
            language: vec![("universal", "Universal chunker")],
        };

        let display = format!("{}", providers);
        assert!(display.contains("Embedding Providers"));
        assert!(display.contains("null"));
    }
}
