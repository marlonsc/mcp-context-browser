//! Tests for hybrid search engine
//!
//! Tests for the hybrid search engine that combines BM25 and semantic search.

use mcp_context_browser::adapters::hybrid_search::HybridSearchEngine;
use mcp_context_browser::domain::types::{CodeChunk, Language, SearchResult};

#[test]
fn test_hybrid_search_engine() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    let hybrid_results = engine.hybrid_search("programming", semantic_results, 10)?;

    assert_eq!(hybrid_results.len(), 1);
    assert!(hybrid_results[0].hybrid_score > 0.0);
    assert!(hybrid_results[0].bm25_score >= 0.0);
    assert_eq!(hybrid_results[0].semantic_score, 0.8);
    Ok(())
}
