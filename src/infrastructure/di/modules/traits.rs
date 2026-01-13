//! Submodule Trait Interfaces
//!
//! These traits define the interfaces for domain-specific DI modules.
//! Concrete implementations can be swapped out for testing or different environments.
//!
//! Note: These traits only extend `HasComponent<T>`, not `Module`.
//! The `Module` trait is automatically implemented by the `module!` macro.

use shaku::HasComponent;

use crate::adapters::http_client::HttpClientProvider;
use crate::admin::service::AdminService;
use crate::infrastructure::di::factory::ServiceProviderInterface;
use crate::infrastructure::metrics::system::SystemMetricsCollectorInterface;
use crate::server::metrics::PerformanceMetricsInterface;
use crate::server::operations::IndexingOperationsInterface;

/// Adapters module trait - external adapters like HTTP clients
pub trait AdaptersModule: HasComponent<dyn HttpClientProvider> {}

/// Infrastructure module trait - core infrastructure services
pub trait InfrastructureModule:
    HasComponent<dyn SystemMetricsCollectorInterface> + HasComponent<dyn ServiceProviderInterface>
{
}

/// Server module trait - MCP server components
pub trait ServerModule:
    HasComponent<dyn PerformanceMetricsInterface> + HasComponent<dyn IndexingOperationsInterface>
{
}

/// Admin module trait - admin service with dependencies
pub trait AdminModule: HasComponent<dyn AdminService> {}
