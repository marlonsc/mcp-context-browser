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

// Import null implementations from infrastructure (for Shaku DI defaults)
use crate::infrastructure::{NullIndexingOperations, NullPerformanceMetrics};

// Import traits
use super::traits::ServerModule;

module! {
    pub ServerModuleImpl: ServerModule {
        components = [
            // Null server components for testing defaults
            NullPerformanceMetrics,
            NullIndexingOperations
        ],
        providers = []
    }
}
