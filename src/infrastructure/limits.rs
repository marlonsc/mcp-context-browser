//! Resource limits and quotas for production safety
//!
//! Implements resource monitoring and limits to prevent system overload.
//! Supports memory, CPU, disk, and concurrent operation limits.

mod config;
mod enforcer;
mod types;

// Re-export configuration types
pub use config::{CpuLimits, DiskLimits, MemoryLimits, OperationLimits, ResourceLimitsConfig};

// Re-export implementations
pub use enforcer::{
    NullResourceLimits, OperationPermit, ResourceLimits, ResourceLimitsProvider,
};

// Re-export types
pub use types::{
    CpuStats, DiskStats, MemoryStats, OperationStats, ResourceStats, ResourceViolation,
};

use std::sync::Arc;

/// Type alias for shared resource limits provider
pub type SharedResourceLimits = Arc<dyn ResourceLimitsProvider>;
