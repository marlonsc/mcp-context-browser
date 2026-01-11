//! Synchronization system with cross-process coordination
//!
//! Provides batch-based synchronization to coordinate multiple MCP instances.

pub mod manager;

pub use manager::{SyncConfig, SyncManager, SyncStats};
