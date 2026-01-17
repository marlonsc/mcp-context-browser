//! Admin Module Implementation
//!
//! Administrative services are provided by the ServerModule (PerformanceMetricsInterface,
//! IndexingOperationsInterface) which is the correct location for MCP server admin components.
//!
//! This module exists for future admin-specific services like shutdown coordination
//! that don't belong in the server module.

use shaku::module;

// Import traits
use super::traits::AdminModule;

module! {
    pub AdminModuleImpl: AdminModule {
        components = [],
        providers = []
    }
}
