//! Hybrid search engine combining BM25 and semantic search
//!
//! This module provides the core engine that combines BM25 text-based ranking
//! with semantic similarity scores for improved search relevance.
//!
//! # Architecture
//!
//! ```text
//! Query Input
//!     |
//!     v
//! Parallel Processing:
//!     +-> BM25 Scorer (keyword matching)
//!     |   +-> Term frequency score (0-1)
//!     |
//!     +-> Semantic (from vector store)
//!         +-> Vector similarity score (0-1)
//!
//! Score Fusion:
//!     Hybrid Score = bm25_weight * BM25 + semantic_weight * Semantic
//!
//! Rank Results (highest score first)
//!     |
//!     v
//! Return Top-K Results
//! ```

use async_trait::async_trait;
use mcb_domain::ports::providers::HybridSearchProvider;
use mcb_domain::{entities::CodeChunk, error::Result, value_objects::SearchResult};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::bm25::{BM25Params, BM25Scorer};
use crate::constants::{HYBRID_SEARCH_BM25_WEIGHT, HYBRID_SEARCH_SEMANTIC_WEIGHT};

/// Hybrid search engine combining BM25 and semantic search
///
/// This engine maintains separate BM25 indexes for each collection and combines
/// BM25 scores with semantic similarity scores provided by a vector store.
pub struct HybridSearchEngine {
    /// Weight for BM25 score in hybrid combination (0.0-1.0)
    bm25_weight: f32,
    /// Weight for semantic score in hybrid combination (0.0-1.0)
    semantic_weight: f32,
    /// Collection indexes: collection_name -> (documents, scorer, document_index)
    collections: RwLock<HashMap<String, CollectionIndex>>,
}

/// Index for a single collection
struct CollectionIndex {
    /// Indexed documents
    documents: Vec<CodeChunk>,
    /// BM25 scorer for this collection
    scorer: BM25Scorer,
    /// Document index mapping (file_path:start_line -> document index)
    document_index: HashMap<String, usize>,
}

impl HybridSearchEngine {
    /// Create a new hybrid search engine with default weights
    pub fn new() -> Self {
        Self::with_weights(HYBRID_SEARCH_BM25_WEIGHT, HYBRID_SEARCH_SEMANTIC_WEIGHT)
    }

    /// Create a new hybrid search engine with custom weights
    ///
    /// # Arguments
    ///
    /// * `bm25_weight` - Weight for BM25 score (0.0-1.0)
    /// * `semantic_weight` - Weight for semantic score (0.0-1.0)
    ///
    /// # Note
    ///
    /// Weights do not need to sum to 1.0, but the resulting scores will be
    /// more interpretable if they do.
    pub fn with_weights(bm25_weight: f32, semantic_weight: f32) -> Self {
        Self {
            bm25_weight,
            semantic_weight,
            collections: RwLock::new(HashMap::new()),
        }
    }

    /// Get BM25 weight
    pub fn bm25_weight(&self) -> f32 {
        self.bm25_weight
    }

    /// Get semantic weight
    pub fn semantic_weight(&self) -> f32 {
        self.semantic_weight
    }

    /// Normalize BM25 score to 0-1 range using sigmoid
    fn normalize_bm25_score(score: f32) -> f32 {
        if score > 0.0 {
            1.0 / (1.0 + (-score).exp())
        } else {
            0.0
        }
    }
}

impl Default for HybridSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HybridSearchProvider for HybridSearchEngine {
    /// Index code chunks for hybrid search
    ///
    /// Builds a BM25 index for the specified collection. If the collection
    /// already exists, the new chunks are added to the existing index.
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        let mut collections = self.collections.write().await;

        // Build document index and deduplicate
        let mut documents = Vec::new();
        let mut document_index = HashMap::new();

        // If collection exists, start with existing documents
        if let Some(existing) = collections.get(collection) {
            documents = existing.documents.clone();
            document_index = existing.document_index.clone();
        }

        // Add new documents, deduplicating by key
        for chunk in chunks {
            let key = format!("{}:{}", chunk.file_path, chunk.start_line);
            if let std::collections::hash_map::Entry::Vacant(e) = document_index.entry(key) {
                let idx = documents.len();
                e.insert(idx);
                documents.push(chunk.clone());
            }
        }

        // Build BM25 scorer for all documents
        let scorer = BM25Scorer::new(&documents, BM25Params::default());

        collections.insert(
            collection.to_string(),
            CollectionIndex {
                documents,
                scorer,
                document_index,
            },
        );

        Ok(())
    }

    /// Perform hybrid search combining BM25 and semantic scores
    ///
    /// Takes semantic search results (from a vector store) and re-ranks them
    /// using a combination of BM25 and semantic similarity scores.
    async fn search(
        &self,
        collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let collections = self.collections.read().await;

        // If collection doesn't exist, return semantic results as-is
        let Some(index) = collections.get(collection) else {
            return Ok(semantic_results.into_iter().take(limit).collect());
        };

        // Pre-tokenize query once for all BM25 scoring
        let query_terms = BM25Scorer::tokenize(query);

        // Calculate hybrid scores for semantic results
        let mut scored_results: Vec<(SearchResult, f32)> = semantic_results
            .into_iter()
            .map(|result| {
                let doc_key = format!("{}:{}", result.file_path, result.start_line);
                let semantic_score = result.score as f32;

                // Look up document in index for BM25 scoring
                let hybrid_score = if let Some(&doc_idx) = index.document_index.get(&doc_key) {
                    let document = &index.documents[doc_idx];
                    let bm25_score = index.scorer.score_with_tokens(document, &query_terms);
                    let normalized_bm25 = Self::normalize_bm25_score(bm25_score);

                    // Combine scores using weighted average
                    self.bm25_weight * normalized_bm25 + self.semantic_weight * semantic_score
                } else {
                    // Document not found in BM25 index, use semantic score only
                    self.semantic_weight * semantic_score
                };

                (result, hybrid_score)
            })
            .collect();

        // Sort by hybrid score (descending)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Update scores in results and return top limit
        Ok(scored_results
            .into_iter()
            .take(limit)
            .map(|(mut result, hybrid_score)| {
                result.score = hybrid_score as f64;
                result
            })
            .collect())
    }

    /// Clear indexed data for a collection
    async fn clear_collection(&self, collection: &str) -> Result<()> {
        let mut collections = self.collections.write().await;
        collections.remove(collection);
        Ok(())
    }

    /// Get hybrid search statistics
    async fn get_stats(&self) -> HashMap<String, Value> {
        let collections = self.collections.read().await;

        let mut stats = HashMap::new();

        // Global stats
        stats.insert(
            "bm25_weight".to_string(),
            serde_json::json!(self.bm25_weight),
        );
        stats.insert(
            "semantic_weight".to_string(),
            serde_json::json!(self.semantic_weight),
        );
        stats.insert(
            "collection_count".to_string(),
            serde_json::json!(collections.len()),
        );

        // Per-collection stats
        let mut collection_stats = HashMap::new();
        for (name, index) in collections.iter() {
            collection_stats.insert(
                name.clone(),
                serde_json::json!({
                    "total_documents": index.scorer.total_docs(),
                    "unique_terms": index.scorer.unique_terms(),
                    "average_doc_length": index.scorer.avg_doc_len(),
                    "bm25_k1": index.scorer.params().k1,
                    "bm25_b": index.scorer.params().b,
                }),
            );
        }
        stats.insert(
            "collections".to_string(),
            serde_json::json!(collection_stats),
        );

        stats
    }
}
