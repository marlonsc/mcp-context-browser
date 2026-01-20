//! Tests for search domain services

use mcb_application::domain_services::search::SearchServiceInterface;
use mcb_application::use_cases::SearchServiceImpl;
use std::sync::Arc;

// Mock implementation for testing
#[derive(Clone)]
struct MockContextService;

#[async_trait::async_trait]
impl mcb_application::ports::services::ContextServiceInterface for MockContextService {
    async fn initialize(&self, _collection: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn store_chunks(
        &self,
        _collection: &str,
        _chunks: &[mcb_domain::entities::CodeChunk],
    ) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query: &str,
        _limit: usize,
    ) -> mcb_domain::error::Result<Vec<mcb_domain::value_objects::SearchResult>> {
        Ok(Vec::new())
    }

    async fn embed_text(
        &self,
        _text: &str,
    ) -> mcb_domain::error::Result<mcb_domain::value_objects::Embedding> {
        Ok(mcb_domain::value_objects::Embedding {
            vector: vec![0.0; 384],
            model: "mock-model".to_string(),
            dimensions: 384,
        })
    }

    async fn clear_collection(&self, _collection: &str) -> mcb_domain::error::Result<()> {
        Ok(())
    }

    async fn get_stats(&self) -> mcb_domain::error::Result<(i64, i64)> {
        Ok((0, 0))
    }

    fn embedding_dimensions(&self) -> usize {
        384
    }
}

#[test]
fn test_search_service_creation() {
    // Create a mock context service
    let context_service: Arc<
        dyn mcb_application::domain_services::search::ContextServiceInterface,
    > = Arc::new(MockContextService);

    let search_service = SearchServiceImpl::new(context_service);

    // Test that service can be created without panicking
    let _service: Box<dyn SearchServiceInterface> = Box::new(search_service);
}

#[test]
fn test_search_service_search() {
    // Create a mock context service
    let context_service: Arc<
        dyn mcb_application::domain_services::search::ContextServiceInterface,
    > = Arc::new(MockContextService);

    let search_service = SearchServiceImpl::new(context_service);

    // Test search functionality (will return empty results with mock)
    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        search_service
            .search("test-collection", "test-query", 10)
            .await
    });

    // Should succeed but return empty results
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
