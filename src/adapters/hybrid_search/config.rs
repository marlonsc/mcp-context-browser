//! Hybrid search configuration
//!
//! This module provides configuration options for the hybrid search system,
//! including weights for BM25 and semantic scores.

use crate::infrastructure::constants::{
    HYBRID_SEARCH_BM25_B, HYBRID_SEARCH_BM25_K1, HYBRID_SEARCH_BM25_WEIGHT,
    HYBRID_SEARCH_SEMANTIC_WEIGHT,
};
use validator::Validate;

/// Hybrid search configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct HybridSearchConfig {
    /// Enable hybrid search
    pub enabled: bool,
    /// Weight for BM25 score (0-1)
    #[validate(range(min = 0.0, max = 1.0))]
    pub bm25_weight: f32,
    /// Weight for semantic score (0-1)
    #[validate(range(min = 0.0, max = 1.0))]
    pub semantic_weight: f32,
    /// BM25 k1 parameter
    #[validate(range(min = 0.0))]
    pub bm25_k1: f32,
    /// BM25 b parameter
    #[validate(range(min = 0.0, max = 1.0))]
    pub bm25_b: f32,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bm25_weight: HYBRID_SEARCH_BM25_WEIGHT as f32,
            semantic_weight: HYBRID_SEARCH_SEMANTIC_WEIGHT as f32,
            bm25_k1: HYBRID_SEARCH_BM25_K1 as f32,
            bm25_b: HYBRID_SEARCH_BM25_B as f32,
        }
    }
}

impl HybridSearchConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: std::env::var("HYBRID_SEARCH_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            bm25_weight: std::env::var("HYBRID_SEARCH_BM25_WEIGHT")
                .unwrap_or_else(|_| HYBRID_SEARCH_BM25_WEIGHT.to_string())
                .parse()
                .unwrap_or(HYBRID_SEARCH_BM25_WEIGHT as f32),
            semantic_weight: std::env::var("HYBRID_SEARCH_SEMANTIC_WEIGHT")
                .unwrap_or_else(|_| HYBRID_SEARCH_SEMANTIC_WEIGHT.to_string())
                .parse()
                .unwrap_or(HYBRID_SEARCH_SEMANTIC_WEIGHT as f32),
            bm25_k1: std::env::var("HYBRID_SEARCH_BM25_K1")
                .unwrap_or_else(|_| HYBRID_SEARCH_BM25_K1.to_string())
                .parse()
                .unwrap_or(HYBRID_SEARCH_BM25_K1 as f32),
            bm25_b: std::env::var("HYBRID_SEARCH_BM25_B")
                .unwrap_or_else(|_| HYBRID_SEARCH_BM25_B.to_string())
                .parse()
                .unwrap_or(HYBRID_SEARCH_BM25_B as f32),
        }
    }
}
