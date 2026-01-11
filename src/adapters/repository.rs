//! Repository pattern implementation for data access abstraction
//!
//! This module provides repository interfaces and implementations following
//! the Repository pattern to separate data access logic from business logic.

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, SearchResult};
use async_trait::async_trait;

/// Repository for managing code chunks
#[async_trait]
pub trait ChunkRepository: Send + Sync {
    /// Save a code chunk to the repository
    async fn save(&self, chunk: &CodeChunk) -> Result<String>;

    /// Save multiple code chunks to the repository
    async fn save_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<String>>;

    /// Find a code chunk by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<CodeChunk>>;

    /// Find code chunks by collection
    async fn find_by_collection(&self, collection: &str, limit: usize) -> Result<Vec<CodeChunk>>;

    /// Delete a code chunk by ID
    async fn delete(&self, id: &str) -> Result<()>;

    /// Delete all chunks in a collection
    async fn delete_collection(&self, collection: &str) -> Result<()>;

    /// Get repository statistics
    async fn stats(&self) -> Result<RepositoryStats>;
}

/// Repository for managing search operations and results
#[async_trait]
pub trait SearchRepository: Send + Sync {
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

/// Statistics for repository operations
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    /// Total number of chunks stored
    pub total_chunks: u64,
    /// Number of collections
    pub total_collections: u64,
    /// Total storage size in bytes
    pub storage_size_bytes: u64,
    /// Average chunk size in bytes
    pub avg_chunk_size_bytes: f64,
}

/// Statistics for search operations
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total number of search queries performed
    pub total_queries: u64,
    /// Average query response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Number of indexed documents for hybrid search
    pub indexed_documents: u64,
}

// Submodules
pub mod chunk_repository;
pub mod search_repository;

// Re-export implementations
pub use chunk_repository::VectorStoreChunkRepository;
pub use search_repository::VectorStoreSearchRepository;
