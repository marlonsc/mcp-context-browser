//! Error Recovery Tests
//!
//! Tests that validate graceful error handling and recovery scenarios.
//!
//! ## Key Principle
//!
//! These tests verify:
//! 1. Errors are descriptive and actionable
//! 2. System handles edge cases gracefully
//! 3. Invalid configurations fail fast with clear messages
//! 4. Partial failures don't corrupt state

// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_application::ports::registry::cache::*;
use mcb_application::ports::registry::embedding::*;
use mcb_application::ports::registry::language::*;
use mcb_application::ports::registry::vector_store::*;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;

// ============================================================================
// Provider Resolution Error Handling
// ============================================================================

#[test]
fn test_unknown_embedding_provider_error_message() {
    let config = EmbeddingProviderConfig::new("nonexistent_xyz_provider");
    let result = resolve_embedding_provider(&config);

    assert!(result.is_err(), "Should fail for unknown provider");

    // Use match to avoid unwrap_err requiring Debug on Ok type
    match result {
        Err(err) => {
            assert!(
                err.contains("Unknown") || err.contains("not found") || err.contains("nonexistent"),
                "Error should mention the issue. Got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for unknown provider"),
    }
}

#[test]
fn test_unknown_vector_store_provider_error_message() {
    let config = VectorStoreProviderConfig::new("nonexistent_xyz_store");
    let result = resolve_vector_store_provider(&config);

    assert!(result.is_err(), "Should fail for unknown provider");

    match result {
        Err(err) => {
            assert!(
                err.contains("Unknown") || err.contains("not found") || err.contains("nonexistent"),
                "Error should mention the issue. Got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for unknown provider"),
    }
}

#[test]
fn test_unknown_cache_provider_error_message() {
    let config = CacheProviderConfig::new("nonexistent_xyz_cache");
    let result = resolve_cache_provider(&config);

    assert!(result.is_err(), "Should fail for unknown provider");

    match result {
        Err(err) => {
            assert!(
                err.contains("Unknown") || err.contains("not found") || err.contains("nonexistent"),
                "Error should mention the issue. Got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for unknown provider"),
    }
}

#[test]
fn test_unknown_language_provider_error_message() {
    let config = LanguageProviderConfig::new("nonexistent_xyz_lang");
    let result = resolve_language_provider(&config);

    assert!(result.is_err(), "Should fail for unknown provider");

    match result {
        Err(err) => {
            assert!(
                err.contains("Unknown") || err.contains("not found") || err.contains("nonexistent"),
                "Error should mention the issue. Got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for unknown provider"),
    }
}

// ============================================================================
// Search on Empty/Missing Collections
// ============================================================================

#[tokio::test]
async fn test_search_empty_collection_returns_empty_not_error() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let collection = "error_test_empty_collection";

    // Create empty collection
    vector_store
        .create_collection(collection, 384)
        .await
        .expect("Create collection");

    // Search in empty collection
    let query_embedding = embedding
        .embed_batch(&["test query".to_string()])
        .await
        .expect("Embed");

    let results = vector_store
        .search_similar(collection, &query_embedding[0].vector, 10, None)
        .await
        .expect("Search should not error on empty collection");

    assert!(
        results.is_empty(),
        "Empty collection should return empty results"
    );
}

// ============================================================================
// Configuration Validation
// ============================================================================

#[tokio::test]
async fn test_init_app_with_default_config_succeeds() {
    // Default config should always work (uses null/memory providers)
    let config = AppConfig::default();
    let result = init_app(config).await;

    assert!(
        result.is_ok(),
        "init_app with default config should succeed: {}",
        result.err().map(|e| e.to_string()).unwrap_or_default()
    );
}

#[tokio::test]
async fn test_provider_handles_return_valid_instances() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // All handles should return valid providers
    let embedding = ctx.embedding_handle().get();
    assert!(
        embedding.dimensions() > 0,
        "Embedding should have positive dimensions"
    );

    let vector_store = ctx.vector_store_handle().get();
    assert!(
        !vector_store.provider_name().is_empty(),
        "Vector store should have a name"
    );

    let cache = ctx.cache_handle().get();
    assert!(
        !cache.provider_name().is_empty(),
        "Cache should have a name"
    );
}

// ============================================================================
// Multiple Operation Error Isolation
// ============================================================================

#[tokio::test]
async fn test_failed_search_doesnt_corrupt_state() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let collection = "error_isolation_test";

    // Create and populate collection
    vector_store
        .create_collection(collection, 384)
        .await
        .expect("Create collection");

    let embeddings = embedding
        .embed_batch(&["test data".to_string()])
        .await
        .expect("Embed");

    let metadata = vec![{
        let mut m = std::collections::HashMap::new();
        m.insert("content".to_string(), serde_json::json!("test"));
        m
    }];

    vector_store
        .insert_vectors(collection, &embeddings, metadata)
        .await
        .expect("Insert");

    // Try search with wrong dimensions (should fail or handle gracefully)
    let wrong_dim_vector = vec![0.1f32; 100]; // Wrong dimensions

    // This might fail, but shouldn't corrupt the collection
    let _ = vector_store
        .search_similar(collection, &wrong_dim_vector, 10, None)
        .await;

    // Original search should still work
    let correct_query = embedding
        .embed_batch(&["test".to_string()])
        .await
        .expect("Embed");

    let results = vector_store
        .search_similar(collection, &correct_query[0].vector, 10, None)
        .await
        .expect("Search should still work after failed attempt");

    assert!(!results.is_empty(), "Collection should not be corrupted");
}

// ============================================================================
// Registry Robustness
// ============================================================================

#[test]
fn test_list_providers_never_panics() {
    // These should never panic, even if registry is empty
    let embedding_providers = list_embedding_providers();
    let vector_store_providers = list_vector_store_providers();
    let cache_providers = list_cache_providers();
    let language_providers = list_language_providers();

    // With extern crate mcb_providers, none should be empty
    assert!(
        !embedding_providers.is_empty(),
        "Should have embedding providers"
    );
    assert!(
        !vector_store_providers.is_empty(),
        "Should have vector store providers"
    );
    assert!(!cache_providers.is_empty(), "Should have cache providers");
    assert!(
        !language_providers.is_empty(),
        "Should have language providers"
    );
}

#[test]
fn test_resolve_with_empty_config_values() {
    // Config with empty strings should fail gracefully
    let embedding_config = EmbeddingProviderConfig::new("");
    let result = resolve_embedding_provider(&embedding_config);

    assert!(result.is_err(), "Empty provider name should fail");
}

// ============================================================================
// Concurrent Access Safety
// ============================================================================

#[tokio::test]
async fn test_concurrent_handle_access() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let handle = ctx.embedding_handle();

    // Spawn multiple tasks accessing the handle
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let h = handle.clone();
        tasks.push(tokio::spawn(async move {
            let provider = h.get();
            provider.dimensions()
        }));
    }

    // All should succeed
    for task in tasks {
        let dims = task.await.expect("Task should not panic");
        assert_eq!(dims, 384, "All accesses should return same dimensions");
    }
}
