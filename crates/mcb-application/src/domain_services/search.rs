//! Application Service Interfaces
//!
//! Re-exports service interfaces from the ports module.
//! The canonical definitions are in `crate::ports::services`.

// Re-export all service interfaces from ports for backward compatibility
pub use crate::ports::services::{
    ChunkingOrchestratorInterface, ContextServiceInterface, IndexingResult,
    IndexingServiceInterface, IndexingStatus, SearchServiceInterface,
};
