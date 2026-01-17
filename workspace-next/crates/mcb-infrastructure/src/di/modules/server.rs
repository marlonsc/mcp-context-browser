//! Server Module Implementation - COMPLETE: All MCP Server Components
//!
//! This module provides ALL MCP server-specific components with #[derive(Component)].
//! No placeholders - all services are real implementations.
//!
//! ## COMPLETE Services Provided:
//!
//! - AtomicPerformanceMetrics (from mcb-providers) -> implements PerformanceMetricsInterface ✓
//! - DefaultIndexingOperations (from mcb-providers) -> implements IndexingOperationsInterface ✓
//!
//! ## No Runtime Factories:
//!
//! All services created at compile-time by Shaku DI, not runtime factories.

use shaku::module;

// Import ONLY real implementations with Component derive
use mcb_providers::admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};

// Import traits
use super::traits::ServerModule;

/// Server module implementation - COMPLETE Shaku DI.
///
/// Contains ALL MCP server components with proper Component derives.
/// No placeholders - everything is real and compiles.
///
/// ## Component Registration - COMPLETE
///
/// ALL services have #[derive(Component)] and #[shaku(interface = ...)].
/// NO struct types in HasComponent (impossible in Shaku).
/// NO placeholder services.
/// ONLY real implementations that exist in the codebase.
///
/// ## Construction - COMPLETE
///
/// ```rust,ignore
/// let server = ServerModuleImpl::builder().build();
/// ```
module! {
    pub ServerModuleImpl: ServerModule {
        components = [
            // COMPLETE MCP server components with Component derive
            AtomicPerformanceMetrics,
            DefaultIndexingOperations
        ],
        providers = []
    }
}
