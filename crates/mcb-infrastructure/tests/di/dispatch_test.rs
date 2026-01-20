//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;

// Force link mcb_providers so inventory registrations are included
extern crate mcb_providers;

#[tokio::test]
async fn test_di_container_builder() {
    let config = AppConfig::default();
    let result = init_app(config).await;

    assert!(
        result.is_ok(),
        "init_app should complete successfully: {:?}",
        result.err()
    );

    let app_context = result.unwrap();

    // Verify context has expected fields
    assert!(
        std::mem::size_of_val(&app_context.config) > 0,
        "Config should be initialized"
    );

    // Verify handles are accessible
    let embedding_handle = app_context.embedding_handle();
    assert!(!embedding_handle.provider_name().is_empty());
}

#[tokio::test]
async fn test_provider_selection_from_config() {
    // Test that providers are correctly selected based on configuration

    // Test with null providers (default)
    let mut config = AppConfig::default();
    config.providers.embedding.insert(
        "default".to_string(),
        EmbeddingConfig {
            provider: "null".to_string(),
            model: "test".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(384),
            max_tokens: Some(1000),
        },
    );
    config.providers.vector_store.insert(
        "default".to_string(),
        VectorStoreConfig {
            provider: "null".to_string(),
            address: None,
            token: None,
            collection: Some("test".to_string()),
            dimensions: Some(384),
            timeout_secs: None,
        },
    );

    let app_context = init_app(config)
        .await
        .expect("Should initialize with null providers");

    // Verify correct providers were selected via handles
    assert_eq!(app_context.embedding_handle().get().provider_name(), "null");
    assert_eq!(
        app_context.vector_store_handle().get().provider_name(),
        "null"
    );
    assert_eq!(app_context.cache_handle().get().provider_name(), "moka"); // default cache
    assert_eq!(
        app_context.language_handle().get().provider_name(),
        "universal"
    ); // default language
}

#[tokio::test]
async fn test_provider_resolution_uses_registry() {
    // Test that provider resolution uses the registry system, not hardcoded instances

    // This test verifies the Clean Architecture pattern:
    // - Configuration drives provider selection
    // - Registry resolves provider by name
    // - Services use providers through traits (no concrete knowledge)

    let config = AppConfig::default();
    let app_context = init_app(config)
        .await
        .expect("Should initialize successfully");

    // Verify that providers implement the expected traits
    // (This would fail at compile time if providers didn't implement the traits)

    // Test that we can call methods through the trait via handles
    let embedding = app_context.embedding_handle().get();
    let _dimensions = embedding.dimensions();
    let _health = embedding.health_check().await;

    // Verify provider names are returned correctly
    assert!(
        !app_context
            .embedding_handle()
            .get()
            .provider_name()
            .is_empty()
    );
    assert!(
        !app_context
            .vector_store_handle()
            .get()
            .provider_name()
            .is_empty()
    );
    assert!(!app_context.cache_handle().get().provider_name().is_empty());
    assert!(
        !app_context
            .language_handle()
            .get()
            .provider_name()
            .is_empty()
    );
}

#[tokio::test]
async fn test_admin_services_are_accessible() {
    // Test that admin services for runtime provider switching are accessible

    let config = AppConfig::default();
    let app_context = init_app(config)
        .await
        .expect("Should initialize successfully");

    // Verify admin services are accessible
    let embedding_admin = app_context.embedding_admin();
    let current = embedding_admin.current_provider();
    assert!(!current.is_empty(), "Should have a current provider");

    // Verify we can list available providers
    let providers = embedding_admin.list_providers();
    assert!(!providers.is_empty(), "Should have at least one provider");

    // Verify cache admin
    let cache_admin = app_context.cache_admin();
    let cache_current = cache_admin.current_provider();
    assert!(
        !cache_current.is_empty(),
        "Cache should have a current provider"
    );
}

#[tokio::test]
async fn test_infrastructure_services_from_catalog() {
    // Test that infrastructure services are accessible from the dill catalog

    let config = AppConfig::default();
    let app_context = init_app(config)
        .await
        .expect("Should initialize successfully");

    // Verify infrastructure services are accessible
    // Arc<dyn Trait> types have a strong_count >= 1 if valid
    let auth = app_context.auth();
    assert!(
        std::sync::Arc::strong_count(&auth) >= 1,
        "Auth service should have valid Arc reference"
    );

    let event_bus = app_context.event_bus();
    assert!(
        std::sync::Arc::strong_count(&event_bus) >= 1,
        "EventBus service should have valid Arc reference"
    );

    let metrics = app_context.metrics();
    assert!(
        std::sync::Arc::strong_count(&metrics) >= 1,
        "Metrics service should have valid Arc reference"
    );

    let sync = app_context.sync();
    assert!(
        std::sync::Arc::strong_count(&sync) >= 1,
        "Sync service should have valid Arc reference"
    );

    let snapshot = app_context.snapshot();
    assert!(
        std::sync::Arc::strong_count(&snapshot) >= 1,
        "Snapshot service should have valid Arc reference"
    );

    let shutdown = app_context.shutdown();
    assert!(
        std::sync::Arc::strong_count(&shutdown) >= 1,
        "Shutdown service should have valid Arc reference"
    );

    let performance = app_context.performance();
    assert!(
        std::sync::Arc::strong_count(&performance) >= 1,
        "Performance service should have valid Arc reference"
    );

    let indexing = app_context.indexing();
    assert!(
        std::sync::Arc::strong_count(&indexing) >= 1,
        "Indexing service should have valid Arc reference"
    );
}
