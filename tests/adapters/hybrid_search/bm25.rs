//! Tests for BM25 text ranking algorithm
//!
//! Tests for the BM25 scorer used in hybrid search.

use mcp_context_browser::adapters::hybrid_search::{BM25Params, BM25Scorer};
use mcp_context_browser::domain::types::{CodeChunk, Language};

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

// Note: test_tokenization removed as BM25Scorer::tokenize is pub(crate)
// The tokenization behavior is tested indirectly through test_bm25_scoring
