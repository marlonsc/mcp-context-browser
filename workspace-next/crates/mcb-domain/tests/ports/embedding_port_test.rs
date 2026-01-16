//! Tests for EmbeddingProvider port trait
//!
//! Validates the contract and default implementations of the embedding provider port.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;
use std::sync::Arc;

/// Mock implementation of EmbeddingProvider for testing
struct MockEmbeddingProvider {
    dimensions: usize,
    provider_name: String,
    should_fail: bool,
}

impl MockEmbeddingProvider {
    fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            provider_name: "mock".to_string(),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            dimensions: 384,
            provider_name: "failing-mock".to_string(),
            should_fail: true,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::embedding("Simulated failure"));
        }

        Ok(texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; self.dimensions],
                model: self.provider_name.clone(),
                dimensions: self.dimensions,
            })
            .collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }
}

#[test]
fn test_embedding_provider_trait_object() {
    // Verify that EmbeddingProvider can be used as a trait object
    let provider: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingProvider::new(384));

    assert_eq!(provider.dimensions(), 384);
    assert_eq!(provider.provider_name(), "mock");
}

#[tokio::test]
async fn test_embedding_provider_default_embed() {
    // Test the default embed() implementation that delegates to embed_batch()
    let provider = MockEmbeddingProvider::new(384);

    let result = provider.embed("test text").await;

    assert!(result.is_ok());
    let embedding = result.expect("Expected embedding");
    assert_eq!(embedding.dimensions, 384);
    assert_eq!(embedding.model, "mock");
    assert_eq!(embedding.vector.len(), 384);
}

#[tokio::test]
async fn test_embedding_provider_batch() {
    let provider = MockEmbeddingProvider::new(512);

    let texts = vec![
        "first text".to_string(),
        "second text".to_string(),
        "third text".to_string(),
    ];

    let result = provider.embed_batch(&texts).await;

    assert!(result.is_ok());
    let embeddings = result.expect("Expected embeddings");
    assert_eq!(embeddings.len(), 3);
    for embedding in embeddings {
        assert_eq!(embedding.dimensions, 512);
    }
}

#[tokio::test]
async fn test_embedding_provider_health_check_default() {
    // Test the default health_check() implementation
    let provider = MockEmbeddingProvider::new(384);

    let result = provider.health_check().await;

    // Default health check calls embed("health check")
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_embedding_provider_health_check_failure() {
    let provider = MockEmbeddingProvider::failing();

    let result = provider.health_check().await;

    assert!(result.is_err());
}

#[test]
fn test_embedding_provider_dimensions_variety() {
    // Test various common embedding dimensions
    let dimensions = vec![384, 512, 768, 1024, 1536, 3072];

    for dim in dimensions {
        let provider = MockEmbeddingProvider::new(dim);
        assert_eq!(provider.dimensions(), dim);
    }
}
