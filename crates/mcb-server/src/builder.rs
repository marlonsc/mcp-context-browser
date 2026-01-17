//! MCP Server Builder
//!
//! Builder pattern for constructing MCP servers with dependency injection.
//! Ensures all required dependencies are provided before server construction.

use crate::McpServer;
use mcb_application::{ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface};
use std::sync::Arc;

/// Builder for MCP Server with dependency injection
///
/// Ensures all required domain services are provided before server construction.
/// Follows the builder pattern to make server construction explicit and testable.
#[derive(Default)]
pub struct McpServerBuilder {
    indexing_service: Option<Arc<dyn IndexingServiceInterface>>,
    context_service: Option<Arc<dyn ContextServiceInterface>>,
    search_service: Option<Arc<dyn SearchServiceInterface>>,
}

impl McpServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the indexing service
    ///
    /// # Arguments
    /// * `service` - Implementation of the indexing service port
    pub fn with_indexing_service(mut self, service: Arc<dyn IndexingServiceInterface>) -> Self {
        self.indexing_service = Some(service);
        self
    }

    /// Set the context service
    ///
    /// # Arguments
    /// * `service` - Implementation of the context service port
    pub fn with_context_service(mut self, service: Arc<dyn ContextServiceInterface>) -> Self {
        self.context_service = Some(service);
        self
    }

    /// Set the search service
    ///
    /// # Arguments
    /// * `service` - Implementation of the search service port
    pub fn with_search_service(mut self, service: Arc<dyn SearchServiceInterface>) -> Self {
        self.search_service = Some(service);
        self
    }

    /// Build the MCP server
    ///
    /// # Returns
    /// A Result containing the McpServer or an error if dependencies are missing
    ///
    /// # Errors
    /// Returns `BuilderError::MissingDependency` if any required service is not provided
    pub fn build(self) -> Result<McpServer, BuilderError> {
        self.try_build()
    }

    /// Try to build the MCP server (alias for `build`)
    ///
    /// This method is kept for API compatibility.
    ///
    /// # Returns
    /// A Result containing the McpServer or an error if dependencies are missing
    pub fn try_build(self) -> Result<McpServer, BuilderError> {
        let indexing_service = self
            .indexing_service
            .ok_or(BuilderError::MissingDependency("indexing service"))?;
        let context_service = self
            .context_service
            .ok_or(BuilderError::MissingDependency("context service"))?;
        let search_service = self
            .search_service
            .ok_or(BuilderError::MissingDependency("search service"))?;

        Ok(McpServer::new(
            indexing_service,
            context_service,
            search_service,
        ))
    }
}

/// Errors that can occur during server building
#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    /// A required dependency was not provided
    #[error("Missing required dependency: {0}")]
    MissingDependency(&'static str),
}
