//! MCP tool handlers
//!
//! This module contains the individual handlers for each MCP tool.
//! Each handler is responsible for a specific tool's logic and follows
//! the single responsibility principle.

pub mod index_codebase;
pub mod search_code;
pub mod get_indexing_status;
pub mod clear_index;