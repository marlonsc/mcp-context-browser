//! Clear Index Tool Handler
//!
//! Handles the clear_index MCP tool call using the domain indexing service.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::sync::Arc;
use validator::Validate;

use mcb_application::domain_services::search::IndexingServiceInterface;

use crate::args::ClearIndexArgs;
use crate::formatter::ResponseFormatter;

/// Handler for index clearing operations
pub struct ClearIndexHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl ClearIndexHandler {
    /// Create a new clear_index handler
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    /// Handle the clear_index tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ClearIndexArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        self.indexing_service
            .clear_collection(&args.collection)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to clear index: {}", e), None))?;

        Ok(ResponseFormatter::format_clear_index(&args.collection))
    }
}
