//! Hybrid Search Port
//!
//! Defines the interface for hybrid search capabilities that combine
//! lexical (BM25) and semantic (vector) search.

use crate::error::Result;
use crate::entities::CodeChunk;
use crate::value_objects::SearchResult;
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
