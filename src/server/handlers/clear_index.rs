//! Handler for the clear_index MCP tool
//!
//! This handler is responsible for clearing indexed data collections.
//! It validates inputs, prevents accidental clearing of critical collections,
//! and performs the actual clearing operation through the IndexingService.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use std::time::Instant;

use crate::application::IndexingService;
use crate::server::args::ClearIndexArgs;
use crate::server::formatter::ResponseFormatter;

/// Handler for index clearing operations
pub struct ClearIndexHandler {
    indexing_service: Arc<IndexingService>,
}

impl ClearIndexHandler {
    /// Create a new clear_index handler with required dependencies
    pub fn new(indexing_service: Arc<IndexingService>) -> Self {
        Self { indexing_service }
    }

    /// Handle the clear_index tool request
    pub async fn handle(
        &self,
        Parameters(ClearIndexArgs { collection }): Parameters<ClearIndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        // Validate collection name
        if collection.trim().is_empty() {
            return Ok(ResponseFormatter::format_query_validation_error(
                "Collection name cannot be empty. Please specify a valid collection name.",
            ));
        }

        // Prevent clearing critical collections accidentally
        if collection == "system" || collection == "admin" {
            return Ok(ResponseFormatter::format_query_validation_error(
                "Cannot clear system collections. These are reserved for internal use.",
            ));
        }

        tracing::warn!("Index clearing requested for collection: {}", collection);

        // Perform the actual clearing operation
        match self.indexing_service.clear_collection(&collection).await {
            Ok(()) => {
                let duration = start_time.elapsed();
                let message = format_clear_success(&collection, duration);
                tracing::info!(
                    "Index clearing completed successfully for collection: {} in {:?}",
                    collection,
                    duration
                );
                Ok(rmcp::model::CallToolResult::success(vec![
                    rmcp::model::Content::text(message),
                ]))
            }
            Err(e) => {
                let message = format_clear_error(&collection, &e.to_string());
                tracing::error!(
                    "Index clearing failed for collection: {} - {}",
                    collection,
                    e
                );
                Ok(rmcp::model::CallToolResult::success(vec![
                    rmcp::model::Content::text(message),
                ]))
            }
        }
    }
}

fn format_clear_success(collection: &str, duration: std::time::Duration) -> String {
    let mut message = "🗑️ **Index Clearing Completed Successfully**\n\n".to_string();

    message.push_str(&format!(
        "✅ Collection **`{}`** has been cleared.\n\n",
        collection
    ));

    message.push_str(&format!("**Duration:** {:?}\n\n", duration));

    message.push_str("**What was removed:**\n");
    message.push_str("• All indexed code chunks\n");
    message.push_str("• Vector embeddings\n");
    message.push_str("• Cached search results\n");
    message.push_str("• Collection metadata\n\n");

    message.push_str("**Next Steps:**\n");
    message.push_str("1. Run `index_codebase` with your source directory to rebuild the index\n");
    message.push_str("2. Wait for indexing to complete\n");
    message.push_str("3. Use `get_indexing_status` to verify the new index state\n");

    message
}

fn format_clear_error(collection: &str, error: &str) -> String {
    let mut message = "❌ **Index Clearing Failed**\n\n".to_string();

    message.push_str(&format!(
        "Failed to clear collection **`{}`**.\n\n",
        collection
    ));

    message.push_str(&format!("**Error:** {}\n\n", error));

    message.push_str("**Troubleshooting:**\n");
    message.push_str("• Verify the collection name is correct\n");
    message.push_str("• Check that you have appropriate permissions\n");
    message.push_str("• Ensure the vector store backend is accessible\n");
    message.push_str("• Review server logs for detailed error information\n");

    message
}
