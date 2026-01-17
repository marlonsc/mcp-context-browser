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

/// Use case module trait - application business logic.
pub trait UseCaseModule:
    HasComponent<dyn mcb_application::domain_services::search::ContextServiceInterface>
    + HasComponent<dyn mcb_application::domain_services::search::SearchServiceInterface>
    + HasComponent<dyn mcb_application::domain_services::search::IndexingServiceInterface>
{
}

// ============================================================================
// Legacy Modules (Compatibility)
// ============================================================================

/// Adapters module trait - external service integrations.
pub trait AdaptersModule:
    HasComponent<dyn mcb_application::ports::providers::cache::CacheProvider>
    + HasComponent<dyn mcb_application::ports::providers::EmbeddingProvider>
    + HasComponent<dyn mcb_application::ports::providers::VectorStoreProvider>
    + HasComponent<dyn mcb_application::ports::providers::LanguageChunkingProvider>
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
