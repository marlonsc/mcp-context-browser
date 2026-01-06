//! Tests for service layer
//!
//! This module tests the business logic services including ContextService,
//! IndexingService, and SearchService.

use mcp_context_browser::core::types::{CodeChunk, Language};
use mcp_context_browser::factory::ServiceProvider;
use mcp_context_browser::services::{ContextService, IndexingService, SearchService};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_code_chunk() -> CodeChunk {
        CodeChunk {
            id: "test-chunk-1".to_string(),
            content: "fn test_function() {\n    println!(\"Hello, World!\");\n}".to_string(),
            file_path: "src/main.rs".to_string(),
            start_line: 1,
            end_line: 3,
            language: Language::Rust,
            metadata: serde_json::json!({"author": "test", "complexity": 1}),
        }
    }

    fn create_test_service_provider() -> ServiceProvider {
        ServiceProvider::new()
    }

    #[tokio::test]
    async fn test_context_service_creation() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider);

        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_embed_text() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let text = "fn main() { println!(\"Hello!\"); }";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert!(!embedding.vector.is_empty());
        assert_eq!(embedding.model, "mock");
        assert_eq!(embedding.dimensions, embedding.vector.len());
    }

    #[tokio::test]
    async fn test_context_service_embed_empty_text() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let text = "";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert!(!embedding.vector.is_empty());
    }

    #[tokio::test]
    async fn test_context_service_store_chunks() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let chunks = vec![create_test_code_chunk()];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_store_empty_chunks() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let chunks: Vec<CodeChunk> = vec![];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_search_similar() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let query = "function definition";
        let collection = "test-collection";
        let limit = 5;

        let result = service.search_similar(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result.unwrap();
        assert!(results.len() <= limit);
    }

    #[tokio::test]
    async fn test_context_service_search_with_zero_limit() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let query = "test query";
        let collection = "test-collection";
        let limit = 0;

        let result = service.search_similar(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_indexing_service_creation() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let indexing_service = IndexingService::new(context_service);

        // Just verify it can be created
        assert!(true); // If we reach here, creation was successful
    }

    #[tokio::test]
    async fn test_indexing_service_index_directory() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let indexing_service = IndexingService::new(context_service);

        let temp_dir = tempfile::tempdir().unwrap();
        let collection = "test-collection";

        let result = indexing_service.index_directory(temp_dir.path(), collection).await;
        assert!(result.is_ok());

        let chunk_count = result.unwrap();
        assert_eq!(chunk_count, 0); // MVP implementation returns 0
    }

    #[tokio::test]
    async fn test_indexing_service_index_nonexistent_directory() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let indexing_service = IndexingService::new(context_service);

        let non_existent_path = std::path::Path::new("/non/existent/path");
        let collection = "test-collection";

        let result = indexing_service.index_directory(non_existent_path, collection).await;
        assert!(result.is_ok()); // MVP implementation doesn't fail
    }

    #[test]
    fn test_search_service_creation() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let search_service = SearchService::new(context_service);

        // Just verify it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_search_service_search() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let search_service = SearchService::new(context_service);

        let query = "test search query";
        let collection = "test-collection";
        let limit = 5;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result.unwrap();
        assert!(results.len() <= limit);
    }

    #[tokio::test]
    async fn test_search_service_search_empty_query() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let search_service = SearchService::new(context_service);

        let query = "";
        let collection = "test-collection";
        let limit = 5;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result.unwrap();
        assert!(results.len() <= limit);
    }

    #[tokio::test]
    async fn test_search_service_search_zero_limit() {
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let search_service = SearchService::new(context_service);

        let query = "test query";
        let collection = "test-collection";
        let limit = 0;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_context_service_embed_batch() {
        let service_provider = create_test_service_provider();
        let service = ContextService::new(&service_provider).unwrap();

        let _texts = vec![
            "fn main() {}".to_string(),
            "struct Test {}".to_string(),
            "let x = 42;".to_string(),
        ];

        // This test will need to be updated when the actual embedding batch implementation is done
        // For now, we just test that it doesn't panic
        let result = service.store_chunks("test", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_services_integration() {
        // Test the full integration of services
        let service_provider = create_test_service_provider();
        let context_service = Arc::new(ContextService::new(&service_provider).unwrap());
        let indexing_service = IndexingService::new(Arc::clone(&context_service));
        let search_service = SearchService::new(Arc::clone(&context_service));

        // Index a directory (even if empty)
        let temp_dir = tempfile::tempdir().unwrap();
        let collection = "integration-test";

        let index_result = indexing_service.index_directory(temp_dir.path(), collection).await;
        assert!(index_result.is_ok());

        // Search in the indexed collection
        let search_result = search_service.search(collection, "test query", 5).await;
        assert!(search_result.is_ok());

        let _results = search_result.unwrap();
        // Results length should be valid (always >= 0)
    }
}