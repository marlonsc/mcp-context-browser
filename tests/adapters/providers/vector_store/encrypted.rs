//! Tests for encrypted vector store wrapper
//!
//! Tests for encryption at rest for any vector store provider.

use mcp_context_browser::adapters::providers::vector_store::encrypted::EncryptedVectorStoreProvider;
use mcp_context_browser::adapters::providers::vector_store::InMemoryVectorStoreProvider;
use mcp_context_browser::domain::ports::VectorStoreProvider;
use mcp_context_browser::infrastructure::crypto::EncryptionConfig;

#[tokio::test]
async fn test_encrypted_vector_store_creation(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let inner = InMemoryVectorStoreProvider::new();
    let config = EncryptionConfig::default();

    let encrypted_store = EncryptedVectorStoreProvider::new(inner, config).await?;
    assert_eq!(encrypted_store.provider_name(), "encrypted");
    Ok(())
}

#[tokio::test]
async fn test_encrypted_vector_store_disabled(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let inner = InMemoryVectorStoreProvider::new();
    let config = EncryptionConfig {
        enabled: false,
        ..Default::default()
    };

    let encrypted_store = EncryptedVectorStoreProvider::new(inner, config).await?;
    assert_eq!(encrypted_store.provider_name(), "encrypted");
    Ok(())
}

#[tokio::test]
async fn test_encrypted_search_returns_empty() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    // Test that encrypted search returns empty results (as implemented)
    let inner = InMemoryVectorStoreProvider::new();
    let config = EncryptionConfig::default();
    let encrypted_store = EncryptedVectorStoreProvider::new(inner, config).await?;

    // Create collection
    encrypted_store.create_collection("test", 128).await?;

    let results = encrypted_store
        .search_similar("test", &[1.0, 2.0, 3.0], 10, None)
        .await?;
    assert!(results.is_empty());
    Ok(())
}
