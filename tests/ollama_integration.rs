//! Integration tests for Ollama embedding provider with various vector stores
//!
//! These tests verify that the MCP server works end-to-end with real Ollama embeddings
//! and different vector store backends. They require Ollama to be running locally.

use mcp_context_browser::core::types::{Embedding, SearchResult};
use mcp_context_browser::providers::{EmbeddingProvider, VectorStoreProvider};
use mcp_context_browser::services::{ContextService, IndexingService, SearchService};
use std::sync::Arc;

/// Test utilities for Ollama integration tests
mod test_utils {
    use super::*;

    pub async fn create_ollama_provider() -> Option<Arc<dyn EmbeddingProvider>> {
        // Try to create Ollama provider - return None if Ollama is not available
        match mcp_context_browser::providers::OllamaEmbeddingProvider::new(
            "http://localhost:11434".to_string(),
            "nomic-embed-text".to_string(),
        ) {
            Ok(provider) => Some(Arc::new(provider)),
            Err(_) => None, // Ollama not available, skip test
        }
    }

    pub fn create_test_embedding(id: usize, dimensions: usize) -> Embedding {
        // Create embeddings that are more similar for closer IDs
        let base_value = id as f32 * 0.5;
        Embedding {
            vector: (0..dimensions)
                .map(|i| base_value + (i as f32) * 0.01)
                .collect(),
            model: "nomic-embed-text".to_string(),
            dimensions,
        }
    }

    pub fn create_test_metadata(id: usize) -> std::collections::HashMap<String, serde_json::Value> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("id".to_string(), serde_json::json!(id.to_string()));
        metadata.insert(
            "file_path".to_string(),
            serde_json::json!(format!("test/file_{}.rs", id)),
        );
        metadata.insert("line_number".to_string(), serde_json::json!(id * 10));
        metadata.insert(
            "content".to_string(),
            serde_json::json!(format!("Test content for item {}", id)),
        );
        metadata
    }

    pub async fn setup_test_data(
        vector_store: &Arc<dyn VectorStoreProvider>,
        embedding_provider: &Arc<dyn EmbeddingProvider>,
        collection: &str,
        count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dimensions = embedding_provider.dimensions();

        // Create collection
        vector_store.create_collection(collection, dimensions).await?;

        // Add test data
        for i in 0..count {
            let embedding = create_test_embedding(i, dimensions);
            let metadata = create_test_metadata(i);
            let content = format!("Test content for item {}", i);

            vector_store
                .store_embedding(collection, embedding, metadata, content)
                .await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod ollama_in_memory_tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_with_in_memory_store() {
        let ollama_provider = match test_utils::create_ollama_provider().await {
            Some(provider) => provider,
            None => {
                println!("⚠️  Ollama not available, skipping test");
                return;
            }
        };

        let in_memory_store = Arc::new(mcp_context_browser::providers::InMemoryVectorStoreProvider::new());

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            in_memory_store.clone(),
        ));

        // Setup test data
        let collection = "ollama_in_memory_test";
        test_utils::setup_test_data(&in_memory_store, &ollama_provider, collection, 5).await
            .expect("Failed to setup test data");

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search
        let query = "test content";
        let results = search_service.search(collection, query, 3).await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find some results");
        assert!(results.len() <= 3, "Should not exceed limit");

        for result in &results {
            assert!(result.score >= 0.0 && result.score <= 1.0, "Score should be between 0 and 1");
            assert!(!result.content.is_empty(), "Content should not be empty");
            assert!(!result.file_path.is_empty(), "File path should not be empty");
        }

        println!("✅ Ollama + InMemory integration test passed");
    }
}

#[cfg(test)]
mod ollama_filesystem_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ollama_with_filesystem_store() {
        let ollama_provider = match test_utils::create_ollama_provider().await {
            Some(provider) => provider,
            None => {
                println!("⚠️  Ollama not available, skipping test");
                return;
            }
        };

        // Create temporary directory for filesystem store
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        let filesystem_store = Arc::new(
            mcp_context_browser::providers::FilesystemVectorStoreProvider::new(
                Some(temp_path),
                None, // max_vectors_per_shard
                Some(ollama_provider.dimensions()),
                None, // compression_enabled
                None, // index_cache_size
                None, // memory_mapping_enabled
            ).expect("Failed to create filesystem provider")
        );

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            filesystem_store.clone(),
        ));

        // Setup test data
        let collection = "ollama_filesystem_test";
        test_utils::setup_test_data(&filesystem_store, &ollama_provider, collection, 5).await
            .expect("Failed to setup test data");

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search
        let query = "test content";
        let results = search_service.search(collection, query, 3).await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find some results");
        assert!(results.len() <= 3, "Should not exceed limit");

        for result in &results {
            assert!(result.score >= 0.0 && result.score <= 1.0, "Score should be between 0 and 1");
        }

        println!("✅ Ollama + Filesystem integration test passed");
    }
}

#[cfg(test)]
mod ollama_indexing_tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ollama_full_indexing_workflow() {
        let ollama_provider = match test_utils::create_ollama_provider().await {
            Some(provider) => provider,
            None => {
                println!("⚠️  Ollama not available, skipping test");
                return;
            }
        };

        let in_memory_store = Arc::new(mcp_context_browser::providers::InMemoryVectorStoreProvider::new());

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            in_memory_store.clone(),
        ));

        // Create indexing service
        let indexing_service = IndexingService::new(context_service.clone())
            .expect("Failed to create indexing service");

        // Create temporary directory with test files
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let test_file_path = temp_dir.path().join("test.rs");
        std::fs::write(&test_file_path, r#"
// Test Rust file for indexing
fn main() {
    println!("Hello, World!");
    let x = 42;
    let y = calculate(x);
}

fn calculate(value: i32) -> i32 {
    value * 2
}

struct TestStruct {
    field: String,
}

impl TestStruct {
    fn new() -> Self {
        Self {
            field: "test".to_string(),
        }
    }
}
"#).expect("Failed to write test file");

        // Index the directory
        let collection = "ollama_indexing_test";
        let chunk_count = indexing_service.index_directory(temp_dir.path(), collection).await
            .expect("Indexing failed");

        assert!(chunk_count > 0, "Should have indexed some chunks");

        // Create search service and test search
        let search_service = SearchService::new(context_service);

        let results = search_service.search(collection, "calculate function", 5).await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find function definition");

        // Test different queries
        let queries = vec![
            "struct definition",
            "main function",
            "field",
            "println",
        ];

        for query in queries {
            let results = search_service.search(collection, query, 3).await
                .expect("Search failed");
            assert!(!results.is_empty(), "Should find results for query: {}", query);
        }

        println!("✅ Ollama full indexing workflow test passed");
    }
}