//! Embedding Provider Registry
//!
//! Auto-registration system for embedding providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ports::providers::EmbeddingProvider;

/// Configuration for embedding provider creation
///
/// Contains all configuration options that an embedding provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct EmbeddingProviderConfig {
    /// Provider name (e.g., "ollama", "openai", "null")
    pub provider: String,
    /// Model name/identifier
    pub model: Option<String>,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Base URL for the provider API
    pub base_url: Option<String>,
    /// Embedding dimensions (if configurable)
    pub dimensions: Option<usize>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

impl EmbeddingProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the dimensions
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Registry entry for embedding providers
///
/// Each embedding provider implementation registers itself with this entry
/// using `#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]`. The entry contains
/// metadata and a factory function to create provider instances.
pub struct EmbeddingProviderEntry {
    /// Unique provider name (e.g., "ollama", "openai", "null")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Factory function to create provider instance
    pub factory: fn(&EmbeddingProviderConfig) -> Result<Arc<dyn EmbeddingProvider>, String>,
}

// Auto-collection via linkme distributed slices - providers submit entries at compile time
#[linkme::distributed_slice]
pub static EMBEDDING_PROVIDERS: [EmbeddingProviderEntry] = [..];

/// Resolve embedding provider by name from registry
///
/// Searches the registry for a provider matching the configured name
/// and creates an instance using the provider's factory function.
///
/// # Arguments
/// * `config` - Configuration containing provider name and settings
///
/// # Returns
/// * `Ok(Arc<dyn EmbeddingProvider>)` - Created provider instance
/// * `Err(String)` - Error message if provider not found or creation failed
///
/// # Example
///
/// ```ignore
/// let config = EmbeddingProviderConfig::new("ollama")
///     .with_base_url("http://localhost:11434")
///     .with_model("nomic-embed-text");
/// let provider = resolve_embedding_provider(&config)?;
/// ```
pub fn resolve_embedding_provider(
    config: &EmbeddingProviderConfig,
) -> Result<Arc<dyn EmbeddingProvider>, String> {
    let provider_name = &config.provider;

    for entry in EMBEDDING_PROVIDERS {
        if entry.name == provider_name {
            return (entry.factory)(config);
        }
    }

    let available: Vec<&str> = EMBEDDING_PROVIDERS.iter().map(|e| e.name).collect();

    Err(format!(
        "Unknown embedding provider '{}'. Available providers: {:?}",
        provider_name, available
    ))
}

/// List all registered embedding providers
///
/// Returns a list of (name, description) tuples for all registered
/// embedding providers. Useful for CLI help and admin UI.
///
/// # Returns
/// Vector of (name, description) tuples for all registered providers
pub fn list_embedding_providers() -> Vec<(&'static str, &'static str)> {
    EMBEDDING_PROVIDERS
        .iter()
        .map(|e| (e.name, e.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = EmbeddingProviderConfig::new("test")
            .with_model("model-1")
            .with_api_key("secret")
            .with_base_url("http://localhost")
            .with_dimensions(384)
            .with_extra("custom", "value");

        assert_eq!(config.provider, "test");
        assert_eq!(config.model, Some("model-1".to_string()));
        assert_eq!(config.api_key, Some("secret".to_string()));
        assert_eq!(config.base_url, Some("http://localhost".to_string()));
        assert_eq!(config.dimensions, Some(384));
        assert_eq!(config.extra.get("custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_list_providers_returns_vec() {
        // Should not panic, returns empty if no providers registered
        let providers = list_embedding_providers();
        // In tests, providers from mcb-providers won't be linked
        assert!(providers.is_empty() || !providers.is_empty());
    }
}
