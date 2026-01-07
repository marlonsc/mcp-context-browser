//! Factory implementations for creating providers

use crate::core::{
    error::{Error, Result},
    types::{EmbeddingConfig, VectorStoreConfig, VectorStoreProviderConfig},
};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};

// Import individual providers that exist
use crate::providers::embedding::fastembed::FastEmbedProvider;
use crate::providers::embedding::gemini::GeminiEmbeddingProvider;
use crate::providers::embedding::null::NullEmbeddingProvider;
use crate::providers::embedding::ollama::OllamaEmbeddingProvider;
use crate::providers::embedding::openai::OpenAIEmbeddingProvider;
use crate::providers::embedding::voyageai::VoyageAIEmbeddingProvider;
use crate::providers::vector_store::milvus::MilvusVectorStoreProvider;
use crate::providers::vector_store::InMemoryVectorStoreProvider;
use async_trait::async_trait;
use std::sync::Arc;

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
            "edgevec" => {
                use crate::providers::vector_store::edgevec::{EdgeVecVectorStoreProvider, EdgeVecConfig, MetricType, HnswConfig};

                // Parse EdgeVec-specific config from the provider config
                let edgevec_config = if let Some(VectorStoreProviderConfig::EdgeVec {
                    max_vectors: _,
                    collection,
                    hnsw_m: _,
                    hnsw_ef_construction: _,
                    distance_metric: _,
                    use_quantization,
                }) = &config.provider_config {
                    EdgeVecConfig {
                        dimensions: config.dimensions.unwrap_or(1536),
                        hnsw_config: HnswConfig {
                            m: hnsw_m.unwrap_or(16) as u32,
                            m0: (hnsw_m.unwrap_or(16) * 2) as u32, // m0 is typically 2*m
                            ef_construction: hnsw_ef_construction.unwrap_or(200) as u32,
                            ef_search: 64, // Default search parameter
                        },
                        metric: match distance_metric.as_deref() {
                            Some("l2_squared") | Some("euclidean") => MetricType::L2Squared,
                            Some("dot_product") => MetricType::DotProduct,
                            _ => MetricType::Cosine, // Default
                        },
                        use_quantization: use_quantization.unwrap_or(false),
                        quantizer_config: Default::default(),
                    }
                } else {
                    EdgeVecConfig {
                        dimensions: config.dimensions.unwrap_or(1536),
                        ..Default::default()
                    }
                };

                let collection = collection.clone().unwrap_or_else(|| "default".to_string());
                Ok(Arc::new(EdgeVecVectorStoreProvider::with_collection(edgevec_config, collection)?))
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
