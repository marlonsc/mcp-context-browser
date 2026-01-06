//! Context service for managing embeddings and vector storage

use crate::error::{Error, Result};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use crate::types::{CodeChunk, Embedding, SearchResult};
use std::collections::HashMap;
use std::sync::Arc;

/// Context service that orchestrates embedding and vector storage operations
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    /// Create a new context service with specified providers
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            embedding_provider,
            vector_store_provider,
        }
    }

    /// Create a new context service with default providers (for backward compatibility)
    pub fn default() -> Self {
        use crate::providers::{MockEmbeddingProvider, InMemoryVectorStoreProvider};

        Self {
            embedding_provider: Arc::new(MockEmbeddingProvider::new()),
            vector_store_provider: Arc::new(InMemoryVectorStoreProvider::new()),
        }
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Store code chunks in vector database
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embed_texts(&texts).await?;

        // Prepare metadata for each chunk
        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks.iter().map(|chunk| {
            let mut meta = HashMap::new();
            meta.insert("content".to_string(), serde_json::json!(chunk.content));
            meta.insert("file_path".to_string(), serde_json::json!(chunk.file_path));
            meta.insert("start_line".to_string(), serde_json::json!(chunk.start_line));
            meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
            meta.insert("language".to_string(), serde_json::json!(format!("{:?}", chunk.language)));
            meta
        }).collect();

        // Ensure collection exists
        if !self.vector_store_provider.collection_exists(collection).await? {
            self.vector_store_provider.create_collection(collection, self.embedding_dimensions()).await?;
        }

        self.vector_store_provider.insert_vectors(collection, &embeddings, metadata).await?;
        Ok(())
    }

    /// Search for similar code chunks
    pub async fn search_similar(&self, collection: &str, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;
        let results = self.vector_store_provider.search_similar(collection, &query_embedding.vector, limit, None).await?;

        let search_results = results.into_iter().map(|result| {
            SearchResult {
                file_path: result.metadata.get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: result.metadata.get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result.metadata.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            }
        }).collect();

        Ok(search_results)
    }

    /// Clear a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.vector_store_provider.delete_collection(collection).await?;
        Ok(())
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}

impl Default for ContextService {
    fn default() -> Self {
        Self::default()
    }
}