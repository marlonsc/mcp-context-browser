//! Provider adapters interfaces
//!
//! This module provides interfaces for external service providers.
//! Implementations are in mcb-providers crate.
//!
//! Following Clean Architecture: adapters implement domain interfaces.

// Re-export null implementations for Shaku DI
pub use null::{NullEmbeddingProvider, NullVectorStoreProvider};

// Include null implementations
mod null;