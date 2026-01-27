//! Catalog DI Integration Tests
//!
//! Tests for the dill Catalog build and service resolution.
//! These tests verify that the IoC container initializes correctly
//! with all required providers and services.

use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::catalog::build_catalog;

// Force linkme registration by linking mcb_providers crate
extern crate mcb_providers;

/// Test that catalog builds successfully with default config
#[tokio::test]
async fn test_catalog_builds_with_default_config() {
    let config = AppConfig::default();
    let result = build_catalog(config).await;

    assert!(result.is_ok(), "Catalog build failed: {:?}", result.err());
}

/// Test that catalog builds with custom embedding provider config
#[tokio::test]
async fn test_catalog_builds_with_custom_embedding_config() {
    let mut config = AppConfig::default();
    config.providers.embedding.provider = Some("null".to_string());

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom embedding config failed: {:?}",
        result.err()
    );
}

/// Test that catalog builds with custom vector store config
#[tokio::test]
async fn test_catalog_builds_with_custom_vector_store_config() {
    let mut config = AppConfig::default();
    config.providers.vector_store.provider = Some("null".to_string());

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom vector store config failed: {:?}",
        result.err()
    );
}

/// Test that catalog builds with custom cache config
#[tokio::test]
async fn test_catalog_builds_with_custom_cache_config() {
    use mcb_infrastructure::config::types::CacheProvider;

    let mut config = AppConfig::default();
    config.system.infrastructure.cache.provider = CacheProvider::Moka;

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom cache config failed: {:?}",
        result.err()
    );
}
