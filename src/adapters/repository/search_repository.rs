//! Repository for managing search operations
//!
//! This module provides search functionality over indexed code chunks.

use crate::adapters::hybrid_search::HybridSearchEngine;
use crate::adapters::repository::{SearchRepository, SearchStats};
use crate::domain::error::Result;
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{CodeChunk, SearchResult};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Vector store backed search repository with hybrid search support
pub struct VectorStoreSearchRepository<V> {
    vector_store_provider: Arc<V>,
    hybrid_engine: Arc<RwLock<HybridSearchEngine>>,
    stats: SearchStatsTracker,
}

/// Tracks search statistics using atomic counters
struct SearchStatsTracker {
    total_queries: AtomicU64,
    total_response_time_ms: AtomicU64,
    cache_hits: AtomicU64,
    indexed_documents: AtomicU64,
}

impl Default for SearchStatsTracker {
    fn default() -> Self {
        Self {
            total_queries: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            indexed_documents: AtomicU64::new(0),
        }
    }
}

impl<V> VectorStoreSearchRepository<V> {
    pub fn new(vector_store_provider: Arc<V>) -> Self {
        Self {
            vector_store_provider,
            hybrid_engine: Arc::new(RwLock::new(HybridSearchEngine::new(0.3, 0.7))),
            stats: SearchStatsTracker::default(),
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
                id: result.id.clone(),
                file_path: result
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                line_number: result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            })
            .collect();

        Ok(search_results)
    }

    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()> {
        // Index documents for BM25 scoring in the hybrid engine
        let mut engine = self.hybrid_engine.write().await;
        engine.index_documents(chunks.to_vec());
        self.stats
            .indexed_documents
            .store(chunks.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    async fn hybrid_search(
        &self,
        collection: &str,
        query: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let start = std::time::Instant::now();
        self.stats.total_queries.fetch_add(1, Ordering::Relaxed);

        // Get semantic search results from vector store
        let semantic_results = self
            .semantic_search(collection, query_vector, limit * 2, None)
            .await?;

        // Apply hybrid search (BM25 + semantic) using the engine
        let engine = self.hybrid_engine.read().await;
        let hybrid_results = engine.hybrid_search(query, semantic_results, limit)?;

        // Track response time
        let elapsed = start.elapsed().as_millis() as u64;
        self.stats
            .total_response_time_ms
            .fetch_add(elapsed, Ordering::Relaxed);

        // Convert HybridSearchResult to SearchResult
        Ok(hybrid_results.into_iter().map(|hr| hr.result).collect())
    }

    async fn clear_index(&self, collection: &str) -> Result<()> {
        // Clear the collection from vector store
        let collection_name = self.collection_name(collection);
        if self
            .vector_store_provider
            .collection_exists(&collection_name)
            .await?
        {
            self.vector_store_provider
                .delete_collection(&collection_name)
                .await?;
        }

        // Reset hybrid engine
        let mut engine = self.hybrid_engine.write().await;
        *engine = HybridSearchEngine::new(0.3, 0.7);
        self.stats.indexed_documents.store(0, Ordering::Relaxed);
        Ok(())
    }

    async fn search_stats(&self) -> Result<SearchStats> {
        let total_queries = self.stats.total_queries.load(Ordering::Relaxed);
        let total_time = self.stats.total_response_time_ms.load(Ordering::Relaxed);
        let cache_hits = self.stats.cache_hits.load(Ordering::Relaxed);
        let indexed_docs = self.stats.indexed_documents.load(Ordering::Relaxed);

        let avg_response_time = if total_queries > 0 {
            total_time as f64 / total_queries as f64
        } else {
            0.0
        };

        let cache_hit_rate = if total_queries > 0 {
            cache_hits as f64 / total_queries as f64
        } else {
            0.0
        };

        Ok(SearchStats {
            total_queries,
            avg_response_time_ms: avg_response_time,
            cache_hit_rate,
            indexed_documents: indexed_docs,
        })
    }
}
