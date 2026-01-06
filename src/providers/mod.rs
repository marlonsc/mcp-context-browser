//! Provider interfaces and implementations

use crate::core::{error::Result, types::Embedding};
use async_trait::async_trait;

/// Embedding provider trait
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Embedding>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
}

/// Vector store provider trait
#[async_trait]
pub trait VectorStoreProvider: Send + Sync {
    async fn store(&self, collection: &str, embeddings: &[Embedding]) -> Result<()>;
    async fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<(f32, Embedding)>>;
    async fn clear(&self, collection: &str) -> Result<()>;
    fn provider_name(&self) -> &str;
}

// Submodules
pub mod embedding;
pub mod vector_store;

// Re-export implementations
pub use embedding::MockEmbeddingProvider;
pub use vector_store::InMemoryVectorStoreProvider;
