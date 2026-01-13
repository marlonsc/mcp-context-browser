//! Repository for managing search operations
//!
//! This module provides search functionality over indexed code chunks.

use crate::adapters::hybrid_search::HybridSearchEngine;
use crate::domain::error::Result;
use crate::domain::ports::{SearchRepository, VectorStoreProvider};
use crate::domain::types::{CodeChunk, SearchResult, SearchStats};
use crate::infrastructure::service_helpers::TimedOperation;
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Vector store backed search repository with hybrid search support
#[derive(shaku::Component)]
#[shaku(interface = SearchRepository)]
pub struct VectorStoreSearchRepository {
    #[shaku(inject)]
    vector_store_provider: Arc<dyn VectorStoreProvider>,
    #[shaku(default = Arc::new(RwLock::new(HybridSearchEngine::new(0.3, 0.7))))]
    hybrid_engine: Arc<RwLock<HybridSearchEngine>>,
    #[shaku(default)]
    stats: SearchStatsTracker,
}

/// Tracks search statistics using atomic counters
pub struct SearchStatsTracker {
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

impl VectorStoreSearchRepository {
    pub fn new(vector_store_provider: Arc<dyn VectorStoreProvider>) -> Self {
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

#[async_trait]
impl SearchRepository for VectorStoreSearchRepository {
    async fn semantic_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let collection_name = self.collection_name(collection);

        if !self
            .vector_store_provider
            .collection_exists(&collection_name)
            .await?
        {
            return Ok(vec![]);
        }

        self.vector_store_provider
            .search_similar(&collection_name, query_vector, limit, filter)
            .await
    }

    async fn index_for_hybrid_search(&self, chunks: &[CodeChunk]) -> Result<()> {
        let mut engine = self.hybrid_engine.write().await;
        engine.add_documents(chunks.to_vec());
        self.stats
            .indexed_documents
            .store(engine.documents.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    async fn hybrid_search(
        &self,
        collection: &str,
        query: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let timer = TimedOperation::start();
        self.stats.total_queries.fetch_add(1, Ordering::Relaxed);

        let semantic_results = self
            .semantic_search(collection, query_vector, limit * 2, None)
            .await?;

        let engine = self.hybrid_engine.read().await;
        let hybrid_results = engine.hybrid_search(query, semantic_results, limit)?;

        self.stats
            .total_response_time_ms
            .fetch_add(timer.elapsed_ms(), Ordering::Relaxed);

        Ok(hybrid_results.into_iter().map(|hr| hr.result).collect())
    }

    async fn clear_index(&self, collection: &str) -> Result<()> {
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
