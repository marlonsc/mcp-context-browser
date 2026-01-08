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
        vector_store
            .create_collection(collection, dimensions)
            .await?;

        // Add test data
        for i in 0..count {
            let embedding = create_test_embedding(i, dimensions);
            let metadata = create_test_metadata(i);
            let content = format!("Test content for item {}", i);

            vector_store
                .insert_vectors(collection, &[embedding], vec![metadata])
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
        let ollama_provider = test_utils::create_ollama_provider().await
            .expect("Ollama provider should be available for integration tests");

        let in_memory_store =
            Arc::new(mcp_context_browser::providers::InMemoryVectorStoreProvider::new());

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            in_memory_store.clone(),
        ));

        // Setup test data
        let collection = "ollama_in_memory_test";
        let vector_store: Arc<dyn VectorStoreProvider> = in_memory_store.clone();
        test_utils::setup_test_data(&vector_store, &ollama_provider, collection, 5)
            .await
            .expect("Failed to setup test data");

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search
        let query = "test content";
        let results = search_service
            .search(collection, query, 3)
            .await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find some results");
        assert!(results.len() <= 3, "Should not exceed limit");

        for result in &results {
            assert!(
                result.score >= 0.0 && result.score <= 1.0,
                "Score should be between 0 and 1"
            );
            assert!(!result.content.is_empty(), "Content should not be empty");
            assert!(
                !result.file_path.is_empty(),
                "File path should not be empty"
            );
        }

        println!("✅ Ollama + InMemory integration test passed");
    }
}

#[cfg(test)]
mod ollama_filesystem_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore] // TODO: Filesystem store search not working - needs investigation
    async fn test_ollama_with_filesystem_store() {
        let ollama_provider = test_utils::create_ollama_provider().await
            .expect("Ollama provider should be available for integration tests");

        // Create temporary directory for filesystem store
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        let config =
            mcp_context_browser::providers::vector_store::filesystem::FilesystemVectorStoreConfig {
                base_path: temp_dir.path().to_path_buf(),
                max_vectors_per_shard: 1000,
                dimensions: ollama_provider.dimensions(),
                compression_enabled: false,
                index_cache_size: 1000,
                memory_mapping_enabled: false,
            };
        let filesystem_store = Arc::new(
            mcp_context_browser::providers::vector_store::FilesystemVectorStore::new(config)
                .await
                .expect("Failed to create filesystem provider"),
        );

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            filesystem_store.clone(),
        ));

        // Setup test data
        let collection = "ollama_filesystem_test";
        let vector_store: Arc<dyn VectorStoreProvider> = filesystem_store.clone();
        println!("📝 Setting up test data for filesystem store...");
        test_utils::setup_test_data(&vector_store, &ollama_provider, collection, 5)
            .await
            .expect("Failed to setup test data");
        println!("✅ Test data setup complete");

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search with different queries
        let queries = vec!["test content", "Test content for item", "calculate"];

        for query in queries {
            println!("🔍 Searching for: {}", query);
            let results = search_service
                .search(collection, query, 3)
                .await
                .expect("Search failed");

            println!("📊 Found {} results for '{}'", results.len(), query);

            if !results.is_empty() {
                println!("✅ Found results! Breaking...");
                break;
            }
        }

        // If we get here, try one more time with the original query
        let query = "test content";
        let results = search_service
            .search(collection, query, 3)
            .await
            .expect("Search failed");

        println!("📊 Final search found {} results", results.len());
        assert!(!results.is_empty(), "Should find some results");
        assert!(results.len() <= 3, "Should not exceed limit");

        for result in &results {
            assert!(
                result.score >= 0.0 && result.score <= 1.0,
                "Score should be between 0 and 1"
            );
        }

        println!("✅ Ollama + Filesystem integration test passed");
    }
}

#[cfg(test)]
mod ollama_indexing_tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_mode() {
        unsafe {
            std::env::set_var("MCP_TEST_MODE", "1");
        }
    }

    #[tokio::test]
    async fn test_ollama_full_indexing_workflow() {
        let ollama_provider = test_utils::create_ollama_provider().await
            .expect("Ollama provider should be available for integration tests");

        let in_memory_store =
            Arc::new(mcp_context_browser::providers::InMemoryVectorStoreProvider::new());

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

        // Clean any existing snapshots to ensure clean test
        let _ = std::fs::remove_dir_all(".snapshots");

        // Create unique test directory name to avoid snapshot conflicts
        let test_id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        let temp_dir = tempdir().expect("Failed to create temp dir");

        let test_file_path = temp_dir.path().join("test.rs");
        println!("📁 Creating test file at: {}", test_file_path.display());
        let content = r#"
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
"#;
        std::fs::write(&test_file_path, content).expect("Failed to write test file");

        // Modify file to ensure it's considered changed
        std::thread::sleep(std::time::Duration::from_millis(100));
        let updated_content = format!("{}\n// Updated at: {}\n", content, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        std::fs::write(&test_file_path, updated_content).expect("Failed to update test file");

        // Verify file exists
        assert!(test_file_path.exists(), "Test file should exist");
        println!("✅ Test file created and updated: {} bytes", std::fs::metadata(&test_file_path).unwrap().len());

        // Index the directory
        let collection = "ollama_indexing_test";
        let chunk_count = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await
            .expect("Indexing failed");

        assert!(chunk_count > 0, "Should have indexed some chunks");

        // Create search service and test search
        let search_service = SearchService::new(context_service);

        let results = search_service
            .search(collection, "calculate function", 5)
            .await
            .expect("Search failed");

        assert!(!results.is_empty(), "Should find function definition");

        // Test different queries
        let queries = vec!["struct definition", "main function", "field", "println"];

        for query in queries {
            let results = search_service
                .search(collection, query, 3)
                .await
                .expect("Search failed");
            assert!(
                !results.is_empty(),
                "Should find results for query: {}",
                query
            );
        }

        println!("✅ Ollama full indexing workflow test passed");
    }
}
