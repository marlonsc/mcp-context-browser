//! Domain Port Interfaces
//!
//! Defines all boundary contracts between domain and external layers.
//! Ports are organized by their purpose and enable dependency injection
//! with clear separation of concerns.
//!
//! ## Architecture
//!
//! Ports define the contracts that external layers must implement.
//! This follows the Dependency Inversion Principle:
//! - High-level modules (domain) define interfaces
//! - Low-level modules (providers, infrastructure) implement them
//!
//! ## Organization
//!
//! - **admin** - Administrative interfaces for system management and monitoring
//! - **infrastructure/** - Infrastructure services (sync, snapshots, auth, events)
//! - **providers/** - External service provider ports (embeddings, vector stores, search)

/// Administrative interfaces for system management and monitoring
pub mod admin;
/// Infrastructure service ports
pub mod infrastructure;
/// External service provider ports
pub mod providers;

// Re-export commonly used port traits for convenience
pub use admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, LifecycleManaged, PerformanceMetricsData,
    PerformanceMetricsInterface, PortServiceState, ShutdownCoordinator,
};
pub use infrastructure::{
    AuthServiceInterface, DomainEventStream, EventBusProvider, LockGuard, LockProvider,
    ProviderContext, ProviderHealthStatus, ProviderRouter, SharedSyncCoordinator, SnapshotProvider,
    StateStoreProvider, SyncCoordinator, SyncOptions, SyncProvider, SyncResult, SystemMetrics,
    SystemMetricsCollectorInterface,
};
pub use providers::{
    CacheEntryConfig, CacheProvider, CacheProviderFactoryInterface, CacheStats, CryptoProvider,
    EmbeddingProvider, EncryptedData, HybridSearchProvider, HybridSearchResult,
    LanguageChunkingProvider, ProviderConfigManagerInterface, VectorStoreAdmin,
    VectorStoreProvider,
};
