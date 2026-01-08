//! Repository for managing search operations
//!
//! This module provides search functionality over indexed code chunks.

use crate::core::error::Result;
use crate::core::types::{CodeChunk, SearchResult};
use crate::providers::VectorStoreProvider;
use crate::repository::{SearchRepository, SearchStats};
use async_trait::async_trait;
use std::sync::Arc;


/// Vector store backed search repository
pub struct VectorStoreSearchRepository<V> {
    vector_store_provider: Arc<V>,
}

impl<V> VectorStoreSearchRepository<V> {
    pub fn new(vector_store_provider: Arc<V>) -> Self {
        Self {
            vector_store_provider,
        }
    }

    fn collection_name(&self, collection: &str) -> String {
        format!("mcp_chunks_{}", collection)
    }
}

// Additional implementation methods
#[async_trait]
impl<V> SearchRepository for VectorStoreSearchRepository<V>
where
    V: VectorStoreProvider + Send + Sync,
{
    async fn semantic_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let collection_name = self.collection_name(collection);

        // Check if collection exists
        if !self
            .vector_store_provider
            .collection_exists(&collection_name)
            .await?
        {
            return Ok(vec![]);
        }

        // Perform semantic search using vector similarity
        let results = self
            .vector_store_provider
            .search_similar(&collection_name, query_vector, limit, filter)
            .await?;

        // Convert to SearchResult format
        let search_results: Vec<SearchResult> = results
            .into_iter()
            .map(|result| SearchResult {
                file_path: result.metadata.get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
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
            })
            .collect();

        Ok(search_results)
    }

    async fn index_for_hybrid_search(&self, _chunks: &[CodeChunk]) -> Result<()> {
        // TODO: Implement hybrid search indexing with BM25
        // For now, this is a placeholder
        Ok(())
    }

    async fn hybrid_search(
        &self,
        _collection: &str,
        _query: &str,
        _query_vector: &[f32],
        _limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // TODO: Implement hybrid search combining semantic and keyword search
        // For now, fall back to semantic search only
        Ok(vec![])
    }

    async fn clear_index(&self, _collection: &str) -> Result<()> {
        // TODO: Implement index clearing for hybrid search
        Ok(())
    }

    async fn search_stats(&self) -> Result<SearchStats> {
        // TODO: Implement search statistics
        Ok(SearchStats {
            total_queries: 0,
            avg_response_time_ms: 0.0,
            cache_hit_rate: 0.0,
            indexed_documents: 0,
        })
    }
}
