//! Tests for FastEmbed local embedding provider
//!
//! Tests for the FastEmbed provider that uses ONNX models for inference.

use mcp_context_browser::adapters::providers::embedding::fastembed::FastEmbedProvider;
use mcp_context_browser::domain::ports::EmbeddingProvider;

#[tokio::test]
async fn test_fastembed_provider_creation() {
    let provider = FastEmbedProvider::new();
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_fastembed_provider_dimensions() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let provider = FastEmbedProvider::new()?;
    assert_eq!(provider.dimensions(), 384);
    assert_eq!(provider.provider_name(), "fastembed");
    Ok(())
}

#[tokio::test]
async fn test_fastembed_provider_embed() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let provider = FastEmbedProvider::new()?;
    let embedding = provider.embed("test text").await?;
    assert_eq!(embedding.dimensions, 384);
    assert_eq!(embedding.model, "AllMiniLML6V2");
    assert_eq!(embedding.vector.len(), 384);
    Ok(())
}

#[tokio::test]
async fn test_fastembed_provider_embed_batch() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let provider = FastEmbedProvider::new()?;
    let texts = vec!["test text 1".to_string(), "test text 2".to_string()];
    let embeddings = provider.embed_batch(&texts).await?;
    assert_eq!(embeddings.len(), 2);

    for embedding in embeddings {
        assert_eq!(embedding.dimensions, 384);
        assert_eq!(embedding.model, "AllMiniLML6V2");
        assert_eq!(embedding.vector.len(), 384);
    }
    Ok(())
}

#[tokio::test]
async fn test_fastembed_provider_health_check(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let provider = FastEmbedProvider::new()?;
    provider.health_check().await?;
    Ok(())
}
