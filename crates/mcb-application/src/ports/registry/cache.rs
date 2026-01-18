//! Cache Provider Registry
//!
//! Auto-registration system for cache providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ports::providers::cache::CacheProvider;

/// Configuration for cache provider creation
///
/// Contains all configuration options that a cache provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct CacheProviderConfig {
    /// Provider name (e.g., "moka", "redis", "null")
    pub provider: String,
    /// Connection URI (for distributed caches)
    pub uri: Option<String>,
    /// Maximum cache size (entries or bytes depending on provider)
    pub max_size: Option<usize>,
    /// Default TTL in seconds
    pub ttl_secs: Option<u64>,
    /// Namespace prefix for keys
    pub namespace: Option<String>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

impl CacheProviderConfig {
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

    /// Set the max size
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = Some(max_size);
        self
    }

    /// Set the TTL in seconds
    pub fn with_ttl_secs(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = Some(ttl_secs);
        self
    }

    /// Set the namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Registry entry for cache providers
///
/// Each cache provider implementation registers itself with this entry
/// using `#[linkme::distributed_slice(CACHE_PROVIDERS)]`. The entry contains
/// metadata and a factory function to create provider instances.
pub struct CacheProviderEntry {
    /// Unique provider name (e.g., "moka", "redis", "null")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Factory function to create provider instance
    pub factory: fn(&CacheProviderConfig) -> Result<Arc<dyn CacheProvider>, String>,
}

// Auto-collection via linkme distributed slices - providers submit entries at compile time
#[linkme::distributed_slice]
pub static CACHE_PROVIDERS: [CacheProviderEntry] = [..];

/// Resolve cache provider by name from registry
///
/// Searches the registry for a provider matching the configured name
/// and creates an instance using the provider's factory function.
///
/// # Arguments
/// * `config` - Configuration containing provider name and settings
///
/// # Returns
/// * `Ok(Arc<dyn CacheProvider>)` - Created provider instance
/// * `Err(String)` - Error message if provider not found or creation failed
pub fn resolve_cache_provider(
    config: &CacheProviderConfig,
) -> Result<Arc<dyn CacheProvider>, String> {
    let provider_name = &config.provider;

    for entry in CACHE_PROVIDERS {
        if entry.name == provider_name {
            return (entry.factory)(config);
        }
    }

    let available: Vec<&str> = CACHE_PROVIDERS.iter().map(|e| e.name).collect();

    Err(format!(
        "Unknown cache provider '{}'. Available providers: {:?}",
        provider_name, available
    ))
}

/// List all registered cache providers
///
/// Returns a list of (name, description) tuples for all registered
/// cache providers. Useful for CLI help and admin UI.
pub fn list_cache_providers() -> Vec<(&'static str, &'static str)> {
    CACHE_PROVIDERS
        .iter()
        .map(|e| (e.name, e.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = CacheProviderConfig::new("redis")
            .with_uri("redis://localhost:6379")
            .with_max_size(10000)
            .with_ttl_secs(3600)
            .with_namespace("mcb");

        assert_eq!(config.provider, "redis");
        assert_eq!(config.uri, Some("redis://localhost:6379".to_string()));
        assert_eq!(config.max_size, Some(10000));
        assert_eq!(config.ttl_secs, Some(3600));
        assert_eq!(config.namespace, Some("mcb".to_string()));
    }
}
