//! FastEmbed local embedding provider implementation
//!
//! This provider uses the fastembed library to generate embeddings locally
//! without requiring external API calls. It uses ONNX models for inference.

use crate::core::error::{Error, Result};
use crate::core::types::Embedding;
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::sync::Arc;
use tokio::sync::Mutex;

/// FastEmbed local embedding provider
pub struct FastEmbedProvider {
    model: Arc<Mutex<TextEmbedding>>,
    model_name: String,
}

impl FastEmbedProvider {
    /// Create a new FastEmbed provider with the default model (AllMiniLML6V2)
    pub fn new() -> Result<Self> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    /// Create a new FastEmbed provider with a specific model
    pub fn with_model(model: EmbeddingModel) -> Result<Self> {
        let init_options = InitOptions::new(model.clone()).with_show_download_progress(true);

        let text_embedding = TextEmbedding::try_new(init_options)
            .map_err(|e| Error::embedding(format!("Failed to initialize FastEmbed model: {}", e)))?;

        let model_name = format!("{:?}", model);

        Ok(Self {
            model: Arc::new(Mutex::new(text_embedding)),
            model_name,
        })
    }

    /// Create a new FastEmbed provider with custom initialization options
    pub fn with_options(init_options: InitOptions) -> Result<Self> {
        let model = init_options.model_name.clone();

        let text_embedding = TextEmbedding::try_new(init_options)
            .map_err(|e| Error::embedding(format!("Failed to initialize FastEmbed model: {}", e)))?;

        let model_name = format!("{:?}", model);

        Ok(Self {
            model: Arc::new(Mutex::new(text_embedding)),
            model_name,
        })
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model_name
    }

    /// Get the maximum tokens supported by this model (approximate)
    pub fn max_tokens(&self) -> usize {
        // Most FastEmbed models support around 512 tokens
        // This is a conservative estimate
        512
    }
}

#[async_trait]
impl EmbeddingProvider for FastEmbedProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings.into_iter().next().ok_or_else(|| {
            Error::embedding("No embedding returned from FastEmbed".to_string())
        })
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Convert texts to Vec<&str> for fastembed
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        // Lock the model for exclusive access
        let model = self.model.lock().await;

        // Generate embeddings
        let embeddings_result = model.embed(text_refs, None)
            .map_err(|e| Error::embedding(format!("FastEmbed embedding failed: {}", e)))?;

        // Convert to our Embedding format
        let embeddings = embeddings_result
            .into_iter()
            .map(|vector| Embedding {
                vector: vector.clone(),
                model: self.model_name.clone(),
                dimensions: vector.len(),
            })
            .collect();

        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        // AllMiniLML6V2 has 384 dimensions
        // We'll query the model for actual dimensions
        // For now, return a reasonable default that matches AllMiniLML6V2
        384
    }

    fn provider_name(&self) -> &str {
        "fastembed"
    }

    async fn health_check(&self) -> Result<()> {
        // Try a simple embedding to verify the model works
        self.embed("health check").await?;
        Ok(())
    }
}

impl Clone for FastEmbedProvider {
    fn clone(&self) -> Self {
        Self {
            model: Arc::clone(&self.model),
            model_name: self.model_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fastembed_provider_creation() {
        let provider = FastEmbedProvider::new();
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_fastembed_provider_dimensions() {
        let provider = FastEmbedProvider::new().unwrap();
        assert_eq!(provider.dimensions(), 384);
        assert_eq!(provider.provider_name(), "fastembed");
    }

    #[tokio::test]
    async fn test_fastembed_provider_embed() {
        let provider = FastEmbedProvider::new().unwrap();
        let result = provider.embed("test text").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.dimensions, 384);
        assert_eq!(embedding.model, "AllMiniLML6V2");
        assert_eq!(embedding.vector.len(), 384);
    }

    #[tokio::test]
    async fn test_fastembed_provider_embed_batch() {
        let provider = FastEmbedProvider::new().unwrap();
        let texts = vec!["test text 1".to_string(), "test text 2".to_string()];
        let result = provider.embed_batch(&texts).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 2);

        for embedding in embeddings {
            assert_eq!(embedding.dimensions, 384);
            assert_eq!(embedding.model, "AllMiniLML6V2");
            assert_eq!(embedding.vector.len(), 384);
        }
    }

    #[tokio::test]
    async fn test_fastembed_provider_health_check() {
        let provider = FastEmbedProvider::new().unwrap();
        let result = provider.health_check().await;
        assert!(result.is_ok());
    }
}