//! Type-safe provider dispatch
//!
//! This module provides type-safe dispatch for creating providers based on
//! enum variants instead of string matching. This ensures:
//! - Compile-time exhaustiveness checking
//! - Invalid provider names caught at config parsing time
//! - Centralized provider creation logic
//!
//! Part of the Shaku DI optimization (Phase 1).

use crate::adapters::http_client::HttpClientProvider;
use crate::domain::error::{Error, Result};
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::domain::types::{
    EmbeddingConfig, EmbeddingProviderKind, VectorStoreConfig, VectorStoreProviderKind,
};
use crate::infrastructure::constants::{HTTP_REQUEST_TIMEOUT, OLLAMA_DEFAULT_URL};
use std::sync::Arc;

/// Dispatch embedding provider creation based on enum variant.
///
/// This function replaces the string-based match in `DefaultProviderFactory`
/// with a type-safe enum dispatch. The compiler ensures all variants are handled.
pub async fn dispatch_embedding_provider(
    kind: EmbeddingProviderKind,
    config: &EmbeddingConfig,
    http_client: Arc<dyn HttpClientProvider>,
) -> Result<Arc<dyn EmbeddingProvider>> {
    use crate::adapters::providers::embedding::*;

    match kind {
        EmbeddingProviderKind::OpenAI => {
            let api_key = config
                .api_key
                .as_ref()
                .ok_or_else(|| Error::config("OpenAI API key required"))?;
            Ok(Arc::new(OpenAIEmbeddingProvider::new(
                api_key.clone(),
                config.base_url.clone(),
                config.model.clone(),
                HTTP_REQUEST_TIMEOUT,
                http_client,
            )) as Arc<dyn EmbeddingProvider>)
        }
        EmbeddingProviderKind::Ollama => Ok(Arc::new(OllamaEmbeddingProvider::new(
            config
                .base_url
                .clone()
                .unwrap_or_else(|| OLLAMA_DEFAULT_URL.to_string()),
            config.model.clone(),
            HTTP_REQUEST_TIMEOUT,
            http_client,
        )) as Arc<dyn EmbeddingProvider>),
        EmbeddingProviderKind::VoyageAI => {
            let api_key = config
                .api_key
                .as_ref()
                .ok_or_else(|| Error::config("VoyageAI API key required"))?;
            Ok(Arc::new(VoyageAIEmbeddingProvider::new(
                api_key.clone(),
                config.base_url.clone(),
                config.model.clone(),
                http_client,
            )) as Arc<dyn EmbeddingProvider>)
        }
        EmbeddingProviderKind::Gemini => {
            let api_key = config
                .api_key
                .as_ref()
                .ok_or_else(|| Error::config("Gemini API key required"))?;
            Ok(Arc::new(GeminiEmbeddingProvider::new(
                api_key.clone(),
                config.base_url.clone(),
                config.model.clone(),
                HTTP_REQUEST_TIMEOUT,
                http_client,
            )) as Arc<dyn EmbeddingProvider>)
        }
        EmbeddingProviderKind::FastEmbed => {
            Ok(Arc::new(FastEmbedProvider::new()?) as Arc<dyn EmbeddingProvider>)
        }
    }
}

/// Dispatch vector store provider creation based on enum variant.
///
/// This function replaces the string-based match in `DefaultProviderFactory`
/// with a type-safe enum dispatch. The compiler ensures all variants are handled.
pub async fn dispatch_vector_store_provider(
    kind: VectorStoreProviderKind,
    config: &VectorStoreConfig,
) -> Result<Arc<dyn VectorStoreProvider>> {
    use crate::adapters::providers::vector_store::*;

    match kind {
        VectorStoreProviderKind::InMemory => {
            Ok(Arc::new(InMemoryVectorStoreProvider::new()) as Arc<dyn VectorStoreProvider>)
        }
        VectorStoreProviderKind::Filesystem => {
            use filesystem::{FilesystemVectorStore, FilesystemVectorStoreConfig};
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
        #[cfg(feature = "milvus")]
        VectorStoreProviderKind::Milvus => {
            let address = config
                .address
                .as_ref()
                .ok_or_else(|| Error::config("Milvus address required"))?;
            Ok(Arc::new(
                MilvusVectorStoreProvider::new(
                    address.clone(),
                    config.token.clone(),
                    config.timeout_secs,
                )
                .await?,
            ) as Arc<dyn VectorStoreProvider>)
        }
        VectorStoreProviderKind::EdgeVec => {
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
    }
}

/// Helper function to create an embedding provider from string config.
///
/// This bridges the string-based config format with type-safe dispatch.
pub async fn create_embedding_provider_from_config(
    config: &EmbeddingConfig,
    http_client: Arc<dyn HttpClientProvider>,
) -> Result<Arc<dyn EmbeddingProvider>> {
    let kind = EmbeddingProviderKind::from_string(&config.provider).ok_or_else(|| {
        Error::config(format!(
            "Unsupported embedding provider: '{}'. Supported providers: {}",
            config.provider,
            EmbeddingProviderKind::supported_providers().join(", ")
        ))
    })?;

    dispatch_embedding_provider(kind, config, http_client).await
}

/// Helper function to create a vector store provider from string config.
///
/// This bridges the string-based config format with type-safe dispatch.
pub async fn create_vector_store_provider_from_config(
    config: &VectorStoreConfig,
) -> Result<Arc<dyn VectorStoreProvider>> {
    let kind = VectorStoreProviderKind::from_string(&config.provider).ok_or_else(|| {
        Error::config(format!(
            "Unsupported vector store provider: '{}'. Supported providers: {}",
            config.provider,
            VectorStoreProviderKind::supported_providers().join(", ")
        ))
    })?;

    dispatch_vector_store_provider(kind, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_provider_kind_from_string() {
        assert_eq!(
            EmbeddingProviderKind::from_string("openai"),
            Some(EmbeddingProviderKind::OpenAI)
        );
        assert_eq!(
            EmbeddingProviderKind::from_string("OPENAI"),
            Some(EmbeddingProviderKind::OpenAI)
        );
        assert_eq!(
            EmbeddingProviderKind::from_string("fastembed"),
            Some(EmbeddingProviderKind::FastEmbed)
        );
        assert_eq!(EmbeddingProviderKind::from_string("invalid"), None);
    }

    #[test]
    fn test_vector_store_provider_kind_from_string() {
        assert_eq!(
            VectorStoreProviderKind::from_string("in-memory"),
            Some(VectorStoreProviderKind::InMemory)
        );
        assert_eq!(
            VectorStoreProviderKind::from_string("inmemory"),
            Some(VectorStoreProviderKind::InMemory)
        );
        assert_eq!(
            VectorStoreProviderKind::from_string("filesystem"),
            Some(VectorStoreProviderKind::Filesystem)
        );
        assert_eq!(VectorStoreProviderKind::from_string("invalid"), None);
    }

    #[test]
    fn test_supported_providers() {
        let embedding_providers = EmbeddingProviderKind::supported_providers();
        assert!(embedding_providers.contains(&"openai"));
        assert!(embedding_providers.contains(&"fastembed"));

        let vector_store_providers = VectorStoreProviderKind::supported_providers();
        assert!(vector_store_providers.contains(&"filesystem"));
        assert!(vector_store_providers.contains(&"in-memory"));
    }
}
