//! Null embedding provider for testing and development

use crate::domain::error::Result;
use crate::domain::ports::EmbeddingProvider;
use crate::domain::types::Embedding;
use crate::infrastructure::constants::EMBEDDING_DIMENSION_NULL;
use async_trait::async_trait;

/// Null embedding provider for testing
/// Returns fixed-size vectors filled with test values
#[derive(shaku::Component)]
#[shaku(interface = EmbeddingProvider)]
pub struct NullEmbeddingProvider;

impl NullEmbeddingProvider {
    /// Create a new null embedding provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for NullEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let embeddings = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                // Create more realistic test embeddings with varied values
                // Use text hash to create deterministic but varied vectors
                let hash = text.chars().map(|c| c as u32).sum::<u32>();
                let base_value = (hash % 1000) as f32 / 1000.0; // 0.0 to 1.0

                // Create a 384-dimensional vector (similar to many real embedding models)
                let vector = (0..384)
                    .map(|j| {
                        // Create varied values based on text hash and position
                        let variation = ((i as f32 + j as f32) * 0.01).sin();
                        (base_value + variation * 0.1).clamp(0.0, 1.0)
                    })
                    .collect();

                Embedding {
                    vector,
                    model: "null-test".to_string(),
                    dimensions: 384,
                }
            })
            .collect();

        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        EMBEDDING_DIMENSION_NULL
    }

    fn provider_name(&self) -> &str {
        "null"
    }
}

impl NullEmbeddingProvider {
    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        "null"
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        512
    }
}
