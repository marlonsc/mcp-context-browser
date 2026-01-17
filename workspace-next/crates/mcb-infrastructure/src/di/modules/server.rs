//! Server Module Implementation - Null Components for Shaku DI
//!
//! This module provides null server components for Shaku DI modules.
//! Real implementations are created at runtime via factories.
//!
//! ## Services Provided:
//!
//! - NullPerformanceMetrics -> implements PerformanceMetricsInterface ✓
//! - NullIndexingOperations -> implements IndexingOperationsInterface ✓
//!
//! ## Note on Production:
//!
//! Real providers are created at runtime via factories, not through Shaku.

use shaku::module;

// Import implementations from mcb-providers
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
            // Server components from mcb-providers
            AtomicPerformanceMetrics,
            DefaultIndexingOperations
        ],
        providers = []
    }
}
