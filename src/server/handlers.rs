//! MCP tool handlers
//!
//! This module contains the individual handlers for each MCP tool.
//! Each handler is responsible for a specific tool's logic and follows
//! the single responsibility principle.

pub mod clear_index;
pub mod get_indexing_status;
pub mod index_codebase;
pub mod search_code;

// Re-export handlers for convenience
pub use clear_index::ClearIndexHandler;
pub use get_indexing_status::GetIndexingStatusHandler;
pub use index_codebase::IndexCodebaseHandler;
pub use search_code::SearchCodeHandler;
