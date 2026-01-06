//! Embedding provider implementations

use crate::core::{error::Result, types::Embedding};
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;

/// Mock embedding provider for MVP/testing
pub struct MockEmbeddingProvider;

impl MockEmbeddingProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Embedding> {
        Ok(Embedding {
            vector: vec![0.1; 128],
            model: "mock".to_string(),
            dimensions: 128,
        })
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; 128],
                model: "mock".to_string(),
                dimensions: 128,
            })
            .collect();
        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        128
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}
