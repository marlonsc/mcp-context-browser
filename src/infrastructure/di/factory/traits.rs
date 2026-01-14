//! Provider factory traits

use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::di::registry::ProviderRegistry;
use crate::{EmbeddingConfig, Result, VectorStoreConfig};
use async_trait::async_trait;
use std::sync::Arc;

/// Provider factory trait
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    /// Create an embedding provider instance based on configuration
    ///
    /// # Arguments
    /// * `config` - Embedding provider configuration specifying the provider type and settings
    /// * `http_client` - HTTP client for external API communication
    ///
    /// # Returns
    /// A configured embedding provider instance wrapped in Arc
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>>;

    /// Create a vector store provider instance based on configuration
    ///
    /// # Arguments
    /// * `config` - Vector store provider configuration specifying the provider type and settings
    ///
    /// # Returns
    /// A configured vector store provider instance wrapped in Arc
    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
    /// Get list of supported embedding provider names
    fn supported_embedding_providers(&self) -> Vec<String>;
    /// Get list of supported vector store provider names
    fn supported_vector_store_providers(&self) -> Vec<String>;
}

/// Service provider interface
#[async_trait]
pub trait ServiceProviderInterface: shaku::Interface + Send + Sync {
    /// Get access to the provider registry
    fn registry(&self) -> &ProviderRegistry;
    /// List all registered providers (embedding, vector_store)
    fn list_providers(&self) -> (Vec<String>, Vec<String>);
    /// Register an embedding provider with the given name
    fn register_embedding_provider(
        &self,
        name: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()>;
    /// Register a vector store provider with the given name
    fn register_vector_store_provider(
        &self,
        name: &str,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()>;
    /// Remove an embedding provider by name
    fn remove_embedding_provider(&self, name: &str) -> Result<()>;
    /// Remove a vector store provider by name
    fn remove_vector_store_provider(&self, name: &str) -> Result<()>;
    /// Create an embedding provider from configuration
    async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>>;
    /// Create a vector store provider from configuration
    async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
}
