//! Encrypted vector store wrapper
//!
//! Provides encryption at rest for any vector store provider.
//! Wraps existing providers with AES-256-GCM encryption.
//!
//! ## Architecture
//!
//! This provider follows the Decorator pattern:
//! - Wraps any `VectorStoreProvider` implementation
//! - Encrypts metadata before storage
//! - Vectors remain unencrypted for searchability
//!
//! ## Usage
//!
//! ```ignore
//! use mcb_providers::vector_store::EncryptedVectorStoreProvider;
//! use mcb_domain::ports::providers::CryptoProvider;
//!
//! let encrypted = EncryptedVectorStoreProvider::new(inner_provider, crypto_service);
//! ```

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{CryptoProvider, EncryptedData};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::value_objects::{Embedding, SearchResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Encrypted vector store provider
///
/// Wraps any VectorStoreProvider implementation to provide encryption at rest.
/// Vectors are stored unencrypted for searchability, but metadata is encrypted
/// using AES-256-GCM.
///
/// ## Encryption Strategy
///
/// - **Vectors**: Unencrypted (required for similarity search)
/// - **Metadata**: Encrypted, with essential fields kept in plaintext for filtering
///
/// Essential unencrypted fields:
/// - `content` - For search result construction
/// - `file_path` - For search result construction
/// - `start_line` - For search result construction
/// - `language` - For search result construction
pub struct EncryptedVectorStoreProvider<P: VectorStoreProvider> {
    /// Underlying vector store provider
    inner: P,
    /// Cryptography provider
    crypto: Arc<dyn CryptoProvider>,
}

impl<P: VectorStoreProvider> EncryptedVectorStoreProvider<P> {
    /// Create a new encrypted vector store provider
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying vector store provider to wrap
    /// * `crypto` - The cryptography provider for encryption operations
    pub fn new(inner: P, crypto: Arc<dyn CryptoProvider>) -> Self {
        Self { inner, crypto }
    }

    /// Get a reference to the inner provider
    pub fn inner(&self) -> &P {
        &self.inner
    }

    /// Get a reference to the crypto provider
    pub fn crypto(&self) -> &Arc<dyn CryptoProvider> {
        &self.crypto
    }

    /// Encrypt metadata while preserving searchable fields
    fn encrypt_metadata(&self, meta: &HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Serialize and encrypt sensitive metadata
        let metadata_json = serde_json::to_string(meta).map_err(|e| Error::Infrastructure {
            message: format!("Failed to serialize metadata: {}", e),
            source: Some(Box::new(e)),
        })?;

        let encrypted_data = self.crypto.encrypt(metadata_json.as_bytes())?;

        let mut processed = HashMap::new();
        processed.insert(
            "encrypted_metadata".to_string(),
            serde_json::to_value(&encrypted_data).map_err(|e| Error::Infrastructure {
                message: format!("Failed to serialize encrypted data: {}", e),
                source: Some(Box::new(e)),
            })?,
        );

        // Preserve unencrypted fields for filtering and SearchResult construction
        for key in ["content", "file_path", "language"] {
            if let Some(val) = meta.get(key) {
                processed.insert(key.to_string(), val.clone());
            }
        }
        if let Some(val) = meta.get("start_line").or_else(|| meta.get("line_number")) {
            processed.insert("start_line".to_string(), val.clone());
        }

        Ok(processed)
    }
}

#[async_trait]
impl<P: VectorStoreProvider> VectorStoreAdmin for EncryptedVectorStoreProvider<P> {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        self.inner.collection_exists(name).await
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, Value>> {
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

#[async_trait]
impl<P: VectorStoreProvider> VectorStoreProvider for EncryptedVectorStoreProvider<P> {
    async fn create_collection(&self, name: &str, dimensions: usize) -> Result<()> {
        self.inner.create_collection(name, dimensions).await
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        self.inner.delete_collection(name).await
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>> {
        if vectors.len() != metadata.len() {
            return Err(Error::invalid_argument(
                "Vectors and metadata length mismatch",
            ));
        }

        // Encrypt metadata while keeping vectors unencrypted for searchability
        let processed_metadata: Vec<_> = metadata
            .iter()
            .map(|meta| self.encrypt_metadata(meta))
            .collect::<Result<Vec<_>>>()?;

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
        // Note: The inner provider returns results with partial metadata (unencrypted fields only)
        // We trust the inner provider's SearchResult structure is correct
        self.inner
            .search_similar(collection, query_vector, limit, filter)
            .await
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        self.inner.delete_vectors(collection, ids).await
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        // Delegate to inner provider - SearchResult fields are extracted from stored metadata
        self.inner.get_vectors_by_ids(collection, ids).await
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Delegate to inner provider - SearchResult fields are extracted from stored metadata
        self.inner.list_vectors(collection, limit).await
    }
}

/// Decrypt metadata from an encrypted search result
///
/// This function is useful when you need to access the full encrypted metadata
/// from a search result.
///
/// # Arguments
///
/// * `crypto` - The crypto provider to use for decryption
/// * `metadata` - The metadata map containing encrypted_metadata field
///
/// # Returns
///
/// The decrypted metadata as a HashMap
pub fn decrypt_metadata(
    crypto: &dyn CryptoProvider,
    metadata: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>> {
    let encrypted_value = metadata
        .get("encrypted_metadata")
        .ok_or_else(|| Error::invalid_argument("No encrypted_metadata field found"))?;

    let encrypted_data: EncryptedData =
        serde_json::from_value(encrypted_value.clone()).map_err(|e| Error::Infrastructure {
            message: format!("Failed to deserialize encrypted data: {}", e),
            source: Some(Box::new(e)),
        })?;

    let decrypted_bytes = crypto.decrypt(&encrypted_data)?;

    let decrypted_str = String::from_utf8(decrypted_bytes).map_err(|e| Error::Infrastructure {
        message: format!("Failed to decode decrypted data as UTF-8: {}", e),
        source: Some(Box::new(e)),
    })?;

    serde_json::from_str(&decrypted_str).map_err(|e| Error::Infrastructure {
        message: format!("Failed to parse decrypted metadata: {}", e),
        source: Some(Box::new(e)),
    })
}
