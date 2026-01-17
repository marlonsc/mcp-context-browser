//! Module Trait Interfaces - Shaku Strict Pattern
//!
//! These traits define the interfaces for domain-specific DI modules.
//! Each trait represents a bounded context in the Clean Architecture.
//!
//! ## Note on Runtime Services
//!
//! Many services are created at runtime via DomainServicesFactory rather than
//! through Shaku DI, because they require runtime configuration.

use shaku::HasComponent;

// ============================================================================
// Infrastructure Module (Core Services)
// ============================================================================

/// Infrastructure module trait - core infrastructure services.
///
/// Provides fundamental infrastructure services with Shaku Component implementation.
pub trait InfrastructureModule:
    HasComponent<dyn mcb_application::ports::infrastructure::AuthServiceInterface>
    + HasComponent<dyn mcb_application::ports::infrastructure::EventBusProvider>
    + HasComponent<dyn mcb_application::ports::infrastructure::SystemMetricsCollectorInterface>
    + HasComponent<dyn mcb_application::ports::infrastructure::SnapshotProvider>
    + HasComponent<dyn mcb_application::ports::infrastructure::SyncProvider>
    + HasComponent<dyn mcb_application::ports::admin::ShutdownCoordinator>
{
}

// ============================================================================
// Server Module (MCP Server Components)
// ============================================================================

/// Server module trait - MCP server-specific components.
///
/// Provides MCP server services with Shaku Component implementation.
pub trait ServerModule:
    HasComponent<dyn mcb_application::ports::admin::PerformanceMetricsInterface>
    + HasComponent<dyn mcb_application::ports::admin::IndexingOperationsInterface>
{
}

// ============================================================================
// Context Modules (Clean Architecture Pattern)
// ============================================================================

/// Cache module trait - caching services.
pub trait CacheModule:
    HasComponent<dyn mcb_application::ports::providers::cache::CacheProvider>
{
}

/// Embedding module trait - text embedding services.
pub trait EmbeddingModule:
    HasComponent<dyn mcb_application::ports::providers::EmbeddingProvider>
{
}

/// Data module trait - data persistence services.
pub trait DataModule:
    HasComponent<dyn mcb_application::ports::providers::VectorStoreProvider>
{
}

/// Language module trait - code processing services.
pub trait LanguageModule:
    HasComponent<dyn mcb_application::ports::providers::LanguageChunkingProvider>
{
}

/// Routing module trait - provider routing and selection services.
pub trait RoutingModule:
    HasComponent<dyn mcb_application::ports::infrastructure::ProviderRouter>
{
}

// ============================================================================
// Admin Module (Administrative Services)
// ============================================================================

/// Admin module trait - administrative services.
///
/// Note: Server admin components (PerformanceMetricsInterface, IndexingOperationsInterface)
/// are registered in ServerModule. This is a marker trait for future admin-specific
/// services like shutdown coordination.
pub trait AdminModule: Send + Sync {}
