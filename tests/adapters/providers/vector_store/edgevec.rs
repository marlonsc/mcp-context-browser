//! Tests for EdgeVec vector store provider
//!
//! Tests for high-performance embedded vector database implementation using EdgeVec.

use mcp_context_browser::adapters::providers::vector_store::edgevec::{
    EdgeVecConfig, EdgeVecVectorStoreProvider, HnswConfig, MetricType,
};
use mcp_context_browser::domain::ports::VectorStoreProvider;
use mcp_context_browser::domain::types::Embedding;
use std::collections::HashMap;

#[tokio::test]
async fn test_edgevec_provider_creation() {
    let config = EdgeVecConfig::default();
    let provider = EdgeVecVectorStoreProvider::new(config);
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_edgevec_collection_management() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let config = EdgeVecConfig::default();
    let provider = EdgeVecVectorStoreProvider::new(config)?;

    provider.create_collection("test_collection", 1536).await?;
    assert!(provider.collection_exists("test_collection").await?);
    let stats = provider.get_stats("test_collection").await?;
    assert_eq!(
        stats.get("collection").ok_or("collection not found")?,
        "test_collection"
    );
    assert_eq!(
        stats.get("vector_count").ok_or("vector_count not found")?,
        &serde_json::json!(0)
    );
    Ok(())
}

#[tokio::test]
async fn test_edgevec_vector_operations() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = EdgeVecConfig {
        dimensions: 3,
        hnsw_config: HnswConfig::default(),
        metric: MetricType::Cosine,
        use_quantization: false,
        quantizer_config: Default::default(),
    };

    let provider = EdgeVecVectorStoreProvider::new(config)?;
    provider.create_collection("test", 3).await?;

    let vectors = vec![
        Embedding {
            vector: vec![1.0, 0.0, 0.0],
            dimensions: 3,
            model: "test".to_string(),
        },
        Embedding {
            vector: vec![0.0, 1.0, 0.0],
            dimensions: 3,
            model: "test".to_string(),
        },
        Embedding {
            vector: vec![0.0, 0.0, 1.0],
            dimensions: 3,
            model: "test".to_string(),
        },
    ];

    let metadata = vec![
        HashMap::from([
            ("file_path".to_string(), serde_json::json!("test.rs")),
            ("line_number".to_string(), serde_json::json!(1)),
            ("content".to_string(), serde_json::json!("vec1")),
        ]),
        HashMap::from([
            ("file_path".to_string(), serde_json::json!("test.rs")),
            ("line_number".to_string(), serde_json::json!(2)),
            ("content".to_string(), serde_json::json!("vec2")),
        ]),
        HashMap::from([
            ("file_path".to_string(), serde_json::json!("test.rs")),
            ("line_number".to_string(), serde_json::json!(3)),
            ("content".to_string(), serde_json::json!("vec3")),
        ]),
    ];

    let ids = provider.insert_vectors("test", &vectors, metadata).await?;
    assert_eq!(ids.len(), 3);

    let query = vec![1.0, 0.1, 0.1];
    let results = provider.search_similar("test", &query, 2, None).await?;
    assert_eq!(results.len(), 2);

    provider.delete_vectors("test", &ids[..1]).await?;
    let stats = provider.get_stats("test").await?;
    assert_eq!(
        stats.get("vector_count").ok_or("vector_count not found")?,
        &serde_json::json!(2)
    );
    Ok(())
}
