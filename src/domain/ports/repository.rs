use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, RepositoryStats, SearchResult, SearchStats};
use async_trait::async_trait;
use shaku::Interface;

/// Repository for managing code chunks
#[async_trait]
pub trait ChunkRepository: Interface + Send + Sync {
    /// Save a code chunk to the repository
    async fn save(&self, collection: &str, chunk: &CodeChunk) -> Result<String>;

    /// Save multiple code chunks to the repository
    async fn save_batch(&self, collection: &str, chunks: &[CodeChunk]) -> Result<Vec<String>>;

    /// Find a code chunk by ID in a collection
    async fn find_by_id(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>>;

    /// Find code chunks by collection
    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>>;

    /// Delete a code chunk by ID in a collection
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;

    /// Delete all chunks in a collection
    async fn delete_collection(&self, collection: &str) -> Result<()>;

    /// Get repository statistics
    async fn stats(&self) -> Result<RepositoryStats>;
}

/// Repository for managing search operations and results
#[async_trait]
pub trait SearchRepository: Interface + Send + Sync {
    /// Perform semantic search and return results
    async fn semantic_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Index code chunks for hybrid search (BM25)
    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()>;

    /// Perform hybrid search (semantic + keyword)
    async fn hybrid_search(
        &self,
        collection: &str,
        query: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear search index for a collection
    async fn clear_index(&self, collection: &str) -> Result<()>;

    /// Get search repository statistics
    async fn search_stats(&self) -> Result<SearchStats>;
}
