//! Language Chunking Provider Registry
//!
//! Auto-registration system for language chunking providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ports::providers::LanguageChunkingProvider;

/// Configuration for language chunking provider creation
///
/// Contains all configuration options that a language chunking provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct LanguageProviderConfig {
    /// Provider name (e.g., "universal", "treesitter", "null")
    pub provider: String,
    /// Maximum chunk size in characters
    pub max_chunk_size: Option<usize>,
    /// Minimum chunk size in characters
    pub min_chunk_size: Option<usize>,
    /// Chunk overlap in characters
    pub overlap: Option<usize>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

impl LanguageProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the max chunk size
    pub fn with_max_chunk_size(mut self, size: usize) -> Self {
        self.max_chunk_size = Some(size);
        self
    }

    /// Set the min chunk size
    pub fn with_min_chunk_size(mut self, size: usize) -> Self {
        self.min_chunk_size = Some(size);
        self
    }

    /// Set the overlap
    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.overlap = Some(overlap);
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// Registry entry for language chunking providers
///
/// Each language chunking provider implementation registers itself with this entry
/// using `#[linkme::distributed_slice(LANGUAGE_PROVIDERS)]`. The entry contains
/// metadata and a factory function to create provider instances.
pub struct LanguageProviderEntry {
    /// Unique provider name (e.g., "universal", "treesitter", "null")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Factory function to create provider instance
    pub factory: fn(&LanguageProviderConfig) -> Result<Arc<dyn LanguageChunkingProvider>, String>,
}

// Auto-collection via linkme distributed slices - providers submit entries at compile time
#[linkme::distributed_slice]
pub static LANGUAGE_PROVIDERS: [LanguageProviderEntry] = [..];

/// Resolve language chunking provider by name from registry
///
/// Searches the registry for a provider matching the configured name
/// and creates an instance using the provider's factory function.
///
/// # Arguments
/// * `config` - Configuration containing provider name and settings
///
/// # Returns
/// * `Ok(Arc<dyn LanguageChunkingProvider>)` - Created provider instance
/// * `Err(String)` - Error message if provider not found or creation failed
pub fn resolve_language_provider(
    config: &LanguageProviderConfig,
) -> Result<Arc<dyn LanguageChunkingProvider>, String> {
    let provider_name = &config.provider;

    for entry in LANGUAGE_PROVIDERS {
        if entry.name == provider_name {
            return (entry.factory)(config);
        }
    }

    let available: Vec<&str> = LANGUAGE_PROVIDERS.iter().map(|e| e.name).collect();

    Err(format!(
        "Unknown language provider '{}'. Available providers: {:?}",
        provider_name, available
    ))
}

/// List all registered language chunking providers
///
/// Returns a list of (name, description) tuples for all registered
/// language chunking providers. Useful for CLI help and admin UI.
pub fn list_language_providers() -> Vec<(&'static str, &'static str)> {
    LANGUAGE_PROVIDERS
        .iter()
        .map(|e| (e.name, e.description))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = LanguageProviderConfig::new("universal")
            .with_max_chunk_size(4096)
            .with_min_chunk_size(100)
            .with_overlap(50);

        assert_eq!(config.provider, "universal");
        assert_eq!(config.max_chunk_size, Some(4096));
        assert_eq!(config.min_chunk_size, Some(100));
        assert_eq!(config.overlap, Some(50));
    }
}
