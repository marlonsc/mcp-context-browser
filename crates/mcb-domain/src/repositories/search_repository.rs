//! Search Repository Interface
//!
//! Interface for search operations that combine semantic vector search
//! with keyword-based search capabilities.

use crate::entities::CodeChunk;
use crate::error::Result;
use crate::value_objects::search::SearchResult;
use async_trait::async_trait;
use shaku::Interface;

/// Repository: Semantic and Hybrid Search Operations
///
/// Provides interfaces for semantic vector search and hybrid search
/// that combines semantic similarity with keyword relevance.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::repositories::SearchRepository;
///
/// // Semantic search using embedding vector
/// let results = repo.semantic_search("my-project", &query_vec, 10, None).await?;
///
/// // Hybrid search (semantic + BM25 keyword matching)
/// let results = repo.hybrid_search("my-project", "async error handling", &query_vec, 10).await?;
/// for result in results {
///     println!("{}: score={:.3}", result.file_path, result.score);
/// }
///
/// // Get search statistics
/// let stats = repo.stats().await?;
/// println!("Queries: {}, Avg time: {:.1}ms", stats.total_queries, stats.avg_response_time_ms);
/// ```
#[async_trait]
pub trait SearchRepository: Interface + Send + Sync {
    /// Semantic search using vector similarity
    ///
    /// # Arguments
    /// - `collection`: Collection to search
    /// - `query_vector`: Embedding vector from the query
    /// - `limit`: Maximum results to return
    /// - `filter`: Optional metadata filter predicate
    ///
    /// # Returns
    /// Results ranked by similarity score (highest first)
    async fn semantic_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Index chunks for keyword/BM25 search
    ///
    /// Called during indexing to make chunks searchable via keyword matching.
    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()>;

    /// Hybrid search combining semantic and keyword relevance
    ///
    /// # Arguments
    /// - `collection`: Collection to search
    /// - `query`: Natural language query text
    /// - `query_vector`: Embedding of the query
    /// - `limit`: Maximum results
    ///
    /// # Returns
    /// Results ranked by combined semantic + BM25 score
    async fn hybrid_search(
        &self,
        collection: &str,
        query: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear keyword search index for a collection
    ///
    /// # Arguments
    /// - `collection`: Collection/namespace identifier to clear the search index for
    ///
    /// # Note
    /// This only affects the keyword/BM25 search index, not the semantic vector storage
    async fn clear_index(&self, collection: &str) -> Result<()>;

    /// Get search operation statistics
    ///
    /// # Returns
    /// Stats including query counts, response times, and cache hit rates
    async fn stats(&self) -> Result<SearchStats>;
}

/// Value Object: Search Operation Statistics
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total queries executed
    pub total_queries: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Number of indexed documents
    pub indexed_documents: u64,
}
