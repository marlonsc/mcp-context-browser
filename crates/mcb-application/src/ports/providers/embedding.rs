use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::value_objects::Embedding;
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
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::EmbeddingProvider;
///
/// // Inject the provider via dependency injection
/// let provider: Arc<dyn EmbeddingProvider> = container.resolve();
///
/// // Generate embedding for code
/// let embedding = provider.embed("fn main() { println!(\"Hello\"); }").await?;
/// println!("Embedding dimensions: {}", provider.dimensions());
///
/// // Batch processing for efficiency
/// let texts = vec!["fn foo() {}".into(), "fn bar() {}".into()];
/// let embeddings = provider.embed_batch(&texts).await?;
/// ```
#[async_trait]
pub trait EmbeddingProvider: Interface + Send + Sync {
    /// Get embedding for a single text (default implementation provided)
    async fn embed(&self, text: &str) -> Result<Embedding> {
        // Default: delegate to embed_batch
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| mcb_domain::error::Error::embedding("No embedding returned"))
    }

    /// Get embeddings for multiple texts (must be implemented by provider)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;

    /// Get the dimensionality of embeddings produced by this provider
    ///
    /// # Returns
    /// The number of dimensions in each embedding vector
    fn dimensions(&self) -> usize;

    /// Get the name/identifier of this provider implementation
    ///
    /// # Returns
    /// A string identifier for the provider (e.g., "openai", "ollama", "anthropic")
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation provided)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple embed operation
        self.embed("health check").await?;
        Ok(())
    }
}
