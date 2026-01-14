//! Provider registry for dependency injection

use crate::domain::error::{Error, Result};
use crate::domain::ports::{EmbeddingProvider, VectorStoreProvider};
use crate::infrastructure::events::{SharedEventBusProvider, SystemEvent};
use dashmap::DashMap;
use std::sync::Arc;

/// Trait for provider registry
pub trait ProviderRegistryTrait: Send + Sync {
    /// Register an embedding provider with the given name
    fn register_embedding_provider(
        &self,
        name: String,
        provider: Arc<dyn EmbeddingProvider>,
    ) -> Result<()>;

    /// Register a vector store provider with the given name
    fn register_vector_store_provider(
        &self,
        name: String,
        provider: Arc<dyn VectorStoreProvider>,
    ) -> Result<()>;

    /// Get an embedding provider by name
    fn get_embedding_provider(&self, name: &str) -> Result<Arc<dyn EmbeddingProvider>>;
    /// Get a vector store provider by name
    fn get_vector_store_provider(&self, name: &str) -> Result<Arc<dyn VectorStoreProvider>>;
    /// Remove an embedding provider by name
    fn remove_embedding_provider(&self, name: &str) -> Result<()>;
    /// Remove a vector store provider by name
    fn remove_vector_store_provider(&self, name: &str) -> Result<()>;
    /// List all registered embedding provider names
    fn list_embedding_providers(&self) -> Vec<String>;
    /// List all registered vector store provider names
    fn list_vector_store_providers(&self) -> Vec<String>;
}

/// Thread-safe provider registry using DashMap to eliminate locks
#[derive(Clone)]
pub struct ProviderRegistry {
    /// Map of registered embedding providers by name
    embedding_providers: Arc<DashMap<String, Arc<dyn EmbeddingProvider + 'static>>>,
    /// Map of registered vector store providers by name
    vector_store_providers: Arc<DashMap<String, Arc<dyn VectorStoreProvider + 'static>>>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            embedding_providers: Arc::new(DashMap::new()),
            vector_store_providers: Arc::new(DashMap::new()),
        }
    }

    /// Create a new provider registry with event bus subscription
    pub fn with_event_bus(event_bus: SharedEventBusProvider) -> Self {
        let registry = Self::new();
        registry.start_event_listener(event_bus);
        registry
    }

    /// Start listening for system events (ADR-007: Subsystem Control)
    pub fn start_event_listener(&self, event_bus: SharedEventBusProvider) {
        let registry = self.clone();

        tokio::spawn(async move {
            if let Ok(mut receiver) = event_bus.subscribe().await {
                tracing::info!("[REGISTRY] Event listener started");
                while let Ok(event) = receiver.recv().await {
                    match event {
                        SystemEvent::ProviderRestart {
                            provider_type,
                            provider_id,
                        } => {
                            tracing::info!(
                                "[REGISTRY] Provider restart requested: {}:{}",
                                provider_type,
                                provider_id
                            );
                            // For stateless API clients, "restart" means verifying health
                            // and potentially resetting any connection pools or cached state
                            match provider_type.as_str() {
                                "embedding" => {
                                    // Clone provider Arc out of DashMap to avoid lifetime issues
                                    let provider_opt: Option<Arc<dyn EmbeddingProvider + 'static>> =
                                        registry
                                            .embedding_providers
                                            .get(&provider_id)
                                            .map(|p| Arc::clone(p.value()));
                                    if let Some(provider) = provider_opt {
                                        match provider.health_check().await {
                                            Ok(()) => {
                                                tracing::info!(
                                                "[REGISTRY] Embedding provider '{}' health check passed",
                                                provider_id
                                            );
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                "[REGISTRY] Embedding provider '{}' health check failed: {}",
                                                provider_id,
                                                e
                                            );
                                            }
                                        }
                                    } else {
                                        tracing::warn!(
                                        "[REGISTRY] Embedding provider '{}' not found for restart",
                                        provider_id
                                    );
                                    }
                                }
                                "vector_store" => {
                                    // Clone provider Arc out of DashMap to avoid lifetime issues
                                    let provider_opt: Option<
                                        Arc<dyn VectorStoreProvider + 'static>,
                                    > = registry
                                        .vector_store_providers
                                        .get(&provider_id)
                                        .map(|p| Arc::clone(p.value()));
                                    if let Some(provider) = provider_opt {
                                        match provider.health_check().await {
                                            Ok(()) => {
                                                tracing::info!(
                                                "[REGISTRY] Vector store provider '{}' health check passed",
                                                provider_id
                                            );
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                "[REGISTRY] Vector store provider '{}' health check failed: {}",
                                                provider_id,
                                                e
                                            );
                                            }
                                        }
                                    } else {
                                        tracing::warn!(
                                        "[REGISTRY] Vector store provider '{}' not found for restart",
                                        provider_id
                                    );
                                    }
                                }
                                _ => {
                                    tracing::warn!(
                                        "[REGISTRY] Unknown provider type: {}",
                                        provider_type
                                    );
                                }
                            }
                        }
                        SystemEvent::ProviderReconfigure {
                            provider_type,
                            config: _,
                        } => {
                            // Reconfiguration requires factory pattern to recreate provider
                            // For now, log the request - full implementation would require
                            // passing the ProviderFactory to the registry
                            tracing::warn!(
                            "[REGISTRY] Provider reconfigure requested for '{}' - not yet implemented",
                            provider_type
                        );
                        }
                        SystemEvent::SubsystemHealthCheck { subsystem_id } => {
                            // Run health checks on all providers if subsystem matches
                            if subsystem_id.starts_with("embedding") || subsystem_id == "providers"
                            {
                                tracing::info!(
                                    "[REGISTRY] Running health checks on embedding providers"
                                );
                                // Collect providers first to avoid lifetime issues with async
                                let providers: Vec<(String, Arc<dyn EmbeddingProvider + 'static>)> =
                                    registry
                                        .embedding_providers
                                        .iter()
                                        .map(|entry| {
                                            (entry.key().clone(), Arc::clone(entry.value()))
                                        })
                                        .collect();
                                for (name, provider) in providers {
                                    match provider.health_check().await {
                                        Ok(()) => {
                                            tracing::info!(
                                                "[REGISTRY] Embedding '{}': healthy",
                                                name
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "[REGISTRY] Embedding '{}': unhealthy - {}",
                                                name,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            if subsystem_id.starts_with("vector_store")
                                || subsystem_id == "providers"
                            {
                                tracing::info!(
                                    "[REGISTRY] Running health checks on vector store providers"
                                );
                                // Collect providers first to avoid lifetime issues with async
                                let providers: Vec<(
                                    String,
                                    Arc<dyn VectorStoreProvider + 'static>,
                                )> = registry
                                    .vector_store_providers
                                    .iter()
                                    .map(|entry| (entry.key().clone(), Arc::clone(entry.value())))
                                    .collect();
                                for (name, provider) in providers {
                                    match provider.health_check().await {
                                        Ok(()) => {
                                            tracing::info!(
                                                "[REGISTRY] Vector store '{}': healthy",
                                                name
                                            );
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                "[REGISTRY] Vector store '{}': unhealthy - {}",
                                                name,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // Ignore other events
                        }
                    }
                }
                tracing::warn!("[REGISTRY] Event listener stopped");
            }
        });
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

    /// Remove an embedding provider
    fn remove_embedding_provider(&self, name: &str) -> Result<()> {
        if self.embedding_providers.remove(name).is_some() {
            Ok(())
        } else {
            Err(Error::not_found(format!(
                "Embedding provider '{}' not found",
                name
            )))
        }
    }

    /// Remove a vector store provider
    fn remove_vector_store_provider(&self, name: &str) -> Result<()> {
        if self.vector_store_providers.remove(name).is_some() {
            Ok(())
        } else {
            Err(Error::not_found(format!(
                "Vector store provider '{}' not found",
                name
            )))
        }
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
