//! Tool Registry Module
//!
//! Manages tool definitions and schema generation for the MCP protocol.
//! This module centralizes all tool metadata to enable consistent tool listing.

use rmcp::ErrorData as McpError;
use rmcp::model::Tool;
use std::borrow::Cow;
use std::sync::Arc;

use crate::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};

/// Tool definitions for MCP protocol
pub struct ToolDefinitions;

impl ToolDefinitions {
    /// Get the index_codebase tool definition
    pub fn index_codebase() -> Result<Tool, McpError> {
        Self::create_tool(
            "index_codebase",
            "Index a codebase directory for semantic search using vector embeddings",
            schemars::schema_for!(IndexCodebaseArgs),
        )
    }

    /// Get the search_code tool definition
    pub fn search_code() -> Result<Tool, McpError> {
        Self::create_tool(
            "search_code",
            "Search for code using natural language queries",
            schemars::schema_for!(SearchCodeArgs),
        )
    }

    /// Get the get_indexing_status tool definition
    pub fn get_indexing_status() -> Result<Tool, McpError> {
        Self::create_tool(
            "get_indexing_status",
            "Get the current indexing status and statistics",
            schemars::schema_for!(GetIndexingStatusArgs),
        )
    }

    /// Get the clear_index tool definition
    pub fn clear_index() -> Result<Tool, McpError> {
        Self::create_tool(
            "clear_index",
            "Clear the search index for a collection",
            schemars::schema_for!(ClearIndexArgs),
        )
    }

    /// Create a tool from schema
    fn create_tool(
        name: &'static str,
        description: &'static str,
        schema: schemars::Schema,
    ) -> Result<Tool, McpError> {
        let schema_value = serde_json::to_value(schema)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let input_schema = schema_value
            .as_object()
            .ok_or_else(|| {
                McpError::internal_error(format!("Schema for {} is not an object", name), None)
            })?
            .clone();

        Ok(Tool {
            name: Cow::Borrowed(name),
            title: None,
            description: Some(Cow::Borrowed(description)),
            input_schema: Arc::new(input_schema),
            output_schema: None,
            annotations: None,
            icons: None,
            meta: Default::default(),
        })
    }
}

/// Create the complete list of available tools
///
/// Returns all tool definitions for the MCP list_tools response.
pub fn create_tool_list() -> Result<Vec<Tool>, McpError> {
    Ok(vec![
        ToolDefinitions::index_codebase()?,
        ToolDefinitions::search_code()?,
        ToolDefinitions::get_indexing_status()?,
        ToolDefinitions::clear_index()?,
    ])
}
