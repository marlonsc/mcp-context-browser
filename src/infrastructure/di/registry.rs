//! Provider registry for dependency injection

use crate::domain::error::{Error, Result};
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use dashmap::DashMap;
use std::sync::Arc;

/// Trait for provider registry
pub trait ProviderRegistryTrait: Send + Sync {
    fn register_embedding_provider(
        &self,
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()>;

    fn register_vector_store_provider(
        &self,
        name: String,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()>;

    fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>>;
    fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>>;
    fn list_embedding_providers(&self) -> Vec<String>;
    fn list_vector_store_providers(&self) -> Vec<String>;
}

/// Thread-safe provider registry using DashMap to eliminate locks
#[derive(Clone)]
pub struct ProviderRegistry {
    embedding_providers: Arc<DashMap<String, Arc<dyn EmbeddingProvider>>>,
    vector_store_providers: Arc<DashMap<String, Arc<dyn VectorStoreProvider>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            embedding_providers: Arc::new(DashMap::new()),
            vector_store_providers: Arc::new(DashMap::new()),
        }
    }
}

impl ProviderRegistryTrait for ProviderRegistry {
    /// Register an embedding provider
    fn register_embedding_provider(
        &self,
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()> {
        if self.embedding_providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Embedding provider '{}' already registered",
                name
            )));
        }

        self.embedding_providers.insert(name, provider);
        Ok(())
    }

    /// Register a vector store provider
    fn register_vector_store_provider(
        &self,
        name: String,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()> {
        if self.vector_store_providers.contains_key(&name) {
            return Err(Error::generic(format!(
                "Vector store provider '{}' already registered",
                name
            )));
        }

        self.vector_store_providers.insert(name, provider);
        Ok(())
    }

    /// Get an embedding provider by name
    fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>> {
        self.embedding_providers
            .get(name)
            .map(|p| Arc::clone(p.value()))
            .ok_or_else(|| Error::not_found(format!("Embedding provider '{}' not found", name)))
    }

    /// Get a vector store provider by name
    fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>> {
        self.vector_store_providers
            .get(name)
            .map(|p| Arc::clone(p.value()))
            .ok_or_else(|| Error::not_found(format!("Vector store provider '{}' not found", name)))
    }

    /// List all registered embedding providers
    fn list_embedding_providers(&self) -> Vec<String> {
        self.embedding_providers
            .iter()
            .map(|p| p.key().clone())
            .collect()
    }

    /// List all registered vector store providers
    fn list_vector_store_providers(&self) -> Vec<String> {
        self.vector_store_providers
            .iter()
            .map(|p| p.key().clone())
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
