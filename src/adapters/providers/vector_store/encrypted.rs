//! Encrypted vector store wrapper
//!
//! Provides encryption at rest for any vector store provider.
//! Wraps existing providers with AES-256-GCM encryption.

use crate::domain::error::Result;
use crate::domain::ports::VectorStoreProvider;
use crate::domain::types::{Embedding, SearchResult};
use crate::infrastructure::crypto::{CryptoService, EncryptedEnvelope, EncryptionConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Encrypted vector store provider
pub struct EncryptedVectorStoreProvider<P: VectorStoreProvider> {
    /// Underlying vector store provider
    inner: P,
    /// Cryptography service
    crypto: Arc<CryptoService>,
}

impl<P: VectorStoreProvider> EncryptedVectorStoreProvider<P> {
    /// Create a new encrypted vector store provider
    pub async fn new(inner: P, crypto_config: EncryptionConfig) -> Result<Self> {
        let crypto = Arc::new(CryptoService::new(crypto_config).await?);
        Ok(Self { inner, crypto })
    }

    /// Create with existing crypto service
    pub fn with_crypto_service(inner: P, crypto: Arc<CryptoService>) -> Self {
        Self { inner, crypto }
    }
}

#[async_trait]
impl<P: VectorStoreProvider> VectorStoreProvider for EncryptedVectorStoreProvider<P> {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        self.inner.create_collection(name, dimensions).await
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.inner.delete_collection(name).await
    }

    async fn collection_exists(&self, name: &str) -> Result<bool> {
        self.inner.collection_exists(name).await
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        if vectors.len() != metadata.len() {
            return Err(crate::domain::error::Error::invalid_argument(
                "Vectors and metadata length mismatch",
            ));
        }

        // Keep vectors unencrypted for searchability, encrypt only sensitive metadata
        let mut processed_metadata = Vec::new();

        for meta in metadata {
            // Serialize sensitive metadata for encryption
            let metadata_json = serde_json::to_string(&meta)?;
            let encrypted_metadata_data = self.crypto.encrypt(metadata_json.as_bytes())?;

            // Create metadata with encryption info
            let mut processed_meta = HashMap::new();
            processed_meta.insert(
                "encrypted_metadata".to_string(),
                serde_json::to_value(&encrypted_metadata_data)?,
            );
            // Keep non-sensitive metadata unencrypted for filtering
            if let Some(content) = meta.get("content") {
                processed_meta.insert("content".to_string(), content.clone());
            }
            if let Some(file_path) = meta.get("file_path") {
                processed_meta.insert("file_path".to_string(), file_path.clone());
            }
            if let Some(line_number) = meta.get("line_number") {
                processed_meta.insert("line_number".to_string(), line_number.clone());
            }

            processed_metadata.push(processed_meta);
        }

        self.inner
            .insert_vectors(collection, vectors, processed_metadata)
            .await
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Search using the inner provider (vectors are unencrypted)
        let mut results = self
            .inner
            .search_similar(collection, query_vector, limit, filter)
            .await?;

        // Decrypt the metadata for each result
        for result in &mut results {
            if let Some(encrypted_metadata) = result.metadata.get("encrypted_metadata") {
                let metadata_envelope: EncryptedEnvelope =
                    serde_json::from_value(encrypted_metadata.clone())?;
                let metadata_json = self.crypto.decrypt(&metadata_envelope)?;
                let original_metadata: HashMap<String, serde_json::Value> =
                    serde_json::from_slice(&metadata_json)?;

                // Restore original metadata
                result.metadata = serde_json::to_value(original_metadata)?;
            }
        }

        Ok(results)
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        self.inner.delete_vectors(collection, ids).await
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = self.inner.get_stats(collection).await?;
        stats.insert("encryption_enabled".to_string(), serde_json::json!(true));
        stats.insert(
            "encryption_algorithm".to_string(),
            serde_json::json!("AES-256-GCM"),
        );
        Ok(stats)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        self.inner.flush(collection).await
    }

    fn provider_name(&self) -> &str {
        "encrypted"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::providers::vector_store::InMemoryVectorStoreProvider;

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
    async fn test_encrypted_search_returns_empty(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
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
}
