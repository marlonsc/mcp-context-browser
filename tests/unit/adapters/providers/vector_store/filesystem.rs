//! Tests for filesystem-optimized vector store implementation
//!
//! Tests for high-performance vector storage using memory-mapped files.

use mcp_context_browser::adapters::providers::vector_store::filesystem::{
    FilesystemVectorStore, FilesystemVectorStoreConfig,
};
use mcp_context_browser::domain::ports::VectorStoreProvider;
use mcp_context_browser::domain::types::Embedding;
use std::collections::HashMap;
use tempfile::TempDir;

#[tokio::test]
async fn test_filesystem_vector_store() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemVectorStoreConfig {
        base_path: temp_dir.path().to_path_buf(),
        dimensions: 3,
        ..Default::default()
    };

    let store = FilesystemVectorStore::new(config).await?;
    store.create_collection("test", 3).await?;

    // Insert vectors
    let vectors = vec![Embedding {
        vector: vec![1.0, 2.0, 3.0],
        model: "test".to_string(),
        dimensions: 3,
    }];

    let metadata = vec![{
        let mut meta = HashMap::new();
        meta.insert("file_path".to_string(), serde_json::json!("test.rs"));
        meta.insert("start_line".to_string(), serde_json::json!(42));
        meta.insert("content".to_string(), serde_json::json!("test content"));
        meta
    }];

    let ids = store.insert_vectors("test", &vectors, metadata).await?;
    assert_eq!(ids.len(), 1);

    // Search similar
    let query = vec![1.0, 0.0, 0.0];
    let results = store.search_similar("test", &query, 5, None).await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].file_path, "test.rs");

    // Check stats
    let stats = store.get_stats("test").await?;
    let total_vectors = stats
        .get("total_vectors")
        .and_then(|v| v.as_u64())
        .ok_or("total_vectors not found")?;
    assert_eq!(total_vectors, 1);

    // Delete vectors
    store.delete_vectors("test", &ids).await?;

    // Check stats after deletion
    let stats = store.get_stats("test").await?;
    let total_vectors = stats
        .get("total_vectors")
        .and_then(|v| v.as_u64())
        .ok_or("total_vectors not found")?;
    assert_eq!(total_vectors, 0);
    Ok(())
}

#[tokio::test]
async fn test_collection_management() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let config = FilesystemVectorStoreConfig {
        base_path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let store = FilesystemVectorStore::new(config).await?;

    // Create collection
    store.create_collection("test1", 3).await?;
    assert!(store.collection_exists("test1").await?);

    // Delete collection
    store.delete_collection("test1").await?;
    assert!(!store.collection_exists("test1").await?);
    Ok(())
}
