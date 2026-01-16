//! Tests for SearchRepository port trait
//!
//! Validates the contract of the search repository port.

use async_trait::async_trait;
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::repositories::search_repository::{SearchRepository, SearchStats};
use mcb_domain::value_objects::SearchResult;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Mock implementation of SearchRepository for testing
struct MockSearchRepository {
    query_count: AtomicU64,
    should_fail: bool,
}

impl MockSearchRepository {
    fn new() -> Self {
        Self {
            query_count: AtomicU64::new(0),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            query_count: AtomicU64::new(0),
            should_fail: true,
        }
    }
}

#[async_trait]
impl SearchRepository for MockSearchRepository {
    async fn semantic_search(
        &self,
        _collection: &str,
        _query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        self.query_count.fetch_add(1, Ordering::SeqCst);

        // Return mock results
        let results: Vec<SearchResult> = (0..limit.min(5))
            .map(|i| SearchResult {
                id: format!("semantic_{}", i),
                file_path: format!("src/module_{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                content: format!("fn function_{}() {{ }}", i),
                score: 0.95 - (i as f64 * 0.05),
                language: "rust".to_string(),
            })
            .collect();

        Ok(results)
    }

    async fn index_for_hybrid_search(&self, _chunks: &[CodeChunk]) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn hybrid_search(
        &self,
        _collection: &str,
        _query: &str,
        _query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        self.query_count.fetch_add(1, Ordering::SeqCst);

        // Return mock results with combined scoring
        let results: Vec<SearchResult> = (0..limit.min(5))
            .map(|i| SearchResult {
                id: format!("hybrid_{}", i),
                file_path: format!("src/module_{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                content: format!("fn function_{}() {{ }}", i),
                score: 0.98 - (i as f64 * 0.03), // Higher scores for hybrid
                language: "rust".to_string(),
            })
            .collect();

        Ok(results)
    }

    async fn clear_index(&self, _collection: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn stats(&self) -> Result<SearchStats> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }

        Ok(SearchStats {
            total_queries: self.query_count.load(Ordering::SeqCst),
            avg_response_time_ms: 45.5,
            cache_hit_rate: 0.75,
            indexed_documents: 1000,
        })
    }
}

#[test]
fn test_search_repository_trait_object() {
    // Verify that SearchRepository can be used as a trait object
    let _repository: Arc<dyn SearchRepository> = Arc::new(MockSearchRepository::new());
}

#[test]
fn test_search_stats_creation() {
    let stats = SearchStats {
        total_queries: 100,
        avg_response_time_ms: 50.0,
        cache_hit_rate: 0.8,
        indexed_documents: 5000,
    };

    assert_eq!(stats.total_queries, 100);
    assert_eq!(stats.avg_response_time_ms, 50.0);
    assert_eq!(stats.cache_hit_rate, 0.8);
    assert_eq!(stats.indexed_documents, 5000);
}

#[tokio::test]
async fn test_search_repository_semantic_search() {
    let repository = MockSearchRepository::new();

    let query_vector = vec![0.1; 384];
    let results = repository
        .semantic_search("test_collection", &query_vector, 5, None)
        .await;

    assert!(results.is_ok());
    let results = results.expect("Expected results");
    assert!(!results.is_empty());
    assert!(results.len() <= 5);

    // Verify results are ordered by score (highest first)
    for i in 1..results.len() {
        assert!(results[i - 1].score >= results[i].score);
    }
}

#[tokio::test]
async fn test_search_repository_hybrid_search() {
    let repository = MockSearchRepository::new();

    let query_vector = vec![0.1; 384];
    let results = repository
        .hybrid_search("test_collection", "find functions", &query_vector, 5)
        .await;

    assert!(results.is_ok());
    let results = results.expect("Expected results");
    assert!(!results.is_empty());

    // Hybrid search should include both semantic and keyword relevance
    for result in &results {
        assert!(result.id.starts_with("hybrid_"));
    }
}

#[tokio::test]
async fn test_search_repository_stats_tracking() {
    let repository = MockSearchRepository::new();

    // Initial stats
    let stats = repository.stats().await;
    assert!(stats.is_ok());
    assert_eq!(stats.expect("Expected stats").total_queries, 0);

    // After some searches
    let query_vector = vec![0.1; 384];
    let _ = repository
        .semantic_search("test", &query_vector, 5, None)
        .await;
    let _ = repository
        .hybrid_search("test", "query", &query_vector, 5)
        .await;

    let stats = repository.stats().await;
    assert!(stats.is_ok());
    assert_eq!(stats.expect("Expected stats").total_queries, 2);
}

#[tokio::test]
async fn test_search_repository_index_for_hybrid() {
    let repository = MockSearchRepository::new();

    let chunks: Vec<CodeChunk> = vec![];
    let result = repository.index_for_hybrid_search(&chunks).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_repository_clear_index() {
    let repository = MockSearchRepository::new();

    let result = repository.clear_index("test_collection").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_repository_failure_handling() {
    let repository = MockSearchRepository::failing();

    let query_vector = vec![0.1; 384];

    let result = repository
        .semantic_search("test", &query_vector, 5, None)
        .await;
    assert!(result.is_err());

    let result = repository
        .hybrid_search("test", "query", &query_vector, 5)
        .await;
    assert!(result.is_err());

    let result = repository.stats().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_search_repository_with_filter() {
    let repository = MockSearchRepository::new();

    let query_vector = vec![0.1; 384];
    let results = repository
        .semantic_search("test", &query_vector, 5, Some("language == 'rust'"))
        .await;

    assert!(results.is_ok());
}
