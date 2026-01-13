//! Indexing module - Focused services for codebase indexing
//!
//! This module provides focused, single-responsibility services for indexing:
//!
//! - `IndexingService` - Main indexing service (backward compatible)
//! - `FileDiscoveryService` - Discovers files eligible for indexing
//! - `ChunkingOrchestrator` - Coordinates batch code chunking
//!
//! These services follow SOLID principles and can be composed for different
//! indexing workflows.

mod chunking_orchestrator;
mod file_discovery;
mod service;

pub use chunking_orchestrator::{
    BatchChunkResult, ChunkingConfig, ChunkingOrchestrator, FileChunkResult,
};
pub use file_discovery::{DiscoveryOptions, DiscoveryResult, FileDiscoveryService};
pub use service::IndexingService;
