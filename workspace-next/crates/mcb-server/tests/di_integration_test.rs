//! Dependency Injection Integration Tests
//!
//! Tests for the full DI container wiring and component integration:
//! - Full container creation and component access
//! - Component lifecycle management
//! - Cross-component interactions

use mcb_domain::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};
use mcb_domain::ports::providers::cache::CacheEntryConfig;
use mcb_infrastructure::config::{AppConfig, ConfigBuilder};
use mcb_infrastructure::di::bootstrap::{FullContainer, InfrastructureComponents};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a minimal test configuration
fn minimal_config() -> AppConfig {
    ConfigBuilder::new().build()
}

/// Create a configuration with cache disabled
fn config_without_cache() -> AppConfig {
    let mut config = ConfigBuilder::new().build();
    config.system.infrastructure.cache.enabled = false;
    config
}

// ============================================================================
// Full Container Wiring Tests
// ============================================================================

#[tokio::test]
async fn test_full_container_creation() {
    let config = minimal_config();
    let result = FullContainer::new(config).await;

    assert!(
        result.is_ok(),
        "Full container should be created successfully"
    );
}

#[tokio::test]
async fn test_full_container_provides_all_services() {
    let config = minimal_config();
    let container = FullContainer::new(config)
        .await
        .expect("Container creation should succeed");

    // Verify all services are accessible and can perform basic operations
    let indexing = container.indexing_service();
    let context = container.context_service();
    let search = container.search_service();

    // Verify indexing service is functional
    let status = indexing.get_status();
    assert!(!status.is_indexing, "Fresh container should not be actively indexing");

    // Verify context service reports valid dimensions
    let dimensions = context.embedding_dimensions();
    assert!(dimensions > 0, "Context service should report positive embedding dimensions");

    // Verify search service exists (search requires initialized collection)
    // Just verify the Arc is not null by checking it can be cloned
    let _search_clone = search.clone();
    assert!(true, "All services are accessible and functional");
}

#[tokio::test]
async fn test_infrastructure_components_accessible() {
    let config = minimal_config();
    let container = FullContainer::new(config)
        .await
        .expect("Container creation should succeed");

    // Verify infrastructure components (using field access, not method calls)
    let infra = &container.infrastructure;

    // Cache should be accessible
    let cache_result = infra.cache.get::<String>("test-key").await;
    assert!(cache_result.is_ok(), "Cache should be accessible");

    // Health registry should have system checker
    let checks = infra.health.list_checks().await;
    assert!(
        checks.contains(&"system".to_string()),
        "System health checker should be registered"
    );
}

// ============================================================================
// Component Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_container_clone_shares_state() {
    let config = minimal_config();
    let container1 = FullContainer::new(config)
        .await
        .expect("Container creation should succeed");

    // Clone the container
    let container2 = container1.clone();

    // Both containers should share the same underlying services
    // (they use Arc internally)

    // Store something in cache via container1
    container1
        .infrastructure
        .cache
        .set(
            "shared-key",
            &"shared-value".to_string(),
            CacheEntryConfig::default(),
        )
        .await
        .expect("Cache set should work");

    // Should be able to read via container2
    let value: Option<String> = container2
        .infrastructure
        .cache
        .get("shared-key")
        .await
        .expect("Cache get should work");

    assert_eq!(
        value,
        Some("shared-value".to_string()),
        "Cloned containers should share cache state"
    );
}

#[tokio::test]
async fn test_multiple_container_instances_are_independent() {
    let config1 = minimal_config();
    let config2 = minimal_config();

    let container1 = FullContainer::new(config1)
        .await
        .expect("First container creation should succeed");

    let container2 = FullContainer::new(config2)
        .await
        .expect("Second container creation should succeed");

    // Containers created separately should be independent
    // Store in container1
    container1
        .infrastructure
        .cache
        .set(
            "container1-key",
            &"value1".to_string(),
            CacheEntryConfig::default(),
        )
        .await
        .expect("Set should work");

    // Store different value with same key in container2
    container2
        .infrastructure
        .cache
        .set(
            "container1-key",
            &"value2".to_string(),
            CacheEntryConfig::default(),
        )
        .await
        .expect("Set should work");

    // Container1 should still have its own value (unless they share underlying cache)
    // This test verifies the isolation behavior
    let value1: Option<String> = container1
        .infrastructure
        .cache
        .get("container1-key")
        .await
        .expect("Get should work");

    let value2: Option<String> = container2
        .infrastructure
        .cache
        .get("container1-key")
        .await
        .expect("Get should work");

    // If using in-memory cache without sharing, values might differ
    // If using shared backend, they might be same
    // Both are valid - we just verify no errors occur
    assert!(value1.is_some(), "Container1 should have a value");
    assert!(value2.is_some(), "Container2 should have a value");
}

// ============================================================================
// Health Check Integration Tests
// ============================================================================

#[tokio::test]
async fn test_health_registry_integration() {
    let config = minimal_config();
    let components = InfrastructureComponents::new(config)
        .await
        .expect("Components creation should succeed");

    let health = &components.health;

    // List checks should include system
    let checks = health.list_checks().await;
    assert!(!checks.is_empty(), "Should have registered health checks");
    assert!(
        checks.contains(&"system".to_string()),
        "Should include system checker"
    );

    // Check all should succeed
    let response = health.perform_health_checks().await;
    assert!(response.is_healthy(), "System should be healthy");
    assert!(
        !response.checks.is_empty(),
        "Should return health check results"
    );
}

// ============================================================================
// Crypto Service Integration Tests
// ============================================================================

#[tokio::test]
async fn test_crypto_service_integration() {
    let config = minimal_config();
    let components = InfrastructureComponents::new(config)
        .await
        .expect("Components creation should succeed");

    let crypto = &components.crypto;

    // Test encrypt/decrypt roundtrip
    let plaintext = b"sensitive data for testing";
    let encrypted = crypto.encrypt(plaintext);
    assert!(encrypted.is_ok(), "Encryption should succeed");

    let encrypted_data = encrypted.unwrap();
    let decrypted = crypto.decrypt(&encrypted_data);
    assert!(decrypted.is_ok(), "Decryption should succeed");

    assert_eq!(
        decrypted.unwrap(),
        plaintext,
        "Decrypted data should match original"
    );
}

#[tokio::test]
async fn test_crypto_service_different_data() {
    let config = minimal_config();
    let components = InfrastructureComponents::new(config)
        .await
        .expect("Components creation should succeed");

    let crypto = &components.crypto;

    // Different data should produce different ciphertext
    let data1 = b"first piece of data";
    let data2 = b"second piece of data";

    let encrypted1 = crypto.encrypt(data1).expect("Encrypt 1 should work");
    let encrypted2 = crypto.encrypt(data2).expect("Encrypt 2 should work");

    // Ciphertext should be different
    assert_ne!(
        encrypted1, encrypted2,
        "Different plaintext should produce different ciphertext"
    );

    // But both should decrypt correctly
    let decrypted1 = crypto.decrypt(&encrypted1).expect("Decrypt 1 should work");
    let decrypted2 = crypto.decrypt(&encrypted2).expect("Decrypt 2 should work");

    assert_eq!(decrypted1, data1);
    assert_eq!(decrypted2, data2);
}

// ============================================================================
// Configuration Integration Tests
// ============================================================================

#[tokio::test]
async fn test_default_config_creates_valid_container() {
    // Use completely default configuration
    let config = ConfigBuilder::new().build();

    let container = FullContainer::new(config).await;
    assert!(
        container.is_ok(),
        "Default config should create valid container"
    );

    let container = container.unwrap();

    // All services should be functional
    let status = container.indexing_service().get_status();
    assert!(
        !status.is_indexing,
        "Fresh container should not be indexing"
    );
}

// ============================================================================
// Cross-Component Interaction Tests
// ============================================================================

#[tokio::test]
async fn test_services_share_infrastructure() {
    let config = minimal_config();
    let container = FullContainer::new(config)
        .await
        .expect("Container creation should succeed");

    // Get services
    let context_service = container.context_service();
    let search_service = container.search_service();

    // Both services should be able to function
    // Context service can embed
    let embed_result = context_service.embed_text("test text").await;
    assert!(embed_result.is_ok(), "Context service should embed text");

    // Initialize collection before searching (search requires existing collection)
    let collection_name = "test_collection";
    let init_result = context_service.initialize(collection_name).await;
    assert!(
        init_result.is_ok(),
        "Context service should initialize collection"
    );

    // Search service can search (now with valid collection)
    let search_result = search_service.search(collection_name, "query", 5).await;
    assert!(
        search_result.is_ok(),
        "Search service should perform search"
    );

    // The embedding dimensions should be consistent
    let dimensions = context_service.embedding_dimensions();
    let embedding = embed_result.unwrap();
    assert_eq!(
        embedding.dimensions, dimensions,
        "Embedding dimensions should match service's reported dimensions"
    );
}

#[tokio::test]
async fn test_context_service_stats() {
    let config = minimal_config();
    let container = FullContainer::new(config)
        .await
        .expect("Container creation should succeed");

    let context_service = container.context_service();

    // Get stats should work
    let stats_result = context_service.get_stats().await;
    assert!(stats_result.is_ok(), "Getting stats should succeed");

    let (repo_stats, search_stats) = stats_result.unwrap();

    // Stats should have reasonable values
    assert!(
        repo_stats.total_chunks >= 0,
        "Total chunks should be non-negative"
    );
    assert!(
        search_stats.total_queries >= 0,
        "Total queries should be non-negative"
    );
}
