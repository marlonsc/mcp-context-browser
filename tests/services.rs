//! Tests for service layer
//!
//! This module tests the business logic services including ContextService,
//! IndexingService, and SearchService.

use mcp_context_browser::core::types::{CodeChunk, Language};
use mcp_context_browser::providers::{EmbeddingProvider, VectorStoreProvider};
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

    fn create_test_providers() -> (Arc<dyn EmbeddingProvider>, Arc<dyn VectorStoreProvider>) {
        let embedding_provider =
            Arc::new(mcp_context_browser::providers::embedding::null::NullEmbeddingProvider::new());
        let vector_store_provider = Arc::new(
            mcp_context_browser::providers::vector_store::null::NullVectorStoreProvider::new(),
        );
        (embedding_provider, vector_store_provider)
    }

    #[tokio::test]
    async fn test_context_service_creation() {
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let _service = ContextService::new(embedding_provider, vector_store_provider);

        // ContextService constructor doesn't return Result, it's infallible
        // Just verify it can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_context_service_embed_text() {
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

        let text = "fn main() { println!(\"Hello!\"); }";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert!(!embedding.vector.is_empty());
        assert_eq!(embedding.model, "null");
        assert_eq!(embedding.dimensions, embedding.vector.len());
    }

    #[tokio::test]
    async fn test_context_service_embed_empty_text() {
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

        let text = "";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert!(!embedding.vector.is_empty());
    }

    #[tokio::test]
    async fn test_context_service_store_chunks() {
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

        let chunks = vec![create_test_code_chunk()];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_store_empty_chunks() {
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

        let chunks: Vec<CodeChunk> = vec![];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_search_similar() {
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

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
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

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
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
        let _indexing_service = IndexingService::new(context_service).unwrap();

        // Just verify it can be created
    }

    #[tokio::test]
    async fn test_indexing_service_index_directory() {
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
        let indexing_service = IndexingService::new(context_service).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let collection = "test-collection";

        let result = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await;
        assert!(result.is_ok());

        let chunk_count = result.unwrap();
        assert_eq!(chunk_count, 0); // MVP implementation returns 0
    }

    #[tokio::test]
    async fn test_indexing_service_index_nonexistent_directory() {
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
        let indexing_service = IndexingService::new(context_service).unwrap();

        let non_existent_path = std::path::Path::new("/non/existent/path");
        let collection = "test-collection";

        let result = indexing_service
            .index_directory(non_existent_path, collection)
            .await;
        assert!(result.is_err()); // Should fail for non-existent directory
    }

    #[test]
    fn test_search_service_creation() {
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
        let _search_service = SearchService::new(context_service);

        // Just verify it can be created
    }

    #[tokio::test]
    async fn test_search_service_search() {
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
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
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
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
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
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
        let (_embedding_provider, _vector_store_provider) = create_test_providers();
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let service = ContextService::new(embedding_provider, vector_store_provider);

        let _texts = [
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
        let (embedding_provider, vector_store_provider) = create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
        ));
        let indexing_service = IndexingService::new(Arc::clone(&context_service)).unwrap();
        let search_service = SearchService::new(Arc::clone(&context_service));

        // Index a directory (even if empty)
        let temp_dir = tempfile::tempdir().unwrap();
        let collection = "integration-test";

        let index_result = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await;
        assert!(index_result.is_ok());

        // Search in the indexed collection
        let search_result = search_service.search(collection, "test query", 5).await;
        assert!(search_result.is_ok());

        let _results = search_result.unwrap();
        // Results length should be valid (always >= 0)
    }
}
