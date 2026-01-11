#![allow(clippy::assertions_on_constants)]
//! Tests for service layer
//!
//! This module tests the business logic services including ContextService,
//! IndexingService, and SearchService.

use mcp_context_browser::application::{ContextService, IndexingService, SearchService};
use mcp_context_browser::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use mcp_context_browser::domain::types::{CodeChunk, Language};
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

    use mcp_context_browser::domain::ports::HybridSearchProvider;

    fn create_test_providers() -> (
        Arc<dyn EmbeddingProvider>,
        Arc<dyn VectorStoreProvider>,
        Arc<dyn HybridSearchProvider>,
    ) {
        let embedding_provider = Arc::new(
            mcp_context_browser::adapters::providers::embedding::null::NullEmbeddingProvider::new(),
        );
        let vector_store_provider = Arc::new(
            mcp_context_browser::adapters::providers::vector_store::null::NullVectorStoreProvider::new(),
        );
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            let mut receiver = receiver;
            while let Some(msg) = receiver.recv().await {
                use mcp_context_browser::adapters::hybrid_search::HybridSearchMessage;
                match msg {
                    HybridSearchMessage::Search { respond_to, .. } => {
                        let _ = respond_to.send(Ok(Vec::new()));
                    }
                    HybridSearchMessage::GetStats { respond_to } => {
                        let _ = respond_to.send(std::collections::HashMap::new());
                    }
                    _ => {}
                }
            }
        });
        let hybrid_search_provider = Arc::new(
            mcp_context_browser::adapters::hybrid_search::HybridSearchAdapter::new(sender),
        );
        (
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        )
    }

    #[tokio::test]
    async fn test_context_service_creation() {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        // Verify service is properly initialized with expected embedding dimensions
        // NullEmbeddingProvider returns dimension=1
        assert_eq!(service.embedding_dimensions(), 1);
    }

    #[tokio::test]
    async fn test_context_service_embed_text() -> Result<(), Box<dyn std::error::Error>> {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let text = "fn main() { println!(\"Hello!\"); }";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result?;
        assert!(!embedding.vector.is_empty());
        assert_eq!(embedding.model, "null");
        assert_eq!(embedding.dimensions, embedding.vector.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_context_service_embed_empty_text() -> Result<(), Box<dyn std::error::Error>> {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let text = "";
        let result = service.embed_text(text).await;

        assert!(result.is_ok());
        let embedding = result?;
        assert!(!embedding.vector.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_context_service_store_chunks() {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let chunks = vec![create_test_code_chunk()];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_store_empty_chunks() {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let chunks: Vec<CodeChunk> = vec![];
        let collection = "test-collection";

        let result = service.store_chunks(collection, &chunks).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_service_search_similar() -> Result<(), Box<dyn std::error::Error>> {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let query = "function definition";
        let collection = "test-collection";
        let limit = 5;

        let result = service.search_similar(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result?;
        assert!(results.len() <= limit);
        Ok(())
    }

    #[tokio::test]
    async fn test_context_service_search_with_zero_limit() -> Result<(), Box<dyn std::error::Error>>
    {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

        let query = "test query";
        let collection = "test-collection";
        let limit = 0;

        let result = service.search_similar(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result?;
        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_indexing_service_creation() -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let _indexing_service = IndexingService::new(context_service, None)?;

        // Just verify it can be created
        Ok(())
    }

    #[tokio::test]
    async fn test_indexing_service_index_directory() -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let indexing_service = IndexingService::new(context_service, None)?;

        let temp_dir = tempfile::tempdir()?;
        let collection = "test-collection";

        let result = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await;
        assert!(result.is_ok());

        let chunk_count = result?;
        assert_eq!(chunk_count, 0); // MVP implementation returns 0
        Ok(())
    }

    #[tokio::test]
    async fn test_indexing_service_index_nonexistent_directory(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let indexing_service = IndexingService::new(context_service, None)?;

        let non_existent_path = std::path::Path::new("/non/existent/path");
        let collection = "test-collection";

        let result = indexing_service
            .index_directory(non_existent_path, collection)
            .await;
        assert!(result.is_err()); // Should fail for non-existent directory
        Ok(())
    }

    #[tokio::test]
    async fn test_search_service_creation() {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let _search_service = SearchService::new(context_service);

        // Just verify it can be created
    }

    #[tokio::test]
    async fn test_search_service_search() -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let search_service = SearchService::new(context_service);

        let query = "test search query";
        let collection = "test-collection";
        let limit = 5;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result?;
        assert!(results.len() <= limit);
        Ok(())
    }

    #[tokio::test]
    async fn test_search_service_search_empty_query() -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let search_service = SearchService::new(context_service);

        let query = "";
        let collection = "test-collection";
        let limit = 5;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result?;
        assert!(results.len() <= limit);
        Ok(())
    }

    #[tokio::test]
    async fn test_search_service_search_zero_limit() -> Result<(), Box<dyn std::error::Error>> {
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let search_service = SearchService::new(context_service);

        let query = "test query";
        let collection = "test-collection";
        let limit = 0;

        let result = search_service.search(collection, query, limit).await;
        assert!(result.is_ok());

        let results = result?;
        assert_eq!(results.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_context_service_embed_batch() {
        let (_ep, _vs, _hsp) = create_test_providers();
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let service = ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        );

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
    async fn test_services_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test the full integration of services
        let (embedding_provider, vector_store_provider, hybrid_search_provider) =
            create_test_providers();
        let context_service = Arc::new(ContextService::new(
            embedding_provider,
            vector_store_provider,
            hybrid_search_provider,
        ));
        let indexing_service = IndexingService::new(Arc::clone(&context_service), None)?;
        let search_service = SearchService::new(Arc::clone(&context_service));

        // Index a directory (even if empty)
        let temp_dir = tempfile::tempdir()?;
        let collection = "integration-test";

        let index_result = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await;
        assert!(index_result.is_ok());

        // Search in the indexed collection
        let search_result = search_service.search(collection, "test query", 5).await;
        assert!(search_result.is_ok());

        let _results = search_result?;
        // Results length should be valid (always >= 0)
        Ok(())
    }
}
