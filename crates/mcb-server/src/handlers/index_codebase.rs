//! Index Codebase Tool Handler
//!
//! Handles the index_codebase MCP tool call using the domain indexing service.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_application::domain_services::search::IndexingServiceInterface;

use crate::args::IndexCodebaseArgs;
use crate::formatter::ResponseFormatter;

/// Handler for codebase indexing operations
pub struct IndexCodebaseHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
}

impl IndexCodebaseHandler {
    /// Create a new index_codebase handler
    pub fn new(indexing_service: Arc<dyn IndexingServiceInterface>) -> Self {
        Self { indexing_service }
    }

    /// Handle the index_codebase tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexCodebaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let path = Path::new(&args.path);
        if !path.exists() {
            return Ok(ResponseFormatter::format_indexing_error(
                "Specified path does not exist",
                path,
            ));
        }

        if !path.is_dir() {
            return Ok(ResponseFormatter::format_indexing_error(
                "Specified path is not a directory",
                path,
            ));
        }

        let collection = args.collection.as_deref().unwrap_or("default");
        let timer = Instant::now();

        match self.indexing_service.index_codebase(path, collection).await {
            Ok(result) => Ok(ResponseFormatter::format_indexing_success(
                &result,
                path,
                timer.elapsed(),
            )),
            Err(e) => Ok(ResponseFormatter::format_indexing_error(
                &e.to_string(),
                path,
            )),
        }
    }
}
