//! Provider factory traits

use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::di::registry::ProviderRegistry;
use crate::{EmbeddingConfig, Result, VectorStoreConfig};
use async_trait::async_trait;
use std::sync::Arc;

/// Provider factory trait
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>>;
    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
    fn supported_embedding_providers(&self) -> Vec<String>;
    fn supported_vector_store_providers(&self) -> Vec<String>;
}

/// Service provider interface
#[async_trait]
pub trait ServiceProviderInterface: shaku::Interface + Send + Sync {
    fn registry(&self) -> &ProviderRegistry;
    fn list_providers(&self) -> (Vec<String>, Vec<String>);
    fn register_embedding_provider(
        &self,
        name: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()>;
    fn register_vector_store_provider(
        &self,
        name: &str,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()>;
    async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>>;
    async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
}
