//! Provider registry for dependency injection

use crate::core::error::{Error, Result};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use crate::core::locks::{lock_rwlock_read, lock_rwlock_write};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe provider registry
#[derive(Clone)]
pub struct ProviderRegistry {
    embedding_providers: Arc<RwLock<HashMap<String, Arc<dyn EmbeddingProvider>>>>,
    vector_store_providers: Arc<RwLock<HashMap<String, Arc<dyn VectorStoreProvider>>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            embedding_providers: Arc::new(RwLock::new(HashMap::new())),
            vector_store_providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an embedding provider
    pub fn register_embedding_provider(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        let name = name.into();
        let mut providers = lock_rwlock_write(&self.embedding_providers, "ProviderRegistry::register_embedding_provider")?;

        if providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Embedding provider '{}' already registered",
                name
            )));
        }

        providers.insert(name, provider);
        Ok(())
    }

    /// Register a vector store provider
    pub fn register_vector_store_provider(
        &self,
        name: impl Into<String>,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        let name = name.into();
        let mut providers = lock_rwlock_write(&self.vector_store_providers, "ProviderRegistry::register_vector_store_provider")?;

        if providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Vector store provider '{}' already registered",
                name
            )));
        }

        providers.insert(name, provider);
        Ok(())
    }

    /// Get an embedding provider by name
    pub fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>> {
        let providers = lock_rwlock_read(&self.embedding_providers, "ProviderRegistry::get_embedding_provider")?;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Embedding provider '{}' not found", name)))
    }

    /// Get a vector store provider by name
    pub fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>> {
        let providers = lock_rwlock_read(&self.vector_store_providers, "ProviderRegistry::get_vector_store_provider")?;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("Vector store provider '{}' not found", name)))
    }

    /// List all registered embedding providers
    pub fn list_embedding_providers(&self) -> Vec<String> {
        let providers = match lock_rwlock_read(&self.embedding_providers, "ProviderRegistry::list_embedding_providers") {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        providers.keys().cloned().collect()
    }

    /// List all registered vector store providers
    pub fn list_vector_store_providers(&self) -> Vec<String> {
        let providers = match lock_rwlock_read(&self.vector_store_providers, "ProviderRegistry::list_vector_store_providers") {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
