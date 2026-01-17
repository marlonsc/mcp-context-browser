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
    HasComponent<dyn mcb_domain::ports::infrastructure::AuthServiceInterface>
    + HasComponent<dyn mcb_domain::ports::infrastructure::EventBusProvider>
    + HasComponent<dyn mcb_domain::ports::infrastructure::SystemMetricsCollectorInterface>
    + HasComponent<dyn mcb_domain::ports::infrastructure::SnapshotProvider>
    + HasComponent<dyn mcb_domain::ports::infrastructure::SyncProvider>
{}

// ============================================================================
// Server Module (MCP Server Components)
// ============================================================================

/// Server module trait - MCP server-specific components.
///
/// Provides MCP server services with Shaku Component implementation.
pub trait ServerModule:
    HasComponent<dyn mcb_domain::ports::admin::PerformanceMetricsInterface>
    + HasComponent<dyn mcb_domain::ports::admin::IndexingOperationsInterface>
{}

// ============================================================================
// Adapters Module (External Integrations)
// ============================================================================

/// Adapters module trait - external service integrations.
pub trait AdaptersModule:
    HasComponent<dyn mcb_domain::ports::providers::EmbeddingProvider>
    + HasComponent<dyn mcb_domain::ports::providers::VectorStoreProvider>
{
}

// ============================================================================
// Application Module (Business Logic)
// ============================================================================

/// Application module trait - business logic services.
/// Services created via DomainServicesFactory at runtime.
pub trait ApplicationModule: Send + Sync {}

// ============================================================================
// Admin Module (Administrative Services)
// ============================================================================

/// Admin module trait - administrative services.
pub trait AdminModule: Send + Sync {}
