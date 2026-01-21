//! dill Catalog - IoC Container Configuration
//!
//! This module provides the dill-based IoC container that wraps the existing
//! manual DI pattern. It uses `add_value()` for services created by the
//! linkme registry system and manual constructors.
//!
//! ## Architecture
//!
//! ```text
//! linkme (compile-time)     dill Catalog (runtime)
//! ─────────────────────     ─────────────────────
//! EMBEDDING_PROVIDERS  →    resolve_providers()
//!                                  ↓
//!                           CatalogBuilder::add_value(provider)
//!                                  ↓
//!                           Catalog::get_one::<dyn EmbeddingProvider>()
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! let catalog = build_catalog(config).await?;
//! let embedding: Arc<dyn EmbeddingProvider> = catalog.get_one()?;
//! ```

use crate::config::AppConfig;
use crate::di::admin::{
    CacheAdminInterface, CacheAdminService, EmbeddingAdminInterface, EmbeddingAdminService,
    LanguageAdminInterface, LanguageAdminService, VectorStoreAdminInterface,
    VectorStoreAdminService,
};
use crate::di::handles::{
    CacheProviderHandle, EmbeddingProviderHandle, LanguageProviderHandle, VectorStoreProviderHandle,
};
use crate::di::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};
use crate::infrastructure::{
    admin::{NullIndexingOperations, NullPerformanceMetrics},
    auth::NullAuthService,
    events::TokioBroadcastEventBus,
    lifecycle::DefaultShutdownCoordinator,
    metrics::NullSystemMetricsCollector,
    snapshot::NullSnapshotProvider,
    sync::NullSyncProvider,
};
use dill::{Catalog, CatalogBuilder};
use mcb_domain::error::Result;
use mcb_domain::ports::admin::{
    IndexingOperationsInterface, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_domain::ports::infrastructure::{
    AuthServiceInterface, EventBusProvider, SnapshotProvider, SyncProvider,
    SystemMetricsCollectorInterface,
};
// Provider traits imported for documentation and future use
#[allow(unused_imports)]
use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use std::sync::Arc;
use tracing::info;

/// Build the dill Catalog with all application services
///
/// This function creates the IoC container with:
/// - External providers (resolved via linkme registry)
/// - Provider handles (for runtime switching)
/// - Admin services (for API-based provider management)
/// - Infrastructure services (null implementations by default)
///
/// # Type Bindings
///
/// The catalog binds these trait objects for dependency injection:
///
/// | Trait | Resolved From |
/// |-------|---------------|
/// | `dyn EmbeddingProvider` | linkme registry → config → handle |
/// | `dyn VectorStoreProvider` | linkme registry → config → handle |
/// | `dyn CacheProvider` | linkme registry → config → handle |
/// | `dyn LanguageChunkingProvider` | linkme registry → config → handle |
/// | `dyn AuthServiceInterface` | NullAuthService (default) |
/// | `dyn EventBusProvider` | TokioBroadcastEventBus (default) |
///
pub async fn build_catalog(config: AppConfig) -> Result<Catalog> {
    info!("Building dill Catalog with provider handles");

    let config = Arc::new(config);

    // ========================================================================
    // Create Resolvers (components that use linkme registry)
    // ========================================================================

    let embedding_resolver = Arc::new(EmbeddingProviderResolver::new(config.clone()));
    let vector_store_resolver = Arc::new(VectorStoreProviderResolver::new(config.clone()));
    let cache_resolver = Arc::new(CacheProviderResolver::new(config.clone()));
    let language_resolver = Arc::new(LanguageProviderResolver::new(config.clone()));

    info!("Created provider resolvers");

    // ========================================================================
    // Resolve initial providers from config
    // ========================================================================

    let embedding_provider = embedding_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Embedding: {e}")))?;

    let vector_store_provider = vector_store_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("VectorStore: {e}")))?;

    let cache_provider = cache_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Cache: {e}")))?;

    let language_provider = language_resolver
        .resolve_from_config()
        .map_err(|e| mcb_domain::error::Error::configuration(format!("Language: {e}")))?;

    info!(
        "Resolved providers: embedding={}, vector_store={}, cache={}, language={}",
        embedding_provider.provider_name(),
        vector_store_provider.provider_name(),
        cache_provider.provider_name(),
        language_provider.provider_name()
    );

    // ========================================================================
    // Create Handles (RwLock wrappers for runtime switching)
    // ========================================================================

    let embedding_handle = Arc::new(EmbeddingProviderHandle::new(embedding_provider.clone()));
    let vector_store_handle = Arc::new(VectorStoreProviderHandle::new(
        vector_store_provider.clone(),
    ));
    let cache_handle = Arc::new(CacheProviderHandle::new(cache_provider.clone()));
    let language_handle = Arc::new(LanguageProviderHandle::new(language_provider.clone()));

    info!("Created provider handles");

    // ========================================================================
    // Create Admin Services (for API-based provider management)
    // ========================================================================

    let embedding_admin: Arc<dyn EmbeddingAdminInterface> = Arc::new(EmbeddingAdminService::new(
        embedding_resolver.clone(),
        embedding_handle.clone(),
    ));
    let vector_store_admin: Arc<dyn VectorStoreAdminInterface> = Arc::new(
        VectorStoreAdminService::new(vector_store_resolver.clone(), vector_store_handle.clone()),
    );
    let cache_admin: Arc<dyn CacheAdminInterface> = Arc::new(CacheAdminService::new(
        cache_resolver.clone(),
        cache_handle.clone(),
    ));
    let language_admin: Arc<dyn LanguageAdminInterface> = Arc::new(LanguageAdminService::new(
        language_resolver.clone(),
        language_handle.clone(),
    ));

    info!("Created admin services");

    // ========================================================================
    // Create Infrastructure Services (null implementations by default)
    // ========================================================================

    let auth_service: Arc<dyn AuthServiceInterface> = Arc::new(NullAuthService::new());
    let event_bus: Arc<dyn EventBusProvider> = Arc::new(TokioBroadcastEventBus::new());
    let metrics_collector: Arc<dyn SystemMetricsCollectorInterface> =
        Arc::new(NullSystemMetricsCollector::new());
    let sync_provider: Arc<dyn SyncProvider> = Arc::new(NullSyncProvider::new());
    let snapshot_provider: Arc<dyn SnapshotProvider> = Arc::new(NullSnapshotProvider::new());
    let shutdown_coordinator: Arc<dyn ShutdownCoordinator> =
        Arc::new(DefaultShutdownCoordinator::new());
    let performance_metrics: Arc<dyn PerformanceMetricsInterface> =
        Arc::new(NullPerformanceMetrics);
    let indexing_operations: Arc<dyn IndexingOperationsInterface> =
        Arc::new(NullIndexingOperations);

    info!("Created infrastructure services");

    // ========================================================================
    // Build the Catalog with all services
    // ========================================================================

    let catalog = CatalogBuilder::new()
        // Configuration
        .add_value(config)
        // Provider traits (via handles)
        .add_value(embedding_provider)
        .add_value(vector_store_provider)
        .add_value(cache_provider)
        .add_value(language_provider)
        // Provider handles (for runtime switching)
        .add_value(embedding_handle)
        .add_value(vector_store_handle)
        .add_value(cache_handle)
        .add_value(language_handle)
        // Provider resolvers (linkme registry access)
        .add_value(embedding_resolver)
        .add_value(vector_store_resolver)
        .add_value(cache_resolver)
        .add_value(language_resolver)
        // Admin services (API-based provider management)
        .add_value(embedding_admin)
        .add_value(vector_store_admin)
        .add_value(cache_admin)
        .add_value(language_admin)
        // Infrastructure services
        .add_value(auth_service)
        .add_value(event_bus)
        .add_value(metrics_collector)
        .add_value(sync_provider)
        .add_value(snapshot_provider)
        .add_value(shutdown_coordinator)
        .add_value(performance_metrics)
        .add_value(indexing_operations)
        .build();

    info!("Built dill Catalog with {} services", 20);

    Ok(catalog)
}

// Note: Provider access should go through AppContext, not directly via catalog.
// The dill Catalog is used for infrastructure service lifecycle management.
// Providers are accessed via: app_context.embedding_handle().get()
//
// The catalog stores handles and resolvers for internal use, but the recommended
// pattern for external access is through AppContext methods.
