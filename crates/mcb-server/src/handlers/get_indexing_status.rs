//! Get Indexing Status Tool Handler
//!
//! Handles the get_indexing_status MCP tool call using the domain indexing service.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use validator::Validate;

use mcb_application::domain_services::search::IndexingServiceInterface;

use crate::args::GetIndexingStatusArgs;
use crate::formatter::ResponseFormatter;

/// Handler for indexing status operations
pub struct GetIndexingStatusHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl GetIndexingStatusHandler {
    /// Create a new get_indexing_status handler
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    /// Handle the get_indexing_status tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<GetIndexingStatusArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let status = self.indexing_service.get_status();
        Ok(ResponseFormatter::format_indexing_status(&status))
    }
}
