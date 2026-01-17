//! Domain Services DI Module
//!
//! Provides domain service implementations that can be injected into the server.
//! These services implement domain interfaces using infrastructure components.
//!
//! ## Runtime Factory Pattern
//!
//! Services are created via `DomainServicesFactory::create_services()` at runtime
//! rather than through Shaku DI, because they require runtime configuration
//! (embedding provider, vector store, cache).

use crate::cache::provider::SharedCacheProvider;
use crate::config::AppConfig;
use crate::crypto::CryptoService;
use mcb_application::use_cases::{ContextServiceImpl, IndexingServiceImpl, SearchServiceImpl};
use mcb_domain::domain_services::search::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
};
use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use std::sync::Arc;

/// Domain services container
#[derive(Clone)]
pub struct DomainServicesContainer {
    pub context_service: Arc<dyn ContextServiceInterface>,
    pub search_service: Arc<dyn SearchServiceInterface>,
    pub indexing_service: Arc<dyn IndexingServiceInterface>,
}

/// Domain services factory - creates services with runtime dependencies
pub struct DomainServicesFactory;

impl DomainServicesFactory {
    /// Create domain services using DI container
    ///
    /// Note: This factory now expects the services to be resolved from a DI container
    /// that has been properly configured with the application use cases.
    pub fn create_services_from_container(
        container: &(impl shaku::HasComponent<dyn ContextServiceInterface>
            + shaku::HasComponent<dyn SearchServiceInterface>
            + shaku::HasComponent<dyn IndexingServiceInterface>),
    ) -> DomainServicesContainer {
        let context_service = container.resolve_ref();
        let search_service = container.resolve_ref();
        let indexing_service = container.resolve_ref();

        DomainServicesContainer {
            context_service: Arc::clone(context_service),
            search_service: Arc::clone(search_service),
            indexing_service: Arc::clone(indexing_service),
        }
    }
}

// Note: Service implementations have been moved to mcb-application crate
// This module now provides a factory for creating services at runtime
