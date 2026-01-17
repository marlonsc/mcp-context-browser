//! Admin Provider Implementations
//!
//! Implementations of admin domain ports for performance metrics tracking,
//! indexing operations monitoring, and shutdown coordination.
//!
//! These are provider implementations that implement traits defined in `mcb-domain/ports/admin.rs`.

pub mod indexing;
pub mod metrics;
pub mod shutdown;

pub use indexing::DefaultIndexingOperations;
pub use metrics::AtomicPerformanceMetrics;
pub use shutdown::DefaultShutdownCoordinator;
