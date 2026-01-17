//! Indexing Operations Tracking Implementation
//!
//! Thread-safe tracking of ongoing indexing operations.
//! Implements the `IndexingOperationsInterface` port from mcb-domain.

use dashmap::DashMap;
use mcb_domain::ports::admin::{IndexingOperation, IndexingOperationsInterface};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Default indexing operations tracker
///
/// Thread-safe implementation using DashMap for concurrent access.
///
/// **Note**: This type can be used in Shaku DI modules.
#[derive(shaku::Component)]
#[shaku(interface = mcb_domain::ports::admin::IndexingOperationsInterface)]
pub struct DefaultIndexingOperations {
    /// Active indexing operations by ID
    operations: Arc<DashMap<String, IndexingOperation>>,
}

impl DefaultIndexingOperations {
    /// Create a new indexing operations tracker
    pub fn new() -> Self {
        Self {
            operations: Arc::new(DashMap::new()),
        }
    }

    /// Create as Arc for sharing
    pub fn new_shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Start tracking a new indexing operation
    pub fn start_operation(&self, collection: &str, total_files: usize) -> String {
        let id = Uuid::new_v4().to_string();
        let operation = IndexingOperation {
            id: id.clone(),
            collection: collection.to_string(),
            current_file: None,
            total_files,
            processed_files: 0,
            start_timestamp: current_timestamp(),
        };
        self.operations.insert(id.clone(), operation);
        id
    }

    /// Update progress for an operation
    pub fn update_progress(
        &self,
        operation_id: &str,
        current_file: Option<String>,
        processed: usize,
    ) {
        if let Some(mut op) = self.operations.get_mut(operation_id) {
            op.current_file = current_file;
            op.processed_files = processed;
        }
    }

    /// Complete and remove an operation
    pub fn complete_operation(&self, operation_id: &str) {
        self.operations.remove(operation_id);
    }

    /// Check if any operations are in progress
    pub fn has_active_operations(&self) -> bool {
        !self.operations.is_empty()
    }

    /// Get count of active operations
    pub fn active_count(&self) -> usize {
        self.operations.len()
    }
}

impl Default for DefaultIndexingOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexingOperationsInterface for DefaultIndexingOperations {
    fn get_operations(&self) -> HashMap<String, IndexingOperation> {
        self.operations
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
