//! Null embedding provider for testing and development
//!
//! Provides deterministic, hash-based embeddings for testing purposes.
//! No external dependencies - always works offline.

use async_trait::async_trait;

use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::constants::EMBEDDING_DIMENSION_NULL;

/// Null embedding provider for testing
///
/// Returns fixed-size vectors filled with deterministic values based on
/// input text hash. Useful for unit tests and development without requiring
/// an actual embedding service.
///
/// # Example
///
/// ```rust
/// use mcb_providers::embedding::NullEmbeddingProvider;
/// use mcb_domain::ports::providers::EmbeddingProvider;
///
/// let provider = NullEmbeddingProvider::new();
/// assert_eq!(provider.dimensions(), 384);
/// assert_eq!(provider.provider_name(), "null");
/// ```
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
                // Create deterministic test embeddings based on text hash
                let hash = text.chars().map(|c| c as u32).sum::<u32>();
                let base_value = (hash % 1000) as f32 / 1000.0; // 0.0 to 1.0

                // Create a 384-dimensional vector (matches common embedding models)
                let vector = (0..EMBEDDING_DIMENSION_NULL)
                    .map(|j| {
                        // Create varied values based on text hash and position
                        let variation = ((i as f32 + j as f32) * 0.01).sin();
                        (base_value + variation * 0.1).clamp(0.0, 1.0)
                    })
                    .collect();

                Embedding {
                    vector,
                    model: "null-test".to_string(),
                    dimensions: EMBEDDING_DIMENSION_NULL,
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

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};
use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;

/// Factory function for creating null embedding provider instances.
fn null_embedding_factory(
    _config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    Ok(Arc::new(NullEmbeddingProvider::new()))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static NULL_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "null",
    description: "Null provider for testing (deterministic hash-based embeddings)",
    factory: null_embedding_factory,
};
