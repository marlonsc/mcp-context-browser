//! Server DI Module Implementation
//!
//! Contains MCP server metrics and indexing operations.


use shaku::module;

use super::traits::ServerModule;
use crate::infrastructure::operations::McpIndexingOperations;
use crate::server::metrics::McpPerformanceMetrics;

// Implementation of the ServerModule trait providing server-side components.
// This module provides performance monitoring and indexing operations for the MCP server.
//
// Generated components:
// - `McpPerformanceMetrics`: Performance metrics collection and reporting for MCP operations
// - `McpIndexingOperations`: Tracking and management of ongoing indexing operations
module! {
    pub ServerModuleImpl: ServerModule {
        components = [McpPerformanceMetrics, McpIndexingOperations],
        providers = []
    }
}
