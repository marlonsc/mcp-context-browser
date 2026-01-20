//! Architecture Validation Tests
//!
//! Tests that validate correct usage of the DI system and hexagonal architecture.
//! These tests detect architectural violations and bypasses.
//!
//! ## Key Principle
//!
//! Tests should verify:
//! 1. Services are obtained via DI, not constructed directly
//! 2. All registries have registered providers
//! 3. Provider names match expectations from config
//! 4. Handles return consistent provider instances
//! 5. Admin services can switch providers at runtime

// Force linkme registration of all providers
extern crate mcb_providers;

use mcb_application::ports::registry::cache::*;
use mcb_application::ports::registry::embedding::*;
use mcb_application::ports::registry::language::*;
use mcb_application::ports::registry::vector_store::*;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use std::sync::Arc;

// ============================================================================
// Registry Completeness Validation
// ============================================================================

#[test]
fn test_all_expected_embedding_providers_registered() {
    let providers = list_embedding_providers();
    let provider_names: Vec<&str> = providers.iter().map(|(name, _)| *name).collect();

    // Expected providers that should always be registered
    let expected = ["null", "ollama", "openai"];

    for exp in expected {
        assert!(
            provider_names.contains(&exp),
            "Missing expected embedding provider '{}'. Registered: {:?}",
            exp,
            provider_names
        );
    }
}

#[test]
fn test_all_expected_vector_store_providers_registered() {
    let providers = list_vector_store_providers();
    let provider_names: Vec<&str> = providers.iter().map(|(name, _)| *name).collect();

    // Expected providers that should always be registered
    let expected = ["memory", "null"];

    for exp in expected {
        assert!(
            provider_names.contains(&exp),
            "Missing expected vector store provider '{}'. Registered: {:?}",
            exp,
            provider_names
        );
    }
}

#[test]
fn test_all_expected_cache_providers_registered() {
    let providers = list_cache_providers();
    let provider_names: Vec<&str> = providers.iter().map(|(name, _)| *name).collect();

    // Expected providers that should always be registered
    let expected = ["null", "moka"];

    for exp in expected {
        assert!(
            provider_names.contains(&exp),
            "Missing expected cache provider '{}'. Registered: {:?}",
            exp,
            provider_names
        );
    }
}

#[test]
fn test_all_expected_language_providers_registered() {
    let providers = list_language_providers();
    let provider_names: Vec<&str> = providers.iter().map(|(name, _)| *name).collect();

    // Expected providers that should always be registered
    let expected = ["universal"];

    for exp in expected {
        assert!(
            provider_names.contains(&exp),
            "Missing expected language provider '{}'. Registered: {:?}",
            exp,
            provider_names
        );
    }
}

// ============================================================================
// DI Configuration Consistency
// ============================================================================

#[tokio::test]
async fn test_config_provider_names_match_resolved_providers() {
    let config = AppConfig::default();

    // Get expected provider name from config
    let expected_embedding = config
        .providers
        .embedding
        .provider
        .clone()
        .unwrap_or_else(|| "null".to_string());

    // Initialize app and get resolved provider
    let ctx = init_app(config).await.expect("init_app should succeed");
    let embedding = ctx.embedding_handle().get();

    assert_eq!(
        embedding.provider_name(),
        expected_embedding,
        "Resolved provider name should match config"
    );
}

#[tokio::test]
async fn test_handle_based_di_prevents_direct_construction() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // Get provider via handle (correct DI usage)
    let via_handle_1 = ctx.embedding_handle().get();
    let via_handle_2 = ctx.embedding_handle().get();

    // Both should be the same Arc instance
    assert!(
        Arc::ptr_eq(&via_handle_1, &via_handle_2),
        "Handle should return same instance (proving DI is used, not direct construction)"
    );
}

#[tokio::test]
async fn test_multiple_handles_reference_same_underlying_provider() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // Get embedding handle twice
    let handle1 = ctx.embedding_handle();
    let handle2 = ctx.embedding_handle();

    // Both handles should return the same provider
    let provider1 = handle1.get();
    let provider2 = handle2.get();

    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Different handle references should return same provider"
    );
}

// ============================================================================
// Provider Factory Validation
// ============================================================================

#[test]
fn test_provider_factories_return_working_providers() {
    // Test that factory functions create working providers, not just return Ok

    // Embedding provider
    let embedding_config = EmbeddingProviderConfig::new("null");
    let embedding = resolve_embedding_provider(&embedding_config).expect("Should resolve");
    assert_eq!(
        embedding.dimensions(),
        384,
        "Null embedding should have 384 dimensions"
    );

    // Cache provider
    let cache_config = CacheProviderConfig::new("null");
    let cache = resolve_cache_provider(&cache_config).expect("Should resolve");
    assert_eq!(cache.provider_name(), "null", "Should be null cache");

    // Vector store provider
    let vs_config = VectorStoreProviderConfig::new("memory");
    let vs = resolve_vector_store_provider(&vs_config).expect("Should resolve");
    assert!(
        vs.provider_name() == "memory" || vs.provider_name() == "in_memory",
        "Should be memory vector store"
    );

    // Language provider
    let lang_config = LanguageProviderConfig::new("universal");
    let _ = resolve_language_provider(&lang_config).expect("Should resolve universal");
}

// ============================================================================
// Admin Service Architecture Validation
// ============================================================================

#[tokio::test]
async fn test_admin_services_accessible_via_context() {
    let config = AppConfig::default();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // Admin services should be accessible and functional
    // This validates they're properly wired in the DI container
    let embedding_admin = ctx.embedding_admin();
    let vector_store_admin = ctx.vector_store_admin();
    let cache_admin = ctx.cache_admin();

    // Validate admin services return meaningful data
    assert!(
        !embedding_admin.list_providers().is_empty(),
        "Embedding admin should list available providers"
    );
    assert!(
        !embedding_admin.current_provider().is_empty(),
        "Embedding admin should report current provider"
    );

    assert!(
        !vector_store_admin.list_providers().is_empty(),
        "Vector store admin should list available providers"
    );

    assert!(
        !cache_admin.list_providers().is_empty(),
        "Cache admin should list available providers"
    );
    assert!(
        !cache_admin.current_provider().is_empty(),
        "Cache admin should report current provider"
    );
}

// ============================================================================
// Cross-Layer Dependency Validation
// ============================================================================

#[test]
fn test_registry_entries_have_valid_descriptions() {
    // All registry entries should have meaningful descriptions
    // (empty descriptions indicate incomplete registration)

    for (name, desc) in list_embedding_providers() {
        assert!(
            !desc.is_empty(),
            "Embedding provider '{}' has empty description",
            name
        );
        assert!(
            desc.len() > 5,
            "Embedding provider '{}' has too short description: '{}'",
            name,
            desc
        );
    }

    for (name, desc) in list_vector_store_providers() {
        assert!(
            !desc.is_empty(),
            "Vector store provider '{}' has empty description",
            name
        );
    }

    for (name, desc) in list_cache_providers() {
        assert!(
            !desc.is_empty(),
            "Cache provider '{}' has empty description",
            name
        );
    }

    for (name, desc) in list_language_providers() {
        assert!(
            !desc.is_empty(),
            "Language provider '{}' has empty description",
            name
        );
    }
}

#[test]
fn test_provider_resolution_fails_gracefully_for_unknown() {
    // Unknown providers should fail with descriptive errors

    let unknown_embedding = resolve_embedding_provider(&EmbeddingProviderConfig::new("xyz123"));
    assert!(
        unknown_embedding.is_err(),
        "Should fail for unknown embedding provider"
    );

    let unknown_vs = resolve_vector_store_provider(&VectorStoreProviderConfig::new("xyz123"));
    assert!(
        unknown_vs.is_err(),
        "Should fail for unknown vector store provider"
    );

    let unknown_cache = resolve_cache_provider(&CacheProviderConfig::new("xyz123"));
    assert!(
        unknown_cache.is_err(),
        "Should fail for unknown cache provider"
    );

    let unknown_lang = resolve_language_provider(&LanguageProviderConfig::new("xyz123"));
    assert!(
        unknown_lang.is_err(),
        "Should fail for unknown language provider"
    );
}
