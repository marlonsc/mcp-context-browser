//! DI Component Registry
//!
//! Provides a registry for managing and resolving infrastructure components
//! at runtime, following the Service Locator pattern.

use crate::cache::provider::SharedCacheProvider;
use crate::crypto::CryptoService;
use crate::health::HealthRegistry;
use mcb_domain::error::{Error, Result};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Component registry for infrastructure services
#[derive(Clone)]
pub struct ComponentRegistry {
    components: Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>,
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component in the registry
    pub async fn register<T: 'static + Send + Sync>(&self, component: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut components = self.components.write().await;

        if components.contains_key(&type_id) {
            return Err(Error::Infrastructure {
                message: format!("Component of type {} is already registered", std::any::type_name::<T>()),
                source: None,
            });
        }

        components.insert(type_id, Box::new(component));
        Ok(())
    }

    /// Get a component from the registry
    pub async fn get<T: 'static + Clone>(&self) -> Result<T> {
        let type_id = TypeId::of::<T>();
        let components = self.components.read().await;

        let component = components.get(&type_id)
            .ok_or_else(|| Error::Infrastructure {
                message: format!("Component of type {} not found in registry", std::any::type_name::<T>()),
                source: None,
            })?;

        let component = component.downcast_ref::<T>()
            .ok_or_else(|| Error::Infrastructure {
                message: format!("Component type mismatch for {}", std::any::type_name::<T>()),
                source: None,
            })?;

        Ok(component.clone())
    }

    /// Check if a component is registered
    pub async fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let components = self.components.read().await;
        components.contains_key(&type_id)
    }

    /// Remove a component from the registry
    pub async fn remove<T: 'static>(&self) -> Result<()> {
        let type_id = TypeId::of::<T>();
        let mut components = self.components.write().await;

        if components.remove(&type_id).is_none() {
            return Err(Error::Infrastructure {
                message: format!("Component of type {} not found in registry", std::any::type_name::<T>()),
                source: None,
            });
        }

        Ok(())
    }

    /// Get the number of registered components
    pub async fn count(&self) -> usize {
        let components = self.components.read().await;
        components.len()
    }

    /// Clear all components from the registry
    pub async fn clear(&self) {
        let mut components = self.components.write().await;
        components.clear();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Service locator for infrastructure components
pub struct ServiceLocator {
    registry: ComponentRegistry,
}

impl ServiceLocator {
    /// Create a new service locator
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
        }
    }

    /// Register infrastructure components
    pub async fn register_infrastructure_components(
        &self,
        cache: SharedCacheProvider,
        crypto: CryptoService,
        health: HealthRegistry,
    ) -> Result<()> {
        self.registry.register(cache).await?;
        self.registry.register(crypto).await?;
        self.registry.register(health).await?;
        Ok(())
    }

    /// Get the cache provider
    pub async fn cache(&self) -> Result<SharedCacheProvider> {
        self.registry.get().await
    }

    /// Get the crypto service
    pub async fn crypto(&self) -> Result<CryptoService> {
        self.registry.get().await
    }

    /// Get the health registry
    pub async fn health(&self) -> Result<HealthRegistry> {
        self.registry.get().await
    }

    /// Get the component registry for advanced usage
    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }
}

impl Default for ServiceLocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestComponent {
        value: String,
    }

    #[tokio::test]
    async fn test_component_registry() {
        let registry = ComponentRegistry::new();

        let component = TestComponent { value: "test".to_string() };

        // Register component
        registry.register(component.clone()).await.unwrap();

        // Get component
        let retrieved: TestComponent = registry.get().await.unwrap();
        assert_eq!(retrieved, component);

        // Check existence
        assert!(registry.has::<TestComponent>().await);

        // Count components
        assert_eq!(registry.count().await, 1);

        // Remove component
        registry.remove::<TestComponent>().await.unwrap();
        assert!(!registry.has::<TestComponent>().await);
    }

    #[tokio::test]
    async fn test_service_locator() {
        let locator = ServiceLocator::new();

        let cache = crate::cache::CacheProviderFactory::create_null();
        let crypto = crate::crypto::CryptoService::new(crate::crypto::CryptoService::generate_master_key()).unwrap();
        let health = crate::health::HealthRegistry::new();

        locator.register_infrastructure_components(cache.clone(), crypto.clone(), health.clone()).await.unwrap();

        // Test that components can be retrieved
        let retrieved_cache = locator.cache().await.unwrap();
        let retrieved_crypto = locator.crypto().await.unwrap();
        let retrieved_health = locator.health().await.unwrap();

        // Components should be accessible
        assert!(retrieved_cache.get::<_, String>("test").await.unwrap().is_none());
    }
}