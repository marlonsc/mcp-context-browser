//! Index Codebase Tool Handler
//!
//! Handles the index_codebase MCP tool call using the domain indexing service.

use crate::handlers::IndexCodebaseArgs;
use crate::McpServer;
use rmcp::model::CallToolRequest;
use std::path::Path;
use tracing::{info, instrument};

/// Handle the index_codebase tool call
#[instrument(skip(server), fields(path = %request.arguments["path"]))]
pub async fn handle_index_codebase(
    server: &McpServer,
    request: CallToolRequest,
) -> Result<rmcp::model::CallToolResponse, rmcp::Error> {
    let args: IndexCodebaseArgs = serde_json::from_value(request.arguments)
        .map_err(|e| rmcp::Error::invalid_params(format!("Invalid arguments: {}", e)))?;

    info!("Indexing codebase at path: {}", args.path);

    // Validate the path exists
    let path = Path::new(&args.path);
    if !path.exists() {
        return Err(rmcp::Error::invalid_params(format!("Path does not exist: {}", args.path)));
    }

    if !path.is_dir() {
        return Err(rmcp::Error::invalid_params(format!("Path is not a directory: {}", args.path)));
    }

    // Use the indexing service from the domain layer
    let result = server
        .indexing_service()
        .index_codebase(path, args.force, args.languages.as_deref())
        .await
        .map_err(|e| {
            rmcp::Error::internal_error(format!("Indexing failed: {}", e))
        })?;

    // Format result for MCP response
    let content = format_indexing_result(&result);

    Ok(rmcp::model::CallToolResponse {
        content: vec![rmcp::model::Content::text(content)],
        is_error: None,
    })
}

/// Format indexing result into a human-readable string
fn format_indexing_result(result: &mcb_domain::IndexingResult) -> String {
    let mut output = String::new();

    output.push_str(&format!("Indexing completed successfully!\n\n"));
    output.push_str(&format!("📁 Codebase: {}\n", result.codebase_path.display()));
    output.push_str(&format!("📊 Files processed: {}\n", result.files_processed));
    output.push_str(&format!("🔍 Chunks created: {}\n", result.chunks_created));
    output.push_str(&format!("⏱️  Duration: {:.2}s\n", result.duration.as_secs_f64()));

    if !result.errors.is_empty() {
        output.push_str(&format!("\n⚠️  Errors encountered: {}\n", result.errors.len()));
        for error in &result.errors {
            output.push_str(&format!("   - {}\n", error));
        }
    }

    output
}