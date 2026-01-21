//! Admin Service Domain Ports
//!
//! Re-exports admin port interfaces from mcb-domain for backward compatibility.
//!
//! **Note**: These types are defined in `mcb_domain::ports::admin`. This module
//! provides re-exports for code that historically imported from mcb-application.

// Re-export all admin port types from mcb-domain
pub use mcb_domain::ports::admin::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, IndexingOperation,
    IndexingOperationsInterface, LifecycleManaged, PerformanceMetricsData,
    PerformanceMetricsInterface, PortServiceState, ShutdownCoordinator,
};
