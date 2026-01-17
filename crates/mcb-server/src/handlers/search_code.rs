//! Search Code Tool Handler
//!
//! Handles the search_code MCP tool call using the domain search service.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_application::domain_services::search::SearchServiceInterface;

use crate::args::SearchCodeArgs;
use crate::formatter::ResponseFormatter;

/// Handler for code search operations
pub struct SearchCodeHandler {
    search_service: Arc<dyn SearchServiceInterface>,
}

impl SearchCodeHandler {
    /// Create a new search_code handler
    pub fn new(search_service: Arc<dyn SearchServiceInterface>) -> Self {
        Self { search_service }
    }

    /// Handle the search_code tool request
    pub async fn handle(
        &self,
        Parameters(mut args): Parameters<SearchCodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.query = args.query.trim().to_string();
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let collection = args.collection.as_deref().unwrap_or("default");
        let timer = Instant::now();

        let results = self
            .search_service
            .search(collection, &args.query, args.limit)
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        ResponseFormatter::format_search_response(
            &args.query,
            &results,
            timer.elapsed(),
            args.limit,
        )
    }
}
