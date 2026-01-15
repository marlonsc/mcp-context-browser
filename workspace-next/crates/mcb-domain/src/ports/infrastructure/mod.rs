//! Infrastructure Ports
//!
//! Ports for infrastructure services that provide technical capabilities
//! to the domain. These interfaces define contracts for file synchronization,
//! snapshot management, and other cross-cutting infrastructure concerns.
//!
//! ## Infrastructure Ports
//!
//! | Port | Description |
//! |------|-------------|
//! | [`SyncProvider`] | File system synchronization services |
//! | [`SnapshotProvider`] | Codebase snapshot management |

/// File synchronization infrastructure port
pub mod sync;
/// Snapshot management infrastructure port
pub mod snapshot;

// Re-export infrastructure ports
pub use snapshot::SyncProvider;
pub use snapshot::SnapshotProvider;