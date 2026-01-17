//! Module Trait Interfaces - Shaku Strict Pattern
//!
//! These traits define the interfaces for internal infrastructure DI modules.
//! External providers (embedding, vector_store, cache, language) are resolved
//! via the registry system in `di/resolver.rs`.
//!
//! ## Note on Provider Resolution
//!
//! External providers are resolved dynamically via `resolve_providers()`.
//! Only internal infrastructure services use Shaku DI.

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
// Admin Module (Administrative Services)
// ============================================================================

/// Admin module trait - administrative services.
///
/// Note: Server admin components (PerformanceMetricsInterface, IndexingOperationsInterface)
/// are registered in ServerModule. This is a marker trait for future admin-specific
/// services like shutdown coordination.
pub trait AdminModule: Send + Sync {}
