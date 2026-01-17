//! Hybrid Search Port
//!
//! Defines the interface for hybrid search capabilities that combine
//! lexical (BM25) and semantic (vector) search.

use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::value_objects::SearchResult;
use async_trait::async_trait;
use shaku::Interface;
use std::collections::HashMap;

/// Result of a hybrid search operation
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    /// The underlying search result with code chunk and metadata
    pub result: SearchResult,
    /// BM25 lexical matching score (0.0 to 1.0)
    pub bm25_score: f32,
    /// Semantic similarity score from vector search (0.0 to 1.0)
    pub semantic_score: f32,
    /// Combined hybrid score from both BM25 and semantic components
    pub hybrid_score: f32,
}

/// Port for hybrid search operations
///
/// Combines lexical (BM25) and semantic (vector) search for improved relevance.
/// BM25 excels at exact keyword matching while semantic search understands meaning.
///
/// # Example
///
/// ```ignore
/// use mcb_domain::ports::providers::HybridSearchProvider;
///
/// // Index code chunks for hybrid search
/// provider.index_chunks("project", &code_chunks).await?;
///
/// // Perform hybrid search combining BM25 and semantic results
/// let semantic_results = vector_store.search_similar("project", &query_vec, 20, None).await?;
/// let results = provider.search("project", "async fn", semantic_results, 10).await?;
///
/// // Results are ranked by combined BM25 + semantic scores
/// for result in results {
///     println!("{}: {}", result.file_path, result.score);
/// }
/// ```
#[async_trait]
pub trait HybridSearchProvider: Interface + Send + Sync {
    /// Index code chunks for hybrid search
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;

    /// Perform hybrid search
    async fn search(
        &self,
        collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;

    /// Clear indexed data for a collection
    async fn clear_collection(&self, collection: &str) -> Result<()>;

    /// Get hybrid search statistics
    async fn get_stats(&self) -> HashMap<String, serde_json::Value>;
}
