//! Provider Factory
//!
//! Factory for creating embedding and vector store providers based on configuration.
//! Follows the Factory pattern to abstract provider instantiation.
//!
//! **IMPORTANT: All provider implementations come from mcb-providers crate.**
//! This factory only handles wiring and dependency injection.

use std::sync::Arc;
use std::time::Duration;

use mcb_application::ports::providers::{CryptoProvider, EmbeddingProvider, VectorStoreProvider};
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use reqwest::Client;

// Provider implementations from mcb-providers (SOURCE OF TRUTH)
use mcb_providers::embedding::{
    FastEmbedProvider, GeminiEmbeddingProvider, NullEmbeddingProvider, OllamaEmbeddingProvider,
    OpenAIEmbeddingProvider, VoyageAIEmbeddingProvider,
};
use mcb_providers::vector_store::{
    EncryptedVectorStoreProvider, InMemoryVectorStoreProvider, NullVectorStoreProvider,
};

// Infrastructure dependencies for wiring
use crate::constants::OLLAMA_DEFAULT_PORT;

/// Known embedding provider names
pub mod embedding_providers {
    pub const OPENAI: &str = "openai";
    pub const VOYAGEAI: &str = "voyageai";
    pub const OLLAMA: &str = "ollama";
    pub const GEMINI: &str = "gemini";
    pub const FASTEMBED: &str = "fastembed";
    pub const NULL: &str = "null";
}

/// Known vector store provider names
pub mod vector_store_providers {
    pub const IN_MEMORY: &str = "in_memory";
    pub const MEMORY: &str = "memory";
    pub const ENCRYPTED: &str = "encrypted";
    pub const FILESYSTEM: &str = "filesystem";
    pub const NULL: &str = "null";
}

/// Factory for creating embedding providers
pub struct EmbeddingProviderFactory;

impl EmbeddingProviderFactory {
    /// Create an embedding provider based on configuration
    ///
    /// The `http_client` parameter is optional. If not provided, a default
    /// client will be created for providers that need HTTP access.
    pub fn create(
        config: &EmbeddingConfig,
        http_client: Option<Client>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let provider_name = config.provider.to_lowercase();

        match provider_name.as_str() {
            embedding_providers::NULL => Ok(Arc::new(NullEmbeddingProvider::new())),
            embedding_providers::OPENAI => Self::create_openai(config, http_client),
            embedding_providers::VOYAGEAI => Self::create_voyageai(config, http_client),
            embedding_providers::OLLAMA => Self::create_ollama(config, http_client),
            embedding_providers::GEMINI => Self::create_gemini(config, http_client),
            embedding_providers::FASTEMBED => Ok(Arc::new(FastEmbedProvider::new()?)),
            _ => Err(Error::Configuration {
                message: format!("Unknown embedding provider: {}", config.provider),
                source: None,
            }),
        }
    }

    fn create_openai(
        config: &EmbeddingConfig,
        http_client: Option<Client>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let client = Self::require_http_client(http_client)?;
        let api_key = Self::require_api_key(config, "OpenAI")?;
        Ok(Arc::new(OpenAIEmbeddingProvider::new(
            api_key,
            config.base_url.clone(),
            config.model.clone(),
            Duration::from_secs(30),
            client,
        )))
    }

    fn create_voyageai(
        config: &EmbeddingConfig,
        http_client: Option<Client>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let client = Self::require_http_client(http_client)?;
        let api_key = Self::require_api_key(config, "VoyageAI")?;
        Ok(Arc::new(VoyageAIEmbeddingProvider::new(
            api_key,
            config.base_url.clone(),
            config.model.clone(),
            client,
        )))
    }

    fn create_ollama(
        config: &EmbeddingConfig,
        http_client: Option<Client>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let client = Self::require_http_client(http_client)?;
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| format!("http://localhost:{}", OLLAMA_DEFAULT_PORT));
        Ok(Arc::new(OllamaEmbeddingProvider::new(
            base_url,
            config.model.clone(),
            Duration::from_secs(30),
            client,
        )))
    }

    fn create_gemini(
        config: &EmbeddingConfig,
        http_client: Option<Client>,
    ) -> Result<Arc<dyn EmbeddingProvider>> {
        let client = Self::require_http_client(http_client)?;
        let api_key = Self::require_api_key(config, "Gemini")?;
        Ok(Arc::new(GeminiEmbeddingProvider::new(
            api_key,
            config.base_url.clone(),
            config.model.clone(),
            Duration::from_secs(30),
            client,
        )))
    }

    fn require_http_client(client: Option<Client>) -> Result<Client> {
        client.map_or_else(Self::create_default_http_client, Ok)
    }

    fn require_api_key(config: &EmbeddingConfig, provider: &str) -> Result<String> {
        config.api_key.clone().ok_or_else(|| Error::Configuration {
            message: format!("API key required for {} provider", provider),
            source: None,
        })
    }

    /// Create a default null provider (for testing/development)
    pub fn create_null() -> Arc<dyn EmbeddingProvider> {
        Arc::new(NullEmbeddingProvider::new())
    }

    /// Create default HTTP client for providers that need it
    fn create_default_http_client() -> Result<Client> {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to create HTTP client: {}", e),
                source: Some(Box::new(e)),
            })
    }
}

/// Factory for creating vector store providers
pub struct VectorStoreProviderFactory;

impl VectorStoreProviderFactory {
    /// Create a vector store provider based on configuration
    ///
    /// # Arguments
    /// * `config` - Vector store provider configuration
    /// * `crypto_provider` - Optional crypto provider for encrypted providers
    ///
    /// # Returns
    /// * `Result<Arc<dyn VectorStoreProvider>>` - The configured provider
    pub fn create(
        config: &VectorStoreConfig,
        crypto_provider: Option<Arc<dyn CryptoProvider>>,
    ) -> Result<Arc<dyn VectorStoreProvider>> {
        let provider_name = config.provider.to_lowercase();

        match provider_name.as_str() {
            vector_store_providers::NULL => Ok(Arc::new(NullVectorStoreProvider::new())),

            vector_store_providers::IN_MEMORY | vector_store_providers::MEMORY => {
                Ok(Arc::new(InMemoryVectorStoreProvider::new()))
            }

            vector_store_providers::ENCRYPTED => {
                let crypto = crypto_provider.ok_or_else(|| Error::Configuration {
                    message: "CryptoProvider required for encrypted vector store".to_string(),
                    source: None,
                })?;

                // Use in-memory as the underlying store for encrypted
                let inner = InMemoryVectorStoreProvider::new();
                Ok(Arc::new(EncryptedVectorStoreProvider::new(inner, crypto)))
            }

            vector_store_providers::FILESYSTEM => {
                // NOTE: Filesystem provider uses in-memory storage as placeholder
                // Actual filesystem-backed vector store planned for future release
                Ok(Arc::new(InMemoryVectorStoreProvider::new()))
            }

            _ => Err(Error::Configuration {
                message: format!("Unknown vector store provider: {}", config.provider),
                source: None,
            }),
        }
    }

    /// Create a default in-memory provider
    pub fn create_in_memory() -> Arc<dyn VectorStoreProvider> {
        Arc::new(InMemoryVectorStoreProvider::new())
    }

    /// Create a null provider (for testing)
    pub fn create_null() -> Arc<dyn VectorStoreProvider> {
        Arc::new(NullVectorStoreProvider::new())
    }
}
