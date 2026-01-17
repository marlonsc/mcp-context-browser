//! Admin Adapter Implementations
//!
//! Infrastructure implementations for admin domain ports including
//! performance metrics tracking, indexing operations monitoring,
//! and shutdown coordination.

pub mod indexing;
pub mod metrics;
pub mod shutdown;

pub use indexing::DefaultIndexingOperations;
pub use metrics::AtomicPerformanceMetrics;
pub use shutdown::DefaultShutdownCoordinator;
