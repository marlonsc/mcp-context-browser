//! BM25 text ranking algorithm implementation
//!
//! BM25 (Best Matching 25) is a ranking function used for information retrieval.
//! It ranks documents based on the query terms appearing in each document.

use crate::domain::types::CodeChunk;
use crate::infrastructure::constants::{
    BM25_TOKEN_MIN_LENGTH, HYBRID_SEARCH_BM25_B, HYBRID_SEARCH_BM25_K1,
};
use std::collections::{HashMap, HashSet};
use validator::Validate;

/// BM25 parameters
#[derive(Debug, Clone, Validate)]
pub struct BM25Params {
    /// k1 parameter (term frequency saturation)
    #[validate(range(min = 0.0))]
    pub k1: f32,
    /// b parameter (document length normalization)
    #[validate(range(min = 0.0, max = 1.0))]
    pub b: f32,
}

impl Default for BM25Params {
    fn default() -> Self {
        Self {
            k1: HYBRID_SEARCH_BM25_K1 as f32, // Standard BM25 k1 value
            b: HYBRID_SEARCH_BM25_B as f32,   // Standard BM25 b value
        }
    }
}

/// BM25 scorer for text-based ranking
#[derive(Debug, Validate)]
pub struct BM25Scorer {
    /// Document frequencies for each term
    pub document_freq: HashMap<String, usize>,
    /// Total number of documents
    #[validate(range(min = 0))]
    pub total_docs: usize,
    /// Average document length
    #[validate(range(min = 0.0))]
    pub avg_doc_len: f32,
    /// BM25 parameters
    #[validate(nested)]
    pub params: BM25Params,
}

impl BM25Scorer {
    /// Create a new BM25 scorer from a collection of documents
    pub fn new(documents: &[CodeChunk], params: BM25Params) -> Self {
        let total_docs = documents.len();
        let mut document_freq = HashMap::new();
        let mut total_length = 0.0;

        // Calculate document frequencies and total length
        for doc in documents {
            let tokens = Self::tokenize(&doc.content);
            let doc_length = tokens.len() as f32;
            total_length += doc_length;

            let mut unique_terms = HashSet::new();
            for token in tokens {
                unique_terms.insert(token);
            }

            for term in unique_terms {
                *document_freq.entry(term).or_insert(0) += 1;
            }
        }

        let avg_doc_len = if total_docs > 0 {
            total_length / total_docs as f32
        } else {
            0.0
        };

        Self {
            document_freq,
            total_docs,
            avg_doc_len,
            params,
        }
    }

    /// Score a document against a query using BM25
    pub fn score(&self, document: &CodeChunk, query: &str) -> f32 {
        let query_terms = Self::tokenize(query);
        let doc_terms = Self::tokenize(&document.content);
        let doc_length = doc_terms.len() as f32;

        let mut score = 0.0;
        let mut doc_term_freq = HashMap::new();

        // Count term frequencies in document
        for term in &doc_terms {
            *doc_term_freq.entry(term.clone()).or_insert(0) += 1;
        }

        // Calculate BM25 score for each query term
        for query_term in &query_terms {
            let tf = *doc_term_freq.get(query_term).unwrap_or(&0) as f32;
            let df = *self.document_freq.get(query_term).unwrap_or(&0) as f32;

            if df > 0.0 {
                let idf = if self.total_docs > 1 {
                    // Standard BM25 IDF for multiple documents
                    ((self.total_docs as f32 - df + 0.5) / (df + 0.5)).ln()
                } else {
                    // Simplified IDF for single document (always positive)
                    1.0
                };

                let tf_normalized = (tf * (self.params.k1 + 1.0))
                    / (tf
                        + self.params.k1
                            * (1.0 - self.params.b
                                + self.params.b * doc_length / self.avg_doc_len));

                score += idf * tf_normalized;
            }
        }

        score
    }

    /// Tokenize text into terms (simple whitespace and punctuation splitting)
    pub(crate) fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .flat_map(|word| {
                word.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>()
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .filter(|token| token.len() > BM25_TOKEN_MIN_LENGTH) // Filter out very short tokens
            .collect()
    }
}
