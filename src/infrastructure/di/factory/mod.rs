//! Factory implementations for creating providers

use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::di::registry::{ProviderRegistry, ProviderRegistryTrait};
use crate::{EmbeddingConfig, Error, Result, VectorStoreConfig};

// Import individual providers that exist
use crate::adapters::providers::embedding::fastembed::FastEmbedProvider;
use crate::adapters::providers::embedding::gemini::GeminiEmbeddingProvider;
use crate::adapters::providers::embedding::null::NullEmbeddingProvider;
use crate::adapters::providers::embedding::ollama::OllamaEmbeddingProvider;
use crate::adapters::providers::embedding::openai::OpenAIEmbeddingProvider;
use crate::adapters::providers::embedding::voyageai::VoyageAIEmbeddingProvider;
use crate::adapters::providers::vector_store::milvus::MilvusVectorStoreProvider;
use crate::adapters::providers::vector_store::InMemoryVectorStoreProvider;
use async_trait::async_trait;
use shaku::Component;
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
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        match config.provider.to_lowercase().as_str() {
            "openai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("OpenAI API key required"))?;
                Ok(Arc::new(OpenAIEmbeddingProvider::with_http_client(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    std::time::Duration::from_secs(30),
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            "ollama" => Ok(Arc::new(OllamaEmbeddingProvider::with_http_client(
                config
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string()),
                config.model.clone(),
                std::time::Duration::from_secs(30),
                http_client,
            )) as Arc<dyn EmbeddingProvider>),
            "voyageai" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("VoyageAI API key required"))?;
                Ok(Arc::new(VoyageAIEmbeddingProvider::with_http_client(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            "gemini" => {
                let api_key = config
                    .api_key
                    .as_ref()
                    .ok_or_else(|| Error::config("Gemini API key required"))?;
                Ok(Arc::new(GeminiEmbeddingProvider::with_http_client(
                    api_key.clone(),
                    config.base_url.clone(),
                    config.model.clone(),
                    std::time::Duration::from_secs(30),
                    http_client,
                )) as Arc<dyn EmbeddingProvider>)
            }
            "fastembed" => Ok(Arc::new(FastEmbedProvider::new()?) as Arc<dyn EmbeddingProvider>),
            "mock" => Ok(Arc::new(NullEmbeddingProvider::new()) as Arc<dyn EmbeddingProvider>),
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
        match config.provider.to_lowercase().as_str() {
            "in-memory" => {
                Ok(Arc::new(InMemoryVectorStoreProvider::new()) as Arc<dyn VectorStoreProvider>)
            }
            "filesystem" => {
                use crate::adapters::providers::vector_store::filesystem::{
                    FilesystemVectorStore, FilesystemVectorStoreConfig,
                };
                let base_path = config
                    .address
                    .as_ref()
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("./data/vectors"));
                let fs_config = FilesystemVectorStoreConfig {
                    base_path,
                    dimensions: config.dimensions.unwrap_or(1536),
                    ..Default::default()
                };
                Ok(Arc::new(FilesystemVectorStore::new(fs_config).await?)
                    as Arc<dyn VectorStoreProvider>)
            }
            "milvus" => {
                let address = config
                    .address
                    .as_ref()
                    .ok_or_else(|| Error::config("Milvus address required"))?;
                Ok(Arc::new(
                    MilvusVectorStoreProvider::new(address.clone(), config.token.clone()).await?,
                ) as Arc<dyn VectorStoreProvider>)
            }
            "edgevec" => {
                use crate::adapters::providers::vector_store::edgevec::{
                    EdgeVecConfig, EdgeVecVectorStoreProvider,
                };
                let edgevec_config = EdgeVecConfig {
                    dimensions: config.dimensions.unwrap_or(1536),
                    ..Default::default()
                };
                Ok(Arc::new(EdgeVecVectorStoreProvider::new(edgevec_config)?)
                    as Arc<dyn VectorStoreProvider>)
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
        vec![
            "in-memory".to_string(),
            "filesystem".to_string(),
            "milvus".to_string(),
            "edgevec".to_string(),
        ]
    }
}

impl Default for DefaultProviderFactory {
    fn default() -> Self {
        Self::new()
    }
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

/// Service provider for dependency injection
#[derive(Component)]
#[shaku(interface = ServiceProviderInterface)]
pub struct ServiceProvider {
    #[shaku(default = DefaultProviderFactory::new())]
    factory: DefaultProviderFactory,
    #[shaku(default = ProviderRegistry::new())]
    registry: ProviderRegistry,
}

#[async_trait]
impl ServiceProviderInterface for ServiceProvider {
    fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    fn list_providers(&self) -> (Vec<String>, Vec<String>) {
        (
            self.registry.list_embedding_providers(),
            self.registry.list_vector_store_providers(),
        )
    }

    fn register_embedding_provider(
        &self,
        name: &str,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        self.registry
            .register_embedding_provider(name.to_string(), provider)
    }

    fn register_vector_store_provider(
        &self,
        name: &str,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        self.registry
            .register_vector_store_provider(name.to_string(), provider)
    }

    async fn get_embedding_provider(
        &self,
        config: &EmbeddingConfig,
        http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        // First try to get from registry
        if let Ok(provider) = self.registry.get_embedding_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self
            .factory
            .create_embedding_provider(config, http_client)
            .await?;
        self.registry
            .register_embedding_provider(config.provider.clone(), Arc::clone(&provider))?;

        Ok(provider)
    }

    async fn get_vector_store_provider(
        &self,
        config: &VectorStoreConfig,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        // First try to get from registry
        if let Ok(provider) = self.registry.get_vector_store_provider(&config.provider) {
            return Ok(provider);
        }

        // If not found, create via factory and register
        let provider = self.factory.create_vector_store_provider(config).await?;
        self.registry
            .register_vector_store_provider(config.provider.clone(), Arc::clone(&provider))?;

        Ok(provider)
    }
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            factory: DefaultProviderFactory::new(),
            registry: ProviderRegistry::new(),
        }
    }
}

impl Default for ServiceProvider {
    fn default() -> Self {
        Self::new()
    }
}
