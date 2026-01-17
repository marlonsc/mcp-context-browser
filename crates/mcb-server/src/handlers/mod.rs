//! MCP Tool Handlers
//!
//! Implementations of MCP tool calls using domain services.
//! Each handler translates MCP protocol requests into domain service calls.

pub mod clear_index;
pub mod get_indexing_status;
pub mod index_codebase;
pub mod search_code;

// Re-export handlers for convenience
pub use clear_index::ClearIndexHandler;
pub use get_indexing_status::GetIndexingStatusHandler;
pub use index_codebase::IndexCodebaseHandler;
pub use search_code::SearchCodeHandler;
