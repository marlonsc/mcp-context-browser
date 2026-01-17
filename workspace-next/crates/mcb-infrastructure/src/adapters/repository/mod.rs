//! Repository adapters null implementations
//!
//! This module provides null implementations for repository interfaces.
//! Interfaces are defined in mcb-domain/repositories.
//!
//! ## Architecture
//! - Interfaces defined in mcb-domain/repositories
//! - Null implementations for testing/Shaku DI defaults
//! - Real implementations injected at runtime

// Re-export null implementations for Shaku DI
pub use null::{NullChunkRepository, NullSearchRepository};

// Include null implementations
mod null;