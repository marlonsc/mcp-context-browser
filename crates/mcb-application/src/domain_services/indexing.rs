//! Indexing Domain Service Interface
//!
//! Re-exports batch indexing service interface from the ports module.
//! The canonical definitions are in `crate::ports::services`.

// Re-export batch indexing interfaces from ports for backward compatibility
pub use crate::ports::services::{BatchIndexingServiceInterface, IndexingStats};
