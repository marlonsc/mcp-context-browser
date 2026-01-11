//! Handler for the index_codebase MCP tool
//!
//! This handler is responsible for indexing codebases for semantic search.
//! It validates inputs, checks permissions, manages resources, and coordinates
//! the indexing process.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use std::time::Instant;

use crate::application::IndexingService;
use crate::infrastructure::auth::Permission;
use crate::infrastructure::limits::ResourceLimits;
use crate::server::args::IndexCodebaseArgs;
use crate::server::auth::AuthHandler;
use crate::server::formatter::ResponseFormatter;

/// Handler for codebase indexing operations
pub struct IndexCodebaseHandler {
    indexing_service: Arc<IndexingService>,
    auth_handler: Arc<AuthHandler>,
    resource_limits: Arc<ResourceLimits>,
}

impl IndexCodebaseHandler {
    /// Create a new index_codebase handler
    pub fn new(
        indexing_service: Arc<IndexingService>,
        auth_handler: Arc<AuthHandler>,
        resource_limits: Arc<ResourceLimits>,
    ) -> Self {
        Self {
            indexing_service,
            auth_handler,
            resource_limits,
        }
    }

    /// Handle the index_codebase tool request
    pub async fn handle(
        &self,
        Parameters(IndexCodebaseArgs { path, token, .. }): Parameters<IndexCodebaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let start_time = Instant::now();

        // Check authentication and permissions
        if let Err(e) = self
            .auth_handler
            .check_auth(token.as_ref(), &Permission::IndexCodebase)
        {
            return Ok(ResponseFormatter::format_auth_error(&e.to_string()));
        }

        // Check resource limits for indexing operation
        if let Err(e) = self
            .resource_limits
            .check_operation_allowed("indexing")
            .await
        {
            return Ok(ResponseFormatter::format_resource_limit_error(
                &e.to_string(),
            ));
        }

        // Acquire indexing permit
        let _permit = match self
            .resource_limits
            .acquire_operation_permit("indexing")
            .await
        {
            Ok(permit) => permit,
            Err(e) => {
                return Ok(ResponseFormatter::format_resource_limit_error(
                    &e.to_string(),
                ));
            }
        };

        // Validate input path
        let path = std::path::Path::new(&path);
        if !path.exists() {
            return Ok(ResponseFormatter::format_path_validation_error(
                "Specified path does not exist. Please provide a valid directory path.",
            ));
        }

        if !path.is_dir() {
            return Ok(ResponseFormatter::format_path_validation_error(
                "Specified path is not a directory. Please provide a directory containing source code.",
            ));
        }

        let collection = "default";
        tracing::info!("Starting codebase indexing for path: {}", path.display());

        // Add timeout for long-running indexing operations
        let indexing_future = self.indexing_service.index_directory(path, collection);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minute timeout
            indexing_future,
        )
        .await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(chunk_count)) => Ok(ResponseFormatter::format_indexing_success(
                chunk_count,
                path,
                duration,
            )),
            Ok(Err(e)) => Ok(ResponseFormatter::format_indexing_error(
                &e.to_string(),
                path,
            )),
            Err(_) => Ok(ResponseFormatter::format_indexing_timeout(path)),
        }
    }
}
