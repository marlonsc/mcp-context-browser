//! DI Container Bootstrap - Provider Handles + Infrastructure Services
//!
//! Provides the composition root using runtime-swappable provider handles
//! and direct infrastructure service storage.
//!
//! ## Architecture
//!
//! External providers (embedding, vector_store, cache, language) are resolved
//! via the linkme-based registry system. Provider Handles allow runtime switching
//! via admin API. Infrastructure services are stored directly in AppContext.
//!
//! ```text
//! AppConfig → Resolvers → Handles (RwLock) → Domain Services
//!                ↑              ↑
//!            linkme         AdminServices
//!           registry       (switch via API)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Create AppContext with provider handles
//! let context = init_app(AppConfig::default()).await?;
//!
//! // Get provider handles (runtime-swappable)
//! let embedding = context.embedding_handle();
//! let current_provider = embedding.get();
//!
//! // Switch provider via admin service
//! let admin = context.embedding_admin();
//! admin.switch_provider(new_config)?;
//!
//! // Access infrastructure services directly
//! let auth = context.auth();
//! let event_bus = context.event_bus();
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
use mcb_domain::error::Result;
use mcb_domain::ports::admin::{
    IndexingOperationsInterface, PerformanceMetricsInterface, ShutdownCoordinator,
};
use mcb_domain::ports::infrastructure::{
    AuthServiceInterface, EventBusProvider, SnapshotProvider, SyncProvider,
    SystemMetricsCollectorInterface,
};
use std::sync::Arc;
use tracing::info;

/// Application context with provider handles and infrastructure services
///
/// This is the composition root that combines:
/// - Provider handles (runtime-swappable via RwLock)
/// - Provider resolvers (linkme registry access)
/// - Admin services (switch providers via API)
/// - Infrastructure services (direct storage)
pub struct AppContext {
    /// Application configuration
    pub config: Arc<AppConfig>,

    // ========================================================================
    // Provider Handles (runtime-swappable)
    // ========================================================================
    embedding_handle: Arc<EmbeddingProviderHandle>,
    vector_store_handle: Arc<VectorStoreProviderHandle>,
    cache_handle: Arc<CacheProviderHandle>,
    language_handle: Arc<LanguageProviderHandle>,

    // ========================================================================
    // Provider Resolvers (linkme registry access)
    // Reserved for future admin API operations (list/switch providers)
    // ========================================================================
    #[allow(dead_code)] // Reserved for admin API: list available providers
    embedding_resolver: Arc<EmbeddingProviderResolver>,
    #[allow(dead_code)] // Reserved for admin API: list available providers
    vector_store_resolver: Arc<VectorStoreProviderResolver>,
    #[allow(dead_code)] // Reserved for admin API: list available providers
    cache_resolver: Arc<CacheProviderResolver>,
    #[allow(dead_code)] // Reserved for admin API: list available providers
    language_resolver: Arc<LanguageProviderResolver>,

    // ========================================================================
    // Admin Services (switch providers via API)
    // ========================================================================
    embedding_admin: Arc<dyn EmbeddingAdminInterface>,
    vector_store_admin: Arc<dyn VectorStoreAdminInterface>,
    cache_admin: Arc<dyn CacheAdminInterface>,
    language_admin: Arc<dyn LanguageAdminInterface>,

    // ========================================================================
    // Infrastructure Services (direct storage)
    // ========================================================================
    auth_service: Arc<dyn AuthServiceInterface>,
    event_bus: Arc<dyn EventBusProvider>,
    metrics_collector: Arc<dyn SystemMetricsCollectorInterface>,
    sync_provider: Arc<dyn SyncProvider>,
    snapshot_provider: Arc<dyn SnapshotProvider>,
    shutdown_coordinator: Arc<dyn ShutdownCoordinator>,
    performance_metrics: Arc<dyn PerformanceMetricsInterface>,
    indexing_operations: Arc<dyn IndexingOperationsInterface>,
}

impl AppContext {
    // ========================================================================
    // Provider Handles (runtime-swappable)
    // ========================================================================

    /// Get embedding provider handle
    pub fn embedding_handle(&self) -> Arc<EmbeddingProviderHandle> {
        self.embedding_handle.clone()
    }

    /// Get vector store provider handle
    pub fn vector_store_handle(&self) -> Arc<VectorStoreProviderHandle> {
        self.vector_store_handle.clone()
    }

    /// Get cache provider handle
    pub fn cache_handle(&self) -> Arc<CacheProviderHandle> {
        self.cache_handle.clone()
    }

    /// Get language provider handle
    pub fn language_handle(&self) -> Arc<LanguageProviderHandle> {
        self.language_handle.clone()
    }

    // ========================================================================
    // Admin Services (switch providers via API)
    // ========================================================================

    /// Get embedding admin service for runtime provider switching
    pub fn embedding_admin(&self) -> Arc<dyn EmbeddingAdminInterface> {
        self.embedding_admin.clone()
    }

    /// Get vector store admin service
    pub fn vector_store_admin(&self) -> Arc<dyn VectorStoreAdminInterface> {
        self.vector_store_admin.clone()
    }

    /// Get cache admin service
    pub fn cache_admin(&self) -> Arc<dyn CacheAdminInterface> {
        self.cache_admin.clone()
    }

    /// Get language admin service
    pub fn language_admin(&self) -> Arc<dyn LanguageAdminInterface> {
        self.language_admin.clone()
    }

    // ========================================================================
    // Infrastructure Services (direct access)
    // ========================================================================

    /// Get auth service
    pub fn auth(&self) -> Arc<dyn AuthServiceInterface> {
        self.auth_service.clone()
    }

    /// Get event bus
    pub fn event_bus(&self) -> Arc<dyn EventBusProvider> {
        self.event_bus.clone()
    }

    /// Get metrics collector
    pub fn metrics(&self) -> Arc<dyn SystemMetricsCollectorInterface> {
        self.metrics_collector.clone()
    }

    /// Get sync provider
    pub fn sync(&self) -> Arc<dyn SyncProvider> {
        self.sync_provider.clone()
    }

    /// Get snapshot provider
    pub fn snapshot(&self) -> Arc<dyn SnapshotProvider> {
        self.snapshot_provider.clone()
    }

    /// Get shutdown coordinator
    pub fn shutdown(&self) -> Arc<dyn ShutdownCoordinator> {
        self.shutdown_coordinator.clone()
    }

    /// Get performance metrics
    pub fn performance(&self) -> Arc<dyn PerformanceMetricsInterface> {
        self.performance_metrics.clone()
    }

    /// Get indexing operations
    pub fn indexing(&self) -> Arc<dyn IndexingOperationsInterface> {
        self.indexing_operations.clone()
    }
}

impl std::fmt::Debug for AppContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppContext")
            .field("embedding", &self.embedding_handle)
            .field("vector_store", &self.vector_store_handle)
            .field("cache", &self.cache_handle)
            .field("language", &self.language_handle)
            .finish_non_exhaustive()
    }
}

/// Initialize application context with provider handles and infrastructure services
///
/// Creates:
/// - Provider Resolvers (using linkme registry)
/// - Provider Handles (RwLock for runtime switching)
/// - Admin Services (for API-based provider management)
/// - Infrastructure Services (null implementations by default)
///
/// Note: Providers are auto-registered via linkme distributed slices when
/// mcb-providers is linked. No explicit registration call is needed.
pub async fn init_app(config: AppConfig) -> Result<AppContext> {
    info!("Initializing application context with provider handles");

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

    let embedding_handle = Arc::new(EmbeddingProviderHandle::new(embedding_provider));
    let vector_store_handle = Arc::new(VectorStoreProviderHandle::new(vector_store_provider));
    let cache_handle = Arc::new(CacheProviderHandle::new(cache_provider));
    let language_handle = Arc::new(LanguageProviderHandle::new(language_provider));

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

    Ok(AppContext {
        config,
        embedding_handle,
        vector_store_handle,
        cache_handle,
        language_handle,
        embedding_resolver,
        vector_store_resolver,
        cache_resolver,
        language_resolver,
        embedding_admin,
        vector_store_admin,
        cache_admin,
        language_admin,
        auth_service,
        event_bus,
        metrics_collector,
        sync_provider,
        snapshot_provider,
        shutdown_coordinator,
        performance_metrics,
        indexing_operations,
    })
}

/// Initialize application for testing
pub async fn init_test_app() -> Result<AppContext> {
    let config = AppConfig::default();
    init_app(config).await
}

/// Type alias for dispatch.rs compatibility
pub type DiContainer = AppContext;

/// Convenience function to create context for testing
pub async fn create_test_container() -> Result<AppContext> {
    init_test_app().await
}
