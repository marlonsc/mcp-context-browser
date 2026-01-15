//! Get Indexing Status Tool Handler
//!
//! Handles the get_indexing_status MCP tool call using the domain indexing service.

use crate::handlers::GetIndexingStatusArgs;
use crate::McpServer;
use rmcp::model::CallToolRequest;
use tracing::instrument;

/// Handle the get_indexing_status tool call
#[instrument(skip(server))]
pub async fn handle_get_indexing_status(
    server: &McpServer,
    request: CallToolRequest,
) -> Result<rmcp::model::CallToolResponse, rmcp::Error> {
    let _args: GetIndexingStatusArgs = serde_json::from_value(request.arguments)
        .map_err(|e| rmcp::Error::invalid_params(format!("Invalid arguments: {}", e)))?;

    // Use the indexing service from the domain layer
    let status = server
        .indexing_service()
        .get_indexing_status()
        .await
        .map_err(|e| {
            rmcp::Error::internal_error(format!("Failed to get indexing status: {}", e))
        })?;

    // Format status for MCP response
    let content = format_indexing_status(&status);

    Ok(rmcp::model::CallToolResponse {
        content: vec![rmcp::model::Content::text(content)],
        is_error: None,
    })
}

/// Format indexing status into a human-readable string
fn format_indexing_status(status: &mcb_domain::IndexingStatus) -> String {
    let mut output = String::new();

    match status {
        mcb_domain::IndexingStatus::Idle => {
            output.push_str("📋 Indexing Status: Idle\n");
            output.push_str("No indexing operation is currently running.");
        }
        mcb_domain::IndexingStatus::Indexing { progress, message } => {
            output.push_str("🔄 Indexing Status: In Progress\n");
            output.push_str(&format!("Progress: {:.1}%\n", progress * 100.0));
            if let Some(msg) = message {
                output.push_str(&format!("Current: {}", msg));
            }
        }
        mcb_domain::IndexingStatus::Completed { result } => {
            output.push_str("✅ Indexing Status: Completed\n");
            output.push_str(&format!("Files processed: {}\n", result.files_processed));
            output.push_str(&format!("Chunks created: {}\n", result.chunks_created));
            output.push_str(&format!("Duration: {:.2}s", result.duration.as_secs_f64()));
        }
        mcb_domain::IndexingStatus::Failed { error } => {
            output.push_str("❌ Indexing Status: Failed\n");
            output.push_str(&format!("Error: {}", error));
        }
    }

    output
}