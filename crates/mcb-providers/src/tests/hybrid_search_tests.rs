//! Tests for hybrid search providers

use crate::constants::{HYBRID_SEARCH_BM25_WEIGHT, HYBRID_SEARCH_SEMANTIC_WEIGHT};
use crate::hybrid_search::{BM25Params, BM25Scorer, HybridSearchEngine, NullHybridSearchProvider};
use mcb_domain::entities::CodeChunk;
use mcb_domain::ports::providers::HybridSearchProvider;
use mcb_domain::value_objects::SearchResult;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_chunk(content: &str, file_path: &str, start_line: u32) -> CodeChunk {
    CodeChunk {
        id: format!("{}:{}", file_path, start_line),
        content: content.to_string(),
        file_path: file_path.to_string(),
        start_line,
        end_line: start_line + content.lines().count() as u32,
        language: "Rust".to_string(),
        metadata: serde_json::json!({}),
    }
}

fn create_test_search_result(file_path: &str, start_line: u32, score: f64) -> SearchResult {
    SearchResult {
        id: format!("{}:{}", file_path, start_line),
        content: format!("Content of {}:{}", file_path, start_line),
        file_path: file_path.to_string(),
        start_line,
        score,
        language: "Rust".to_string(),
    }
}

// ============================================================================
// BM25 Scorer Tests
// ============================================================================

#[test]
fn test_tokenize() {
    let tokens = BM25Scorer::tokenize("fn hello_world() { println!(\"Hello, World!\"); }");
    // Underscores split tokens for better code search matching
    assert!(tokens.contains(&"hello".to_string()));
    assert!(tokens.contains(&"world".to_string()));
    assert!(tokens.contains(&"println".to_string()));
    // Short tokens should be filtered (len <= 2)
    assert!(!tokens.contains(&"fn".to_string())); // len = 2, filtered by BM25_TOKEN_MIN_LENGTH
}

#[test]
fn test_bm25_scorer_creation() {
    let chunks = vec![
        create_test_chunk("fn authenticate_user() {}", "auth.rs", 1),
        create_test_chunk("fn validate_password() {}", "auth.rs", 10),
        create_test_chunk("fn hash_password() {}", "crypto.rs", 1),
    ];

    let scorer = BM25Scorer::new(&chunks, BM25Params::default());

    assert_eq!(scorer.total_docs(), 3);
    assert!(scorer.unique_terms() > 0);
    assert!(scorer.avg_doc_len() > 0.0);
}

#[test]
fn test_bm25_scoring() {
    // Use content with clearly distinct keywords that match as separate tokens
    let chunks = vec![
        create_test_chunk(
            "authenticate the user and validate their credentials with proper authentication",
            "auth.rs",
            1,
        ),
        create_test_chunk(
            "validate the password using hash function for security",
            "auth.rs",
            10,
        ),
        create_test_chunk(
            "process the data and compress it for storage optimization",
            "data.rs",
            1,
        ),
    ];

    let scorer = BM25Scorer::new(&chunks, BM25Params::default());

    // Query with terms that appear in first chunk
    let score_auth = scorer.score(&chunks[0], "authenticate user validate");
    let score_data = scorer.score(&chunks[2], "authenticate user validate");

    // Auth chunk should score highest (contains "authenticate", "user", "validate")
    // Data chunk has none of these terms
    assert!(
        score_auth > score_data,
        "Auth chunk should score higher than data chunk (auth={}, data={})",
        score_auth,
        score_data
    );
}

#[test]
fn test_bm25_batch_scoring() {
    // Use content with clearly distinct keywords
    let chunks = vec![
        create_test_chunk(
            "search through the codebase and find matching patterns",
            "search.rs",
            1,
        ),
        create_test_chunk(
            "index the documents and build inverted index structure",
            "index.rs",
            1,
        ),
    ];

    let scorer = BM25Scorer::new(&chunks, BM25Params::default());
    let chunk_refs: Vec<&CodeChunk> = chunks.iter().collect();

    // Query for "search codebase" - first chunk has these terms
    let scores = scorer.score_batch(&chunk_refs, "search codebase");

    assert_eq!(scores.len(), 2);
    // First chunk contains "search" and "codebase", second has neither
    assert!(
        scores[0] > scores[1],
        "First chunk should score higher (search={}, index={})",
        scores[0],
        scores[1]
    );
}

// ============================================================================
// Hybrid Search Engine Tests
// ============================================================================

#[tokio::test]
async fn test_hybrid_search_engine_creation() {
    let engine = HybridSearchEngine::new();
    assert!((engine.bm25_weight() - HYBRID_SEARCH_BM25_WEIGHT).abs() < f32::EPSILON);
    assert!((engine.semantic_weight() - HYBRID_SEARCH_SEMANTIC_WEIGHT).abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_index_chunks() {
    let engine = HybridSearchEngine::new();

    let chunks = vec![
        create_test_chunk("fn authenticate_user() {}", "auth.rs", 1),
        create_test_chunk("fn validate_password() {}", "auth.rs", 10),
    ];

    engine.index_chunks("test", &chunks).await.unwrap();

    let stats = engine.get_stats().await;
    assert_eq!(stats.get("collection_count"), Some(&serde_json::json!(1)));
}

#[tokio::test]
async fn test_hybrid_search() {
    let engine = HybridSearchEngine::new();

    // Index documents with clearly distinct content
    let chunks = vec![
        create_test_chunk(
            "authenticate the user and validate their credentials for secure access",
            "auth.rs",
            1,
        ),
        create_test_chunk(
            "process the data and compress it for efficient storage optimization",
            "data.rs",
            1,
        ),
    ];
    engine.index_chunks("test", &chunks).await.unwrap();

    // Semantic results: data.rs has slightly higher semantic score
    // But auth.rs has much better BM25 match for the query
    let semantic_results = vec![
        create_test_search_result("auth.rs", 1, 0.7), // Lower semantic
        create_test_search_result("data.rs", 1, 0.75), // Higher semantic
    ];

    // Query matches auth.rs content perfectly
    let results = engine
        .search(
            "test",
            "authenticate user validate credentials",
            semantic_results,
            10,
        )
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    // Auth chunk should rank higher due to strong BM25 boost overcoming semantic difference
    assert_eq!(
        results[0].file_path, "auth.rs",
        "Auth should rank first due to BM25 boost"
    );
}

#[tokio::test]
async fn test_clear_collection() {
    let engine = HybridSearchEngine::new();

    let chunks = vec![create_test_chunk("fn test() {}", "test.rs", 1)];
    engine.index_chunks("test", &chunks).await.unwrap();

    let stats = engine.get_stats().await;
    assert_eq!(stats.get("collection_count"), Some(&serde_json::json!(1)));

    engine.clear_collection("test").await.unwrap();

    let stats = engine.get_stats().await;
    assert_eq!(stats.get("collection_count"), Some(&serde_json::json!(0)));
}

#[tokio::test]
async fn test_search_without_index() {
    let engine = HybridSearchEngine::new();

    // Search without indexing should return semantic results as-is
    let semantic_results = vec![
        create_test_search_result("a.rs", 1, 0.9),
        create_test_search_result("b.rs", 1, 0.8),
    ];

    let results = engine
        .search("nonexistent", "query", semantic_results.clone(), 10)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].file_path, "a.rs");
}

// ============================================================================
// Null Hybrid Search Provider Tests
// ============================================================================

#[tokio::test]
async fn test_null_provider_index() {
    let provider = NullHybridSearchProvider::new();
    let chunks = vec![create_test_chunk("fn test() {}", "test.rs", 1)];

    // Should succeed without error
    provider.index_chunks("test", &chunks).await.unwrap();
}

#[tokio::test]
async fn test_null_provider_search_passthrough() {
    let provider = NullHybridSearchProvider::new();

    let semantic_results = vec![
        create_test_search_result("a.rs", 1, 0.9),
        create_test_search_result("b.rs", 1, 0.8),
        create_test_search_result("c.rs", 1, 0.7),
    ];

    let results = provider
        .search("test", "query", semantic_results.clone(), 2)
        .await
        .unwrap();

    // Should return first 2 results unchanged
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].file_path, "a.rs");
    assert!((results[0].score - 0.9).abs() < f64::EPSILON);
    assert_eq!(results[1].file_path, "b.rs");
}

#[tokio::test]
async fn test_null_provider_clear() {
    let provider = NullHybridSearchProvider::new();

    // Should succeed without error
    provider.clear_collection("test").await.unwrap();
}

#[tokio::test]
async fn test_null_provider_stats() {
    let provider = NullHybridSearchProvider::new();

    let stats = provider.get_stats().await;

    assert_eq!(stats.get("provider"), Some(&serde_json::json!("null")));
    assert_eq!(stats.get("bm25_enabled"), Some(&serde_json::json!(false)));
}
