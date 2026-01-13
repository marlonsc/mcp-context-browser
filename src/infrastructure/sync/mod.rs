//! Synchronization system with cross-process coordination
//!
//! Provides batch-based synchronization to coordinate multiple MCP instances.
//!
//! ## Services
//!
//! - `SyncManager` - Main sync orchestration (backward compatible)
//! - `DebounceService` - Manages sync timing and rate limiting
//! - `SyncStatsCollector` - Tracks sync operation metrics

pub mod debounce;
pub mod manager;
pub mod stats;

pub use debounce::{DebounceConfig, DebounceService};
pub use manager::{SyncConfig, SyncManager};
pub use stats::{SyncStats, SyncStatsCollector};
