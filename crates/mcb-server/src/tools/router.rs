//! Tool Router Module
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParam, CallToolResult};
use std::sync::Arc;

use crate::args::{ClearIndexArgs, GetIndexingStatusArgs, IndexCodebaseArgs, SearchCodeArgs};
use crate::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
};

/// Handler references for tool routing
pub struct ToolHandlers {
    /// Handler for codebase indexing operations
    pub index_codebase: Arc<IndexCodebaseHandler>,
    /// Handler for code search operations
    pub search_code: Arc<SearchCodeHandler>,
    /// Handler for indexing status operations
    pub get_indexing_status: Arc<GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    pub clear_index: Arc<ClearIndexHandler>,
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
pub async fn route_tool_call(
    request: CallToolRequestParam,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "index_codebase" => {
            let args = parse_args::<IndexCodebaseArgs>(&request)?;
            handlers.index_codebase.handle(Parameters(args)).await
        }
        "search_code" => {
            let args = parse_args::<SearchCodeArgs>(&request)?;
            handlers.search_code.handle(Parameters(args)).await
        }
        "get_indexing_status" => {
            let args = parse_args::<GetIndexingStatusArgs>(&request)?;
            handlers.get_indexing_status.handle(Parameters(args)).await
        }
        "clear_index" => {
            let args = parse_args::<ClearIndexArgs>(&request)?;
            handlers.clear_index.handle(Parameters(args)).await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }
}

/// Parse request arguments into the expected type
fn parse_args<T: serde::de::DeserializeOwned>(
    request: &CallToolRequestParam,
) -> Result<T, McpError> {
    let args_value = serde_json::Value::Object(request.arguments.clone().unwrap_or_default());
    serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))
}
