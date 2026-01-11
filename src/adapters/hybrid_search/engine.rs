//! Hybrid search engine combining BM25 and semantic search
//!
//! This module provides the core engine that combines BM25 text-based ranking
//! with semantic similarity scores for improved search relevance.

use crate::domain::error::Result;
use crate::domain::types::{CodeChunk, SearchResult};
use std::collections::HashMap;
use validator::Validate;

use super::bm25::{BM25Params, BM25Scorer};

/// Hybrid search result combining BM25 and semantic scores
#[derive(Debug, Clone, Validate)]
pub struct HybridSearchResult {
    /// The original search result
    pub result: SearchResult,
    /// BM25 score (lexical relevance)
    pub bm25_score: f32,
    /// Semantic similarity score (0-1)
    #[validate(range(min = 0.0, max = 1.0))]
    pub semantic_score: f32,
    /// Combined hybrid score
    pub hybrid_score: f32,
}

/// Hybrid search engine combining BM25 and semantic search
#[derive(Debug, Validate)]
pub struct HybridSearchEngine {
    /// BM25 scorer
    #[validate(nested)]
    pub bm25_scorer: Option<BM25Scorer>,
    /// Collection of indexed documents for BM25 scoring
    pub documents: Vec<CodeChunk>,
    /// Weight for BM25 score in hybrid combination (0-1)
    #[validate(range(min = 0.0, max = 1.0))]
    pub bm25_weight: f32,
    /// Weight for semantic score in hybrid combination (0-1)
    #[validate(range(min = 0.0, max = 1.0))]
    pub semantic_weight: f32,
}

impl HybridSearchEngine {
    /// Create a new hybrid search engine
    pub fn new(bm25_weight: f32, semantic_weight: f32) -> Self {
        Self {
            bm25_scorer: None,
            documents: Vec::new(),
            bm25_weight,
            semantic_weight,
        }
    }

    /// Index documents for BM25 scoring
    pub fn index_documents(&mut self, documents: Vec<CodeChunk>) {
        self.documents = documents;
        self.bm25_scorer = Some(BM25Scorer::new(&self.documents, BM25Params::default()));
    }

    /// Perform hybrid search combining BM25 and semantic similarity
    pub fn hybrid_search(
        &self,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<HybridSearchResult>> {
        if self.bm25_scorer.is_none() {
            // Fallback to semantic-only search if BM25 is not indexed
            return Ok(semantic_results
                .into_iter()
                .take(limit)
                .map(|result| HybridSearchResult {
                    bm25_score: 0.0,
                    semantic_score: result.score,
                    hybrid_score: result.score,
                    result,
                })
                .collect());
        }

        let bm25_scorer = self
            .bm25_scorer
            .as_ref()
            .ok_or_else(|| crate::domain::error::Error::internal("BM25 scorer not initialized"))?;

        // Create a mapping from file_path + line_number to document for BM25 scoring
        let mut doc_map = HashMap::new();
        for doc in &self.documents {
            let key = format!("{}:{}", doc.file_path, doc.start_line);
            doc_map.insert(key, doc.clone());
        }

        // Calculate hybrid scores for semantic results
        let mut hybrid_results: Vec<HybridSearchResult> = semantic_results
            .into_iter()
            .map(|semantic_result| {
                let doc_key = format!(
                    "{}:{}",
                    semantic_result.file_path, semantic_result.line_number
                );
                let semantic_score = semantic_result.score;

                if let Some(document) = doc_map.get(&doc_key) {
                    let bm25_score = bm25_scorer.score(document, query);

                    // Normalize BM25 score to 0-1 range (simple min-max normalization)
                    let normalized_bm25 = if bm25_score > 0.0 {
                        1.0 / (1.0 + (-bm25_score).exp()) // Sigmoid normalization
                    } else {
                        0.0
                    };

                    // Combine scores using weighted average
                    let hybrid_score =
                        self.bm25_weight * normalized_bm25 + self.semantic_weight * semantic_score;

                    HybridSearchResult {
                        result: semantic_result,
                        bm25_score,
                        semantic_score,
                        hybrid_score,
                    }
                } else {
                    // If document not found for BM25, use semantic score only
                    let hybrid_score = self.semantic_weight * semantic_score;
                    HybridSearchResult {
                        result: semantic_result,
                        bm25_score: 0.0,
                        semantic_score,
                        hybrid_score,
                    }
                }
            })
            .collect();

        // Sort by hybrid score (descending)
        hybrid_results.sort_by(|a, b| {
            b.hybrid_score
                .partial_cmp(&a.hybrid_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Return top results
        Ok(hybrid_results.into_iter().take(limit).collect())
    }

    /// Check if BM25 index is available
    pub fn has_bm25_index(&self) -> bool {
        self.bm25_scorer.is_some()
    }

    /// Get BM25 statistics
    pub fn get_bm25_stats(&self) -> Option<HashMap<String, serde_json::Value>> {
        self.bm25_scorer.as_ref().map(|scorer| {
            let mut stats = HashMap::new();
            stats.insert(
                "total_documents".to_string(),
                serde_json::json!(scorer.total_docs),
            );
            stats.insert(
                "unique_terms".to_string(),
                serde_json::json!(scorer.document_freq.len()),
            );
            stats.insert(
                "average_doc_length".to_string(),
                serde_json::json!(scorer.avg_doc_len),
            );
            stats.insert("bm25_k1".to_string(), serde_json::json!(scorer.params.k1));
            stats.insert("bm25_b".to_string(), serde_json::json!(scorer.params.b));
            stats
        })
    }
}

impl Default for HybridSearchEngine {
    fn default() -> Self {
        Self::new(0.4, 0.6) // 40% BM25, 60% semantic by default
    }
}
