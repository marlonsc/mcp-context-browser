//! Tests for VectorStoreProvider port trait
//!
//! Validates the contract and default implementations of the vector store provider port.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::providers::VectorStoreProvider;
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock implementation of VectorStoreProvider for testing
struct MockVectorStoreProvider {
    collections: std::sync::Mutex<HashMap<String, Vec<Embedding>>>,
    should_fail: bool,
}

impl MockVectorStoreProvider {
    fn new() -> Self {
        Self {
            collections: std::sync::Mutex::new(HashMap::new()),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            collections: std::sync::Mutex::new(HashMap::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl VectorStoreProvider for MockVectorStoreProvider {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let mut collections = self.collections.lock().expect("Lock poisoned");
        collections.insert(name.to_string(), Vec::new());
        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let mut collections = self.collections.lock().expect("Lock poisoned");
        collections.remove(name);
        Ok(())
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let collections = self.collections.lock().expect("Lock poisoned");
        Ok(collections.contains_key(name))
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        _metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let mut collections = self.collections.lock().expect("Lock poisoned");
        if let Some(col) = collections.get_mut(collection) {
            let ids: Vec<String> = vectors
                .iter()
                .enumerate()
                .map(|(i, _)| format!("vec_{}", i))
                .collect();
            col.extend(vectors.iter().cloned());
            return Ok(ids);
        }
        Err(mcb_domain::error::Error::internal("Collection not found"))
    }

    async fn search_similar(
        &self,
        _collection: &str,
        _query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        // Return mock results
        let results: Vec<SearchResult> = (0..limit.min(3))
            .map(|i| SearchResult {
                id: format!("result_{}", i),
                file_path: format!("src/file_{}.rs", i),
                start_line: (i * 10 + 1) as u32,
                content: format!("Mock content {}", i),
                score: 0.9 - (i as f64 * 0.1),
                language: "rust".to_string(),
            })
            .collect();
        Ok(results)
    }

    async fn delete_vectors(&self, _collection: &str, _ids: &[String]) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        _collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let results: Vec<SearchResult> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| SearchResult {
                id: id.clone(),
                file_path: format!("src/file_{}.rs", i),
                start_line: 1,
                content: "Mock content".to_string(),
                score: 1.0,
                language: "rust".to_string(),
            })
            .collect();
        Ok(results)
    }

    async fn list_vectors(&self, _collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let results: Vec<SearchResult> = (0..limit.min(5))
            .map(|i| SearchResult {
                id: format!("vec_{}", i),
                file_path: format!("src/file_{}.rs", i),
                start_line: 1,
                content: "Mock content".to_string(),
                score: 1.0,
                language: "rust".to_string(),
            })
            .collect();
        Ok(results)
    }

    async fn get_stats(&self, _collection: &str) -> Result<HashMap<String, Value>> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        let mut stats = HashMap::new();
        stats.insert("total_vectors".to_string(), Value::from(100));
        stats.insert("dimensions".to_string(), Value::from(384));
        Ok(stats)
    }

    async fn flush(&self, _collection: &str) -> Result<()> {
        if self.should_fail {
            return Err(mcb_domain::error::Error::internal("Simulated failure"));
        }
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

#[test]
fn test_vector_store_provider_trait_object() {
    // Verify that VectorStoreProvider can be used as a trait object
    let provider: Arc<dyn VectorStoreProvider> = Arc::new(MockVectorStoreProvider::new());

    assert_eq!(provider.provider_name(), "mock");
}

#[tokio::test]
async fn test_vector_store_full_lifecycle() {
    let provider = MockVectorStoreProvider::new();

    // Create collection
    let result = provider.create_collection("test_collection", 384).await;
    assert!(result.is_ok());

    // Check collection exists
    let exists = provider.collection_exists("test_collection").await;
    assert!(exists.is_ok());
    assert!(exists.expect("Expected result"));

    // Insert vectors
    let embeddings = vec![Embedding {
        vector: vec![0.1; 384],
        model: "test".to_string(),
        dimensions: 384,
    }];
    let metadata = vec![HashMap::new()];
    let ids = provider
        .insert_vectors("test_collection", &embeddings, metadata)
        .await;
    assert!(ids.is_ok());
    assert_eq!(ids.expect("Expected ids").len(), 1);

    // Search similar
    let query_vector = vec![0.1; 384];
    let results = provider
        .search_similar("test_collection", &query_vector, 5, None)
        .await;
    assert!(results.is_ok());

    // Delete collection
    let result = provider.delete_collection("test_collection").await;
    assert!(result.is_ok());

    // Verify collection no longer exists
    let exists = provider.collection_exists("test_collection").await;
    assert!(exists.is_ok());
    assert!(!exists.expect("Expected result"));
}

#[tokio::test]
async fn test_vector_store_health_check_default() {
    // Test the default health_check() implementation
    let provider = MockVectorStoreProvider::new();

    let result = provider.health_check().await;

    // Default health check calls collection_exists("__health_check__")
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_vector_store_health_check_failure() {
    let provider = MockVectorStoreProvider::failing();

    let result = provider.health_check().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_vector_store_search_with_limit() {
    let provider = MockVectorStoreProvider::new();

    let query_vector = vec![0.1; 384];

    // Test with small limit
    let results = provider
        .search_similar("test", &query_vector, 1, None)
        .await;
    assert!(results.is_ok());
    assert!(results.expect("Expected results").len() <= 1);

    // Test with larger limit
    let results = provider
        .search_similar("test", &query_vector, 10, None)
        .await;
    assert!(results.is_ok());
}

#[tokio::test]
async fn test_vector_store_get_stats() {
    let provider = MockVectorStoreProvider::new();

    let stats = provider.get_stats("test_collection").await;

    assert!(stats.is_ok());
    let stats = stats.expect("Expected stats");
    assert!(stats.contains_key("total_vectors"));
    assert!(stats.contains_key("dimensions"));
}
