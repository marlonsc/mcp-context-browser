//! MCP Tool Handlers
//!
//! Implementations of MCP tool calls using domain services.
//! Each handler translates MCP protocol requests into domain service calls.

pub mod search_code;
pub mod index_codebase;
pub mod get_indexing_status;
pub mod clear_index;

// Re-export handler functions for use by the server
pub use search_code::handle_search_code;
pub use index_codebase::handle_index_codebase;
pub use get_indexing_status::handle_get_indexing_status;
pub use clear_index::handle_clear_index;

// Argument types for MCP tools
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for the search_code tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchCodeArgs {
    /// The search query
    pub query: String,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Optional file path filter
    pub file_path: Option<String>,
    /// Optional programming language filter
    pub language: Option<String>,
}

fn default_limit() -> usize {
    10
}

/// Arguments for the index_codebase tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct IndexCodebaseArgs {
    /// Path to the codebase to index
    pub path: String,
    /// Force re-indexing even if already indexed
    #[serde(default)]
    pub force: bool,
    /// Programming languages to include
    pub languages: Option<Vec<String>>,
}

/// Arguments for the get_indexing_status tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetIndexingStatusArgs {}

/// Arguments for the clear_index tool
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ClearIndexArgs {
    /// Confirm the operation
    pub confirm: bool,
}