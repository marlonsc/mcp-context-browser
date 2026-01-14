//! Indexing operations tracking
//!
//! Provides implementations for tracking ongoing indexing operations.
//! The trait and types are defined in `domain::ports::admin` to maintain
//! Clean Architecture boundaries.

use dashmap::DashMap;

// Re-export from domain for convenience
pub use crate::domain::ports::admin::{IndexingOperation, IndexingOperationsInterface};

/// Concrete implementation of indexing operations tracking
#[derive(Debug, Default, shaku::Component)]
#[shaku(interface = IndexingOperationsInterface)]
pub struct McpIndexingOperations {
    /// Thread-safe map storing active indexing operations by ID
    #[shaku(default)]
    pub map: DashMap<String, IndexingOperation>,
}

impl IndexingOperationsInterface for McpIndexingOperations {
    /// Get access to the internal operations map
    fn get_map(&self) -> &DashMap<String, IndexingOperation> {
        &self.map
    }
}
