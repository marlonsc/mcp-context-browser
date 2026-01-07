//! Factory implementations for creating providers

// Standard library imports
use std::sync::Arc;

// External crate imports
use async_trait::async_trait;

// Internal imports - core types
use crate::core::{
    error::{Error, Result},
    types::{EmbeddingConfig, VectorStoreConfig},
};
use crate::config::VectorStoreProviderConfig;
use crate::providers::{EmbeddingProvider, VectorStoreProvider};

// Internal imports - embedding providers
use crate::providers::embedding::{
    fastembed::FastEmbedProvider,
    gemini::GeminiEmbeddingProvider,
    null::NullEmbeddingProvider,
    ollama::OllamaEmbeddingProvider,
    openai::OpenAIEmbeddingProvider,
    voyageai::VoyageAIEmbeddingProvider,
};

// Internal imports - vector store providers
use crate::providers::vector_store::milvus::MilvusVectorStoreProvider;
use crate::providers::vector_store::InMemoryVectorStoreProvider;

/// Provider factory trait
#[async_trait]
pub trait ProviderFactory: Send + Sync {
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>>;
    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>>;
    fn supported_embedding_providers(&self) -> Vec<String>;
    fn supported_vector_store_providers(&self) -> Vec<String>;
}

/// Default provider factory implementation
pub struct DefaultProviderFactory;

impl DefaultProviderFactory {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProviderFactory for DefaultProviderFactory {
    async fn create_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        match config.provider.as_str() {
            "openai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("OpenAI API key required"))?;
                Ok(Arc::new(OpenAIEmbeddingProvider::with_timeout(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    std::time::Duration::from_secs(30),
                )?))
            }
            "ollama" => Ok(Arc::new(OllamaEmbeddingProvider::with_timeout(
                config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string()),
                config.model.clone(),
                std::time::Duration::from_secs(30),
            )?)),
            "voyageai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("VoyageAI API key required"))?;
                Ok(Arc::new(VoyageAIEmbeddingProvider::new(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                )))
            }
            "gemini" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("Gemini API key required"))?;
                Ok(Arc::new(GeminiEmbeddingProvider::with_timeout(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    std::time::Duration::from_secs(30),
                )?))
            }
            "mock" => Ok(Arc::new(NullEmbeddingProvider::new())),
            "fastembed" => Ok(Arc::new(FastEmbedProvider::new()?)),
            _ => Err(Error::config(format!(
                "Unsupported embedding provider: {}",
                config.provider
            ))),
        }
    }

    async fn create_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        match config.provider.as_str() {
            "in-memory" => Ok(Arc::new(InMemoryVectorStoreProvider::new())),
            "filesystem" => {
                use crate::providers::vector_store::filesystem::{FilesystemVectorStore, FilesystemVectorStoreConfig};
                let base_path = config.address.as_ref()
                    .map(|p| std::path::PathBuf::from(p))
                    .unwrap_or_else(|| std::path::PathBuf::from("./data/vectors"));
                let fs_config = FilesystemVectorStoreConfig {
                    base_path,
                    dimensions: config.dimensions.unwrap_or(1536),
                    ..Default::default()
                };
                Ok(Arc::new(FilesystemVectorStore::new(fs_config).await?))
            }
            "milvus" => {
                let address = config
                    .address
                    .as_ref()
                    .ok_or_else(|| Error::config("Milvus address required"))?;
                Ok(Arc::new(
                    MilvusVectorStoreProvider::new(address.clone(), config.token.clone()).await?,
                ))
            }
            _ => Err(Error::config(format!(
                "Unsupported vector store provider: {}",
                config.provider
            ))),
        }
    }

    fn supported_embedding_providers(&self) -> Vec<String> {
        vec![
            "openai".to_string(),
            "ollama".to_string(),
            "voyageai".to_string(),
            "gemini".to_string(),
            "fastembed".to_string(),
            "mock".to_string(),
        ]
    }

    fn supported_vector_store_providers(&self) -> Vec<String> {
        vec!["in-memory".to_string(), "filesystem".to_string(), "edgevec".to_string(), "milvus".to_string()]
    }
}

impl Default for DefaultProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Service provider for dependency injection
pub struct ServiceProvider {
    factory: DefaultProviderFactory,
    registry: crate::di::registry::ProviderRegistry,
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            factory: DefaultProviderFactory::new(),
            registry: crate::di::registry::ProviderRegistry::new(),
        }
    }

    pub async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        // First try to get from registry
        if let Ok(provider) = self.registry.get_embedding_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self.factory.create_embedding_provider(config).await?;
        self.registry.register_embedding_provider(&config.provider, Arc::clone(&provider))?;

        Ok(provider)
    }

    pub async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        // First try to get from registry
        if let Ok(provider) = self.registry.get_vector_store_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self.factory.create_vector_store_provider(config).await?;
        self.registry.register_vector_store_provider(&config.provider, Arc::clone(&provider))?;

        Ok(provider)
    }

    pub fn registry(&self) -> &crate::di::registry::ProviderRegistry {
        &self.registry
    }

    /// Register an embedding provider directly
    pub fn register_embedding_provider(
        &self,
        name: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        self.registry.register_embedding_provider(name, provider)
    }

    /// Register a vector store provider directly
    pub fn register_vector_store_provider(
        &self,
        name: &str,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        self.registry.register_vector_store_provider(name, provider)
    }

    /// List all registered providers
    pub fn list_providers(&self) -> (Vec<String>, Vec<String>) {
        (
            self.registry.list_embedding_providers(),
            self.registry.list_vector_store_providers(),
        )
    }
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self::new()
    }
}
