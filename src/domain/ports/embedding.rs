use crate::domain::error::Result;
use crate::domain::types::Embedding;
use async_trait::async_trait;
use shaku::Interface;

/// AI Semantic Understanding Interface
///
/// Defines the business contract for AI providers that transform text into
/// semantic embeddings. This abstraction enables the platform to work with
/// any AI service that can understand code semantics, from enterprise OpenAI
/// deployments to self-hosted Ollama instances.
///
/// # Default Implementations
///
/// The `embed()` method has a default implementation that delegates to
/// `embed_batch()` with a single item. Providers only need to implement
/// `embed_batch()` unless custom single-item optimization is needed.
#[async_trait]
pub trait EmbeddingProvider: Interface + Send + Sync {
    /// Get embedding for a single text (default implementation provided)
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Default: delegate to embed_batch
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| crate::domain::error::Error::embedding("No embedding returned"))
    }

    /// Get embeddings for multiple texts (must be implemented by provider)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation provided)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple embed operation
        self.embed("health check").await?;
        Ok(())
    }
}
