//! Hybrid Search Port
//!
//! Defines the interface for hybrid search capabilities that combine
//! lexical (BM25) and semantic (vector) search.

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, SearchResult};
use async_trait::async_trait;
use shaku::Interface;
use std::collections::HashMap;

/// Result of a hybrid search operation
#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub result: SearchResult,
    pub bm25_score: f32,
    pub semantic_score: f32,
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
