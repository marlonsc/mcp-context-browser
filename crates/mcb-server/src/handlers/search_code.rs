//! Search Code Tool Handler
//!
//! Handles the search_code MCP tool call using the domain search service.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_application::domain_services::search::SearchServiceInterface;

use crate::args::SearchCodeArgs;
use crate::collection_mapping::map_collection_name;
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

        let collection_name = args.collection.as_deref().unwrap_or("default");

        // Map user-friendly name to Milvus-compatible name
        let milvus_collection = match map_collection_name(collection_name) {
            Ok(name) => name,
            Err(e) => {
                return Err(McpError::internal_error(
                    format!("Failed to map collection name: {}", e),
                    None,
                ));
            }
        };

        let timer = Instant::now();

        let results = self
            .search_service
            .search(&milvus_collection, &args.query, args.limit)
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
