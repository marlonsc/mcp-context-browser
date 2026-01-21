//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! ## Organization
//!
//! - **providers/** - External service providers (embeddings, vector stores, search)
//! - **infrastructure/** - Infrastructure services (sync, snapshots)
//! - **registry/** - Auto-registration system for plugin providers
//! - **services.rs** - Application service interfaces (context, search, indexing)
//! - **admin.rs** - Administrative interfaces for system management

/// Administrative interfaces for system management and monitoring
pub mod admin;
/// Infrastructure service ports
pub mod infrastructure;
/// External service provider ports
pub mod providers;
/// Provider registry for dynamic provider discovery
pub mod registry;
/// Application service interfaces
pub mod services;

// Re-export commonly used port traits for convenience
pub use admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, LifecycleManaged, PerformanceMetricsData,
    PerformanceMetricsInterface, PortServiceState, ShutdownCoordinator,
};
pub use infrastructure::snapshot::SyncProvider;
pub use infrastructure::{
    AuthServiceInterface, DomainEventStream, EventBusProvider, LockGuard, LockProvider,
    ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SyncOptions, SyncResult, SystemMetrics,
    SystemMetricsCollectorInterface,
};
pub use providers::{EmbeddingProvider, HybridSearchProvider, VectorStoreProvider};
pub use registry::{
    CacheProviderConfig, CacheProviderEntry, EmbeddingProviderConfig, EmbeddingProviderEntry,
    LanguageProviderConfig, LanguageProviderEntry, VectorStoreProviderConfig,
    VectorStoreProviderEntry, list_cache_providers, list_embedding_providers,
    list_language_providers, list_vector_store_providers, resolve_cache_provider,
    resolve_embedding_provider, resolve_language_provider, resolve_vector_store_provider,
};
pub use services::{
    BatchIndexingServiceInterface, ChunkingOrchestratorInterface, ContextServiceInterface,
    IndexingResult, IndexingServiceInterface, IndexingStats, IndexingStatus,
    SearchServiceInterface,
};
