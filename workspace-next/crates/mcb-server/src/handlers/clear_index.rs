//! Clear Index Tool Handler
//!
//! Handles the clear_index MCP tool call using the domain indexing service.

use crate::handlers::ClearIndexArgs;
use crate::McpServer;
use rmcp::model::CallToolRequest;
use tracing::{info, instrument, warn};

/// Handle the clear_index tool call
#[instrument(skip(server))]
pub async fn handle_clear_index(
    server: &McpServer,
    request: CallToolRequest,
) -> Result<rmcp::model::CallToolResponse, rmcp::Error> {
    let args: ClearIndexArgs = serde_json::from_value(request.arguments)
        .map_err(|e| rmcp::Error::invalid_params(format!("Invalid arguments: {}", e)))?;

    if !args.confirm {
        warn!("Clear index operation not confirmed");
        return Err(rmcp::Error::invalid_params(
            "Operation must be confirmed by setting confirm=true"
        ));
    }

    info!("Clearing search index");

    // Use the indexing service from the domain layer
    server
        .indexing_service()
        .clear_index()
        .await
        .map_err(|e| {
            rmcp::Error::internal_error(format!("Failed to clear index: {}", e))
        })?;

    let content = "✅ Search index cleared successfully.".to_string();

    Ok(rmcp::model::CallToolResponse {
        content: vec![rmcp::model::Content::text(content)],
        is_error: None,
    })
}