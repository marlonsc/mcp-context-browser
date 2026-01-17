//! Vector Store Provider Registry
//!
//! Auto-registration system for vector store providers.
//! Providers register themselves via `inventory::submit!` and are
//! discovered at runtime via `inventory::iter`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ports::providers::VectorStoreProvider;

/// Configuration for vector store provider creation
///
/// Contains all configuration options that a vector store provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct VectorStoreProviderConfig {
    /// Provider name (e.g., "milvus", "memory", "null")
    pub provider: String,
    /// Connection URI or path
    pub uri: Option<String>,
    /// Collection/index name
    pub collection: Option<String>,
    /// Embedding dimensions
    pub dimensions: Option<usize>,
    /// API key or token for authentication
    pub api_key: Option<String>,
    /// Enable encryption
    pub encrypted: Option<bool>,
    /// Encryption key (if encrypted)
    pub encryption_key: Option<String>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

impl VectorStoreProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the URI
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Set the collection name
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self {
        self.collection = Some(collection.into());
        self
    }

    /// Set the dimensions
    pub fn with_dimensions(mut self, dimensions: usize) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Enable encryption
    pub fn with_encryption(mut self, key: impl Into<String>) -> Self {
        self.encrypted = Some(true);
        self.encryption_key = Some(key.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Registry entry for vector store providers
///
/// Each vector store provider implementation registers itself with this entry
/// using `inventory::submit!`. The entry contains metadata and a factory
/// function to create provider instances.
pub struct VectorStoreProviderEntry {
    /// Unique provider name (e.g., "milvus", "memory", "null")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Factory function to create provider instance
    pub factory: fn(&VectorStoreProviderConfig) -> Result<Arc<dyn VectorStoreProvider>, String>,
}

// Auto-collection via inventory - providers submit entries at compile time
inventory::collect!(VectorStoreProviderEntry);

/// Resolve vector store provider by name from registry
///
/// Searches the registry for a provider matching the configured name
/// and creates an instance using the provider's factory function.
///
/// # Arguments
/// * `config` - Configuration containing provider name and settings
///
/// # Returns
/// * `Ok(Arc<dyn VectorStoreProvider>)` - Created provider instance
/// * `Err(String)` - Error message if provider not found or creation failed
pub fn resolve_vector_store_provider(
    config: &VectorStoreProviderConfig,
) -> Result<Arc<dyn VectorStoreProvider>, String> {
    let provider_name = &config.provider;

    for entry in inventory::iter::<VectorStoreProviderEntry> {
        if entry.name == provider_name {
            return (entry.factory)(config);
        }
    }

    // List available providers for helpful error message
    let available: Vec<&str> = inventory::iter::<VectorStoreProviderEntry>
        .map(|e| e.name)
        .collect();

    Err(format!(
        "Unknown vector store provider '{}'. Available providers: {:?}",
        provider_name, available
    ))
}

/// List all registered vector store providers
///
/// Returns a list of (name, description) tuples for all registered
/// vector store providers. Useful for CLI help and admin UI.
pub fn list_vector_store_providers() -> Vec<(&'static str, &'static str)> {
    inventory::iter::<VectorStoreProviderEntry>
        .map(|e| (e.name, e.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = VectorStoreProviderConfig::new("milvus")
            .with_uri("http://localhost:19530")
            .with_collection("embeddings")
            .with_dimensions(384)
            .with_encryption("secret-key");

        assert_eq!(config.provider, "milvus");
        assert_eq!(config.uri, Some("http://localhost:19530".to_string()));
        assert_eq!(config.collection, Some("embeddings".to_string()));
        assert_eq!(config.dimensions, Some(384));
        assert_eq!(config.encrypted, Some(true));
    }
}
