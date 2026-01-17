//! Configuration Provider Port
//!
//! Port for configuration management of providers.

use mcb_domain::error::Result;
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use shaku::Interface;

/// Provider configuration manager interface
///
/// This port defines the contract for managing provider configurations
/// including embedding and vector store providers.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::ProviderConfigManagerInterface;
///
/// // List available embedding providers
/// let providers = config_manager.list_embedding_providers();
/// for name in providers {
///     if let Ok(config) = config_manager.get_embedding_config(&name) {
///         println!("{}: model={}", name, config.model);
///     }
/// }
///
/// // Check if a specific provider exists
/// if config_manager.has_embedding_provider("openai") {
///     let config = config_manager.get_embedding_config("openai")?;
///     // Use config...
/// }
/// ```
#[async_trait::async_trait]
pub trait ProviderConfigManagerInterface: Interface + Send + Sync {
    /// Get embedding provider configuration by name
    fn get_embedding_config(&self, name: &str) -> Result<&EmbeddingConfig>;

    /// Get vector store provider configuration by name
    fn get_vector_store_config(&self, name: &str) -> Result<&VectorStoreConfig>;

    /// List all embedding provider names
    fn list_embedding_providers(&self) -> Vec<String>;

    /// List all vector store provider names
    fn list_vector_store_providers(&self) -> Vec<String>;

    /// Check if an embedding provider is configured
    fn has_embedding_provider(&self, name: &str) -> bool {
        self.list_embedding_providers().contains(&name.to_string())
    }

    /// Check if a vector store provider is configured
    fn has_vector_store_provider(&self, name: &str) -> bool {
        self.list_vector_store_providers().contains(&name.to_string())
    }

    /// Get default embedding provider configuration
    fn get_default_embedding_config(&self) -> Option<&EmbeddingConfig> {
        let providers = self.list_embedding_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_embedding_config(&providers[0]).ok()
        }
    }

    /// Get default vector store provider configuration
    fn get_default_vector_store_config(&self) -> Option<&VectorStoreConfig> {
        let providers = self.list_vector_store_providers();
        if providers.is_empty() {
            None
        } else {
            self.get_vector_store_config(&providers[0]).ok()
        }
    }
}