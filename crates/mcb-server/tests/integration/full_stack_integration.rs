//! Full-Stack DI Integration Tests
//!
//! Tests end-to-end data flow through the hexagonal architecture:
//! AppContext → Provider Handles → Real Providers → Actual Data Operations
//!
//! ## Key Principle
//!
//! These tests validate that:
//! 1. DI container correctly wires all dependencies
//! 2. Provider handles return working providers
//! 3. Data actually flows through the architecture (not mocked)
//! 4. No architectural bypass occurs
//!
//! Uses real providers (Null/InMemory) for deterministic testing.

// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_domain::entities::CodeChunk;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use serde_json::json;
use std::sync::Arc;

/// Create test code chunks for full-stack testing
fn create_test_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "full_stack_chunk_1".to_string(),
            file_path: "src/config.rs".to_string(),
            content: r#"pub struct AppConfig {
    pub host: String,
    pub port: u16,
}"#
            .to_string(),
            start_line: 1,
            end_line: 4,
            language: "rust".to_string(),
            metadata: json!({"type": "struct", "name": "AppConfig"}),
        },
        CodeChunk {
            id: "full_stack_chunk_2".to_string(),
            file_path: "src/main.rs".to_string(),
            content: r#"#[tokio::main]
async fn main() {
    let config = AppConfig::default();
    run_server(&config).await;
}"#
            .to_string(),
            start_line: 1,
            end_line: 5,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "main"}),
        },
    ]
}

// ============================================================================
// Full-Stack Flow Tests
// ============================================================================

#[tokio::test]
async fn test_init_app_creates_working_context() {
    // Initialize app through DI
    let config = AppConfig::default();
    let result = init_app(config).await;

    assert!(
        result.is_ok(),
        "init_app should succeed: {}",
        result
            .as_ref()
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    );

    let ctx = result.expect("Context should be valid");

    // Verify embedding handle returns a real provider
    let embedding = ctx.embedding_handle().get();
    assert_eq!(
        embedding.provider_name(),
        "null",
        "Default should be null provider"
    );
    assert_eq!(
        embedding.dimensions(),
        384,
        "Null provider has 384 dimensions"
    );

    // Verify vector store handle returns a real provider
    let vector_store = ctx.vector_store_handle().get();
    assert!(
        vector_store.provider_name() == "in_memory" || vector_store.provider_name() == "memory",
        "Default should be in-memory vector store"
    );
}

#[tokio::test]
async fn test_embedding_generates_real_vectors() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();

    // Generate embeddings for test texts
    let texts = vec![
        "authentication middleware".to_string(),
        "database connection pool".to_string(),
    ];

    let embeddings = embedding
        .embed_batch(&texts)
        .await
        .expect("Embedding should work");

    // Validate real embedding generation
    assert_eq!(
        embeddings.len(),
        2,
        "Should generate embedding for each text"
    );

    for (i, emb) in embeddings.iter().enumerate() {
        assert_eq!(
            emb.dimensions, 384,
            "Embedding {} should have 384 dimensions",
            i
        );
        assert_eq!(
            emb.vector.len(),
            384,
            "Embedding {} vector should have 384 elements",
            i
        );
        assert_eq!(
            emb.model, "null-test",
            "Embedding {} should be from null-test model",
            i
        );
    }
}

#[tokio::test]
async fn test_full_index_and_search_flow() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let collection = "full_stack_test_collection";
    let chunks = create_test_chunks();

    // Step 1: Create collection
    vector_store
        .create_collection(collection, 384)
        .await
        .expect("Collection creation should succeed");

    // Step 2: Generate embeddings for chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding
        .embed_batch(&texts)
        .await
        .expect("Embedding should work");

    // Step 3: Build metadata from chunks
    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|chunk| {
            let mut meta = std::collections::HashMap::new();
            meta.insert("id".to_string(), json!(chunk.id));
            meta.insert("file_path".to_string(), json!(chunk.file_path));
            meta.insert("content".to_string(), json!(chunk.content));
            meta.insert("start_line".to_string(), json!(chunk.start_line));
            meta.insert("end_line".to_string(), json!(chunk.end_line));
            meta.insert("language".to_string(), json!(chunk.language));
            meta
        })
        .collect();

    // Step 4: Insert into vector store
    let ids = vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert should succeed");

    assert_eq!(ids.len(), chunks.len(), "Should insert all chunks");

    // Step 5: Search with a query
    let query_text = "application configuration settings".to_string();
    let query_embeddings = embedding
        .embed_batch(&[query_text])
        .await
        .expect("Query embedding should work");
    let query_vector = &query_embeddings[0].vector;

    let results = vector_store
        .search_similar(collection, query_vector, 5, None)
        .await
        .expect("Search should succeed");

    // Validate: we should find results (with deterministic NullEmbeddingProvider)
    assert!(
        !results.is_empty(),
        "Search should return results after indexing real data"
    );

    // Validate result structure
    for result in &results {
        assert!(!result.file_path.is_empty(), "Result should have file path");
        assert!(!result.content.is_empty(), "Result should have content");
        assert!(
            result.score >= 0.0 && result.score <= 1.0,
            "Score should be normalized"
        );
    }
}

#[tokio::test]
async fn test_provider_handles_return_same_instance() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // Get embedding provider twice via handle
    let handle = ctx.embedding_handle();
    let provider1 = handle.get();
    let provider2 = handle.get();

    // Should be the same Arc (same underlying provider)
    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Handle should return same provider instance"
    );
}

#[tokio::test]
async fn test_multiple_collections_isolated() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    // Create two collections
    let collection_a = "isolation_test_a";
    let collection_b = "isolation_test_b";

    vector_store
        .create_collection(collection_a, 384)
        .await
        .expect("Create collection A");
    vector_store
        .create_collection(collection_b, 384)
        .await
        .expect("Create collection B");

    // Insert data only into collection A
    let chunks = create_test_chunks();
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await.expect("Embed");

    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = std::collections::HashMap::new();
            m.insert("content".to_string(), json!(c.content));
            m.insert("file_path".to_string(), json!(c.file_path));
            m
        })
        .collect();

    vector_store
        .insert_vectors(collection_a, &embeddings, metadata)
        .await
        .expect("Insert into A");

    // Search in both collections
    let query_emb = embedding
        .embed_batch(&["config".to_string()])
        .await
        .expect("Query embed");

    let results_a = vector_store
        .search_similar(collection_a, &query_emb[0].vector, 10, None)
        .await
        .expect("Search A");

    let results_b = vector_store
        .search_similar(collection_b, &query_emb[0].vector, 10, None)
        .await
        .expect("Search B");

    // Collection A should have results, B should be empty
    assert!(!results_a.is_empty(), "Collection A should have data");
    assert!(
        results_b.is_empty(),
        "Collection B should be empty (isolated)"
    );
}

#[tokio::test]
async fn test_embedding_dimensions_consistent() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();

    // Generate multiple batches
    let batch1 = embedding
        .embed_batch(&["first text".to_string()])
        .await
        .expect("Batch 1");
    let batch2 = embedding
        .embed_batch(&["second text".to_string(), "third text".to_string()])
        .await
        .expect("Batch 2");

    // All should have consistent dimensions
    let expected_dim = embedding.dimensions();

    assert_eq!(batch1[0].dimensions, expected_dim);
    assert_eq!(batch1[0].vector.len(), expected_dim);
    assert_eq!(batch2[0].dimensions, expected_dim);
    assert_eq!(batch2[0].vector.len(), expected_dim);
    assert_eq!(batch2[1].dimensions, expected_dim);
    assert_eq!(batch2[1].vector.len(), expected_dim);
}
