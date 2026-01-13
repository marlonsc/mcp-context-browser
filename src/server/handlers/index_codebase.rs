//! Handler for the index_codebase MCP tool
//!
//! This handler is responsible for indexing codebases for semantic search.
//! It validates inputs, checks permissions, manages resources, and coordinates
//! the indexing process.

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::ports::IndexingServiceInterface;
use crate::infrastructure::auth::Permission;
use crate::infrastructure::constants::INDEXING_OPERATION_TIMEOUT;
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::service_helpers::TimedOperation;
use crate::server::args::IndexCodebaseArgs;
use crate::server::auth::AuthHandler;
use crate::server::formatter::ResponseFormatter;

/// Handler for codebase indexing operations
pub struct IndexCodebaseHandler {
    indexing_service: Arc<dyn IndexingServiceInterface>,
    auth_handler: Arc<AuthHandler>,
    resource_limits: Arc<ResourceLimits>,
}

impl IndexCodebaseHandler {
    /// Create a new index_codebase handler
    pub fn new(
        indexing_service: Arc<dyn IndexingServiceInterface>,
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
        Parameters(IndexCodebaseArgs {
            path,
            token,
            collection,
            ..
        }): Parameters<IndexCodebaseArgs>,
    ) -> Result<CallToolResult, McpError> {
        let timer = TimedOperation::start();

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

        // Use collection from args, or default to "default"
        let collection = collection.as_deref().unwrap_or("default");
        tracing::info!("Starting codebase indexing for path: {}", path.display());

        // Add timeout for long-running indexing operations
        let indexing_future = self.indexing_service.index_codebase(path, collection);
        let result = tokio::time::timeout(INDEXING_OPERATION_TIMEOUT, indexing_future).await;

        let duration = Duration::from_millis(timer.elapsed_ms());

        match result {
            Ok(Ok(indexing_result)) => Ok(ResponseFormatter::format_indexing_success(
                indexing_result.chunks_created,
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
