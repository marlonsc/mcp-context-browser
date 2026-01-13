//! Background daemon for automatic lock cleanup and monitoring
//!
//! Provides continuous monitoring and maintenance services:
//! - Automatic cleanup of stale sync batches
//! - Sync activity monitoring and reporting
//! - Background health checks

mod service;
pub mod types;

pub use service::ContextDaemon;
pub use types::{DaemonConfig, DaemonStats};
pub use types::{RecoveryConfig, RecoveryPolicy, RecoveryState, RecoveryStatus, RecoveryStrategy};
