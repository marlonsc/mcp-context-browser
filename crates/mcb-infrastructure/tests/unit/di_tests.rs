//! Unit tests for DI resolver module
//!
//! Moved from inline tests in src/di/resolver.rs to comply with test organization standards.

use mcb_infrastructure::di::{AvailableProviders, list_available_providers};

#[test]
fn test_list_available_providers() {
    // Verifies the function is callable and returns valid data
    let providers = list_available_providers();

    // Verify the AvailableProviders struct is valid
    // Providers may be empty in unit tests since mcb-providers isn't linked
    // but the Display implementation should work on any state
    let display = format!("{providers}");
    assert!(
        display.contains("Embedding Providers"),
        "Display should include Embedding Providers section"
    );
}

#[test]
fn test_available_providers_display() {
    let providers = AvailableProviders {
        embedding: vec![("null", "Null provider")],
        vector_store: vec![("memory", "In-memory store")],
        cache: vec![("moka", "Moka cache")],
        language: vec![("universal", "Universal chunker")],
    };

    let display = format!("{providers}");
    assert!(display.contains("Embedding Providers"));
    assert!(display.contains("null"));
    assert!(display.contains("Vector Store Providers"));
    assert!(display.contains("memory"));
    assert!(display.contains("Cache Providers"));
    assert!(display.contains("moka"));
    assert!(display.contains("Language Providers"));
    assert!(display.contains("universal"));
}

#[test]
fn test_available_providers_empty() {
    let providers = AvailableProviders {
        embedding: vec![],
        vector_store: vec![],
        cache: vec![],
        language: vec![],
    };

    let display = format!("{providers}");
    // Even with empty providers, section headers should be present
    assert!(display.contains("Embedding Providers"));
}
