//! Hybrid search combining BM25 text ranking with semantic embeddings
//!
//! This module implements a hybrid search approach that combines:
//! - BM25: Term frequency-based text ranking algorithm
//! - Semantic Embeddings: Vector similarity for semantic understanding
//!
//! The hybrid approach provides better relevance by combining lexical and semantic matching.

use crate::domain::error::Result;
use crate::domain::ports::HybridSearchProvider;
use crate::domain::types::{CodeChunk, SearchResult};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tokio::sync::{mpsc, oneshot};
use validator::Validate;

// ... (rest of the code)

/// Hybrid search adapter that implements the port
pub struct HybridSearchAdapter {
    sender: mpsc::Sender<HybridSearchMessage>,
}

impl HybridSearchAdapter {
    /// Create a new hybrid search adapter
    pub fn new(sender: mpsc::Sender<HybridSearchMessage>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl HybridSearchProvider for HybridSearchAdapter {
    async fn index_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        self.sender
            .send(HybridSearchMessage::Index {
                collection: collection.to_string(),
                chunks: chunks.to_vec(),
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send to hybrid search actor: {}",
                    e
                ))
            })
    }

    async fn search(
        &self,
        _collection: &str,
        query: &str,
        semantic_results: Vec<SearchResult>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let (respond_to, receiver) = oneshot::channel();
        self.sender
            .send(HybridSearchMessage::Search {
                query: query.to_string(),
                semantic_results,
                limit,
                respond_to,
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send search to hybrid search actor: {}",
                    e
                ))
            })?;

        let hybrid_results = receiver.await.map_err(|e| {
            crate::domain::error::Error::internal(format!(
                "Failed to receive hybrid search results: {}",
                e
            ))
        })??;

        Ok(hybrid_results
            .into_iter()
            .map(|hybrid_result| {
                let mut result = hybrid_result.result;
                result.score = hybrid_result.hybrid_score;

                let mut new_metadata = serde_json::Map::new();
                if let serde_json::Value::Object(existing) = &result.metadata {
                    new_metadata.extend(existing.clone());
                }
                new_metadata.insert(
                    "bm25_score".to_string(),
                    serde_json::json!(hybrid_result.bm25_score),
                );
                new_metadata.insert(
                    "semantic_score".to_string(),
                    serde_json::json!(hybrid_result.semantic_score),
                );
                new_metadata.insert(
                    "hybrid_score".to_string(),
                    serde_json::json!(hybrid_result.hybrid_score),
                );
                result.metadata = serde_json::Value::Object(new_metadata);

                result
            })
            .collect())
    }

    async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.sender
            .send(HybridSearchMessage::Clear {
                collection: collection.to_string(),
            })
            .await
            .map_err(|e| {
                crate::domain::error::Error::internal(format!(
                    "Failed to send clear to hybrid search actor: {}",
                    e
                ))
            })
    }

    async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let (respond_to, receiver) = oneshot::channel();
        if self
            .sender
            .send(HybridSearchMessage::GetStats { respond_to })
            .await
            .is_err()
        {
            return HashMap::new();
        }

        receiver.await.unwrap_or_default()
    }
}

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
            k1: 1.2, // Standard BM25 k1 value
            b: 0.75, // Standard BM25 b value
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
    fn tokenize(text: &str) -> Vec<String> {
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
            .filter(|token| token.len() > 2) // Filter out very short tokens
            .collect()
    }
}

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

/// Hybrid search configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, validator::Validate)]
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
            bm25_weight: 0.4,
            semantic_weight: 0.6,
            bm25_k1: 1.2,
            bm25_b: 0.75,
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
                .unwrap_or_else(|_| "0.4".to_string())
                .parse()
                .unwrap_or(0.4),
            semantic_weight: std::env::var("HYBRID_SEARCH_SEMANTIC_WEIGHT")
                .unwrap_or_else(|_| "0.6".to_string())
                .parse()
                .unwrap_or(0.6),
            bm25_k1: std::env::var("HYBRID_SEARCH_BM25_K1")
                .unwrap_or_else(|_| "1.2".to_string())
                .parse()
                .unwrap_or(1.2),
            bm25_b: std::env::var("HYBRID_SEARCH_BM25_B")
                .unwrap_or_else(|_| "0.75".to_string())
                .parse()
                .unwrap_or(0.75),
        }
    }
}

/// Hybrid search message for the actor
pub enum HybridSearchMessage {
    Index {
        collection: String,
        chunks: Vec<CodeChunk>,
    },
    Search {
        query: String,
        semantic_results: Vec<SearchResult>,
        limit: usize,
        respond_to: oneshot::Sender<Result<Vec<HybridSearchResult>>>,
    },
    Clear {
        collection: String,
    },
    GetStats {
        respond_to: oneshot::Sender<HashMap<String, serde_json::Value>>,
    },
}

/// Hybrid search actor that manages the engine state
pub struct HybridSearchActor {
    engine: HybridSearchEngine,
    receiver: mpsc::Receiver<HybridSearchMessage>,
    indexed_docs: HashMap<String, Vec<CodeChunk>>,
}

impl HybridSearchActor {
    /// Create a new hybrid search actor
    pub fn new(
        receiver: mpsc::Receiver<HybridSearchMessage>,
        bm25_weight: f32,
        semantic_weight: f32,
    ) -> Self {
        Self {
            engine: HybridSearchEngine::new(bm25_weight, semantic_weight),
            receiver,
            indexed_docs: HashMap::new(),
        }
    }

    /// Run the actor loop
    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                HybridSearchMessage::Index { collection, chunks } => {
                    let docs = self.indexed_docs.entry(collection).or_default();
                    docs.extend(chunks);

                    // Rebuild BM25 index with all documents
                    let all_docs: Vec<CodeChunk> =
                        self.indexed_docs.values().flatten().cloned().collect();
                    self.engine.index_documents(all_docs);
                }
                HybridSearchMessage::Search {
                    query,
                    semantic_results,
                    limit,
                    respond_to,
                } => {
                    let result = self.engine.hybrid_search(&query, semantic_results, limit);
                    let _ = respond_to.send(result);
                }
                HybridSearchMessage::Clear { collection } => {
                    self.indexed_docs.remove(&collection);
                    let all_docs: Vec<CodeChunk> =
                        self.indexed_docs.values().flatten().cloned().collect();
                    self.engine.index_documents(all_docs);
                }
                HybridSearchMessage::GetStats { respond_to } => {
                    let mut stats = HashMap::new();
                    stats.insert("hybrid_search_enabled".to_string(), serde_json::json!(true));
                    stats.insert(
                        "bm25_index_available".to_string(),
                        serde_json::json!(self.engine.has_bm25_index()),
                    );

                    if let Some(bm25_stats) = self.engine.get_bm25_stats() {
                        stats.extend(bm25_stats);
                    }

                    stats.insert(
                        "total_collections".to_string(),
                        serde_json::json!(self.indexed_docs.len()),
                    );
                    stats.insert(
                        "total_indexed_documents".to_string(),
                        serde_json::json!(self
                            .indexed_docs
                            .values()
                            .map(|v| v.len())
                            .sum::<usize>()),
                    );

                    let _ = respond_to.send(stats);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{CodeChunk, Language};

    #[test]
    fn test_bm25_scorer_creation() {
        let documents = vec![
            CodeChunk {
                id: "1".to_string(),
                content: "This is a test document about programming".to_string(),
                file_path: "test.rs".to_string(),
                start_line: 1,
                end_line: 5,
                language: Language::Rust,
                metadata: Default::default(),
            },
            CodeChunk {
                id: "2".to_string(),
                content: "Another document discussing programming languages".to_string(),
                file_path: "test2.rs".to_string(),
                start_line: 1,
                end_line: 5,
                language: Language::Rust,
                metadata: Default::default(),
            },
        ];

        let scorer = BM25Scorer::new(&documents, BM25Params::default());
        assert_eq!(scorer.total_docs, 2);
        assert!(scorer.document_freq.contains_key("programming"));
        assert!(scorer.avg_doc_len > 0.0);
    }

    #[test]
    fn test_bm25_scoring() {
        let documents = vec![CodeChunk {
            id: "1".to_string(),
            content: "This is a test document about programming and development".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 5,
            language: Language::Rust,
            metadata: Default::default(),
        }];

        let scorer = BM25Scorer::new(&documents, BM25Params::default());

        // Debug tokenization
        let query_tokens = BM25Scorer::tokenize("programming");
        let doc_tokens = BM25Scorer::tokenize(&documents[0].content);
        println!("Query tokens: {:?}", query_tokens);
        println!("Doc tokens: {:?}", doc_tokens);

        let score = scorer.score(&documents[0], "programming");
        println!("BM25 score for 'programming': {}", score);

        // BM25 score should be non-negative
        assert!(
            score >= 0.0,
            "BM25 score should be non-negative, got: {}",
            score
        );

        // With only one document, IDF will be low, but TF should contribute
        // Test with non-existent term (should be 0)
        let zero_score = scorer.score(&documents[0], "nonexistent");
        assert_eq!(zero_score, 0.0, "Score for non-existent term should be 0");

        // Test with a more comprehensive check
        let higher_score = scorer.score(&documents[0], "programming development");
        println!("BM25 score for 'programming development': {}", higher_score);
        assert!(
            higher_score >= score,
            "Score for multiple terms should be at least as high"
        );
    }

    #[test]
    fn test_hybrid_search_engine() {
        let mut engine = HybridSearchEngine::new(0.4, 0.6);

        let documents = vec![CodeChunk {
            id: "1".to_string(),
            content: "This is a test document about programming".to_string(),
            file_path: "test.rs".to_string(),
            start_line: 1,
            end_line: 5,
            language: Language::Rust,
            metadata: Default::default(),
        }];

        engine.index_documents(documents);

        let semantic_results = vec![SearchResult {
            id: "test-id".to_string(),
            file_path: "test.rs".to_string(),
            line_number: 1,
            content: "This is a test document about programming".to_string(),
            score: 0.8,
            metadata: Default::default(),
        }];

        let hybrid_results = engine
            .hybrid_search("programming", semantic_results, 10)
            .unwrap();

        assert_eq!(hybrid_results.len(), 1);
        assert!(hybrid_results[0].hybrid_score > 0.0);
        assert!(hybrid_results[0].bm25_score >= 0.0);
        assert_eq!(hybrid_results[0].semantic_score, 0.8);
    }

    #[test]
    fn test_hybrid_search_config() {
        let config = HybridSearchConfig::default();
        assert!(config.enabled);
        assert_eq!(config.bm25_weight, 0.4);
        assert_eq!(config.semantic_weight, 0.6);
        assert_eq!(config.bm25_k1, 1.2);
        assert_eq!(config.bm25_b, 0.75);
    }

    #[test]
    fn test_tokenization() {
        let text = "Hello, world! This is a test.";
        let tokens = BM25Scorer::tokenize(text);

        // Should contain meaningful tokens
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"this".to_string()));
        assert!(tokens.contains(&"test".to_string()));

        // Should not contain short tokens
        assert!(!tokens.contains(&"a".to_string()));
        assert!(!tokens.contains(&"is".to_string()));
    }
}
