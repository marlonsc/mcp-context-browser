//! Integration tests for Ollama embedding provider with various vector stores
//!
//! These tests verify that the MCP server works end-to-end with real Ollama embeddings
//! and different vector store backends. They require Ollama to be running locally.

use mcp_context_browser::application::{ContextService, IndexingService, SearchService};
use mcp_context_browser::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use mcp_context_browser::domain::types::Embedding;
use std::sync::Arc;

fn dummy_hybrid_search() -> Arc<dyn mcp_context_browser::domain::ports::HybridSearchProvider> {
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    // Run a minimal actor that just returns semantic results as is
    tokio::spawn(async move {
        let mut receiver = receiver;
        while let Some(msg) = receiver.recv().await {
            use mcp_context_browser::adapters::hybrid_search::HybridSearchMessage;
            match msg {
                HybridSearchMessage::Search {
                    semantic_results,
                    respond_to,
                    limit,
                    ..
                } => {
                    use mcp_context_browser::adapters::hybrid_search::HybridSearchResult;
                    let hybrid_results = semantic_results
                        .into_iter()
                        .take(limit)
                        .map(|result| HybridSearchResult {
                            bm25_score: 0.0,
                            semantic_score: result.score,
                            hybrid_score: result.score,
                            result,
                        })
                        .collect();
                    let _ = respond_to.send(Ok(hybrid_results));
                }
                HybridSearchMessage::GetStats { respond_to } => {
                    let _ = respond_to.send(std::collections::HashMap::new());
                }
                _ => {}
            }
        }
    });
    Arc::new(mcp_context_browser::adapters::HybridSearchAdapter::new(
        sender,
    ))
}

/// Test utilities for Ollama integration tests
mod test_utils {
    use super::*;

    pub async fn create_ollama_provider() -> Option<Arc<dyn EmbeddingProvider>> {
        // Try to create Ollama provider - return None if Ollama is not available
        match mcp_context_browser::adapters::providers::OllamaEmbeddingProvider::new(
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
            let _content = format!("Test content for item {}", i);

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
    async fn test_ollama_with_in_memory_store() -> Result<(), Box<dyn std::error::Error>> {
        let ollama_provider = test_utils::create_ollama_provider()
            .await
            .ok_or("Ollama provider should be available for integration tests")?;

        let in_memory_store =
            Arc::new(mcp_context_browser::adapters::providers::InMemoryVectorStoreProvider::new());

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            in_memory_store.clone(),
            dummy_hybrid_search(),
        ));

        // Setup test data
        let collection = "ollama_in_memory_test";
        let vector_store: Arc<dyn VectorStoreProvider> = in_memory_store.clone();
        test_utils::setup_test_data(&vector_store, &ollama_provider, collection, 5).await?;

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search
        let query = "test content";
        let results = search_service.search(collection, query, 3).await?;

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

        Ok(())
    }
}

#[cfg(test)]
mod ollama_filesystem_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ollama_with_filesystem_store() -> Result<(), Box<dyn std::error::Error>> {
        let ollama_provider = test_utils::create_ollama_provider()
            .await
            .ok_or("Ollama provider should be available for integration tests")?;

        // Create temporary directory for filesystem store
        let temp_dir = tempdir()?;
        let _temp_path = temp_dir.path().to_string_lossy().to_string();

        let config =
            mcp_context_browser::adapters::providers::vector_store::filesystem::FilesystemVectorStoreConfig {
                base_path: temp_dir.path().to_path_buf(),
                max_vectors_per_shard: 1000,
                dimensions: ollama_provider.dimensions(),
                compression_enabled: false,
                index_cache_size: 1000,
                memory_mapping_enabled: false,
            };
        let filesystem_store = Arc::new(
            mcp_context_browser::adapters::providers::vector_store::FilesystemVectorStore::new(
                config,
            )
            .await?,
        );

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            filesystem_store.clone(),
            dummy_hybrid_search(),
        ));

        // Setup test data
        let collection = "ollama_filesystem_test";
        let vector_store: Arc<dyn VectorStoreProvider> = filesystem_store.clone();
        println!("📝 Setting up test data for filesystem store...");
        test_utils::setup_test_data(&vector_store, &ollama_provider, collection, 5).await?;
        println!("✅ Test data setup complete");

        // Create search service
        let search_service = SearchService::new(context_service);

        // Test search with different queries
        let queries = vec!["test content", "Test content for item", "calculate"];

        for query in queries {
            println!("🔍 Searching for: {}", query);
            let results = search_service.search(collection, query, 3).await?;

            println!("📊 Found {} results for '{}'", results.len(), query);

            if !results.is_empty() {
                println!("✅ Found results! Breaking...");
                break;
            }
        }

        // If we get here, try one more time with the original query
        let query = "test content";
        let results = search_service.search(collection, query, 3).await?;

        println!("📊 Final search found {} results", results.len());
        assert!(!results.is_empty(), "Should find some results");
        assert!(results.len() <= 3, "Should not exceed limit");

        for result in &results {
            assert!(
                result.score >= 0.0 && result.score <= 1.0,
                "Score should be between 0 and 1"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod ollama_indexing_tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ollama_full_indexing_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let ollama_provider = test_utils::create_ollama_provider()
            .await
            .ok_or("Ollama provider should be available for integration tests")?;

        let in_memory_store =
            Arc::new(mcp_context_browser::adapters::providers::InMemoryVectorStoreProvider::new());

        // Create context service
        let context_service = Arc::new(ContextService::new(
            ollama_provider.clone(),
            in_memory_store.clone(),
            dummy_hybrid_search(),
        ));

        // Create indexing service
        let indexing_service = IndexingService::new(context_service.clone(), None)?;

        // Create temporary directory with test files
        let temp_dir = tempdir()?;

        // Clean any existing snapshots to ensure clean test
        let _ = std::fs::remove_dir_all(".snapshots");

        // Create test file with Rust code
        let test_file_path = temp_dir.path().join("test.rs");
        let content = r#"// Test Rust file for indexing
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
        // Write the test file to disk
        std::fs::write(&test_file_path, content)?;
        println!("📝 Created test file at: {}", test_file_path.display());

        // Index the directory
        let collection = "ollama_indexing_test";
        let chunk_count = indexing_service
            .index_directory(temp_dir.path(), collection)
            .await?;

        assert!(chunk_count > 0, "Should have indexed some chunks");

        // Create search service and test search
        let search_service = SearchService::new(context_service);

        let results = search_service
            .search(collection, "calculate function", 5)
            .await?;

        assert!(!results.is_empty(), "Should find function definition");

        // Test different queries
        let queries = vec!["struct definition", "main function", "field", "println"];

        for query in queries {
            let results = search_service.search(collection, query, 3).await?;
            assert!(
                !results.is_empty(),
                "Should find results for query: {}",
                query
            );
        }

        Ok(())
    }
}
