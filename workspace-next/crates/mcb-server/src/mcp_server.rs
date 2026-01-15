//! MCP Server Implementation
//!
//! Core MCP protocol server that orchestrates semantic code search operations.
//! Follows Clean Architecture principles with dependency injection.

use std::sync::Arc;
use async_trait::async_trait;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, Service, Tool};
use tracing::{info, error, instrument};

use mcb_domain::ports::{
    IndexingServiceInterface, ContextServiceInterface, SearchServiceInterface,
};

/// Core MCP server implementation
///
/// This server implements the MCP protocol for semantic code search.
/// It depends only on domain ports and receives all dependencies through
/// constructor injection following Clean Architecture principles.
#[derive(Clone)]
pub struct McpServer {
    /// Service for indexing codebases
    indexing_service: Arc<dyn IndexingServiceInterface>,
    /// Service for providing code context
    context_service: Arc<dyn ContextServiceInterface>,
    /// Service for semantic code search
    search_service: Arc<dyn SearchServiceInterface>,
    /// Server information
    server_info: ServerInfo,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    ///
    /// # Arguments
    /// * `indexing_service` - Service for indexing codebases
    /// * `context_service` - Service for providing code context
    /// * `search_service` - Service for semantic code search
    ///
    /// # Returns
    /// A new McpServer instance with injected dependencies
    pub fn new(
        indexing_service: Arc<dyn IndexingServiceInterface>,
        context_service: Arc<dyn ContextServiceInterface>,
        search_service: Arc<dyn SearchServiceInterface>,
    ) -> Self {
        let server_info = ServerInfo {
            name: "mcp-context-browser".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        Self {
            indexing_service,
            context_service,
            search_service,
            server_info,
        }
    }

    /// Get server information
    pub fn get_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Get server capabilities
    pub fn get_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(rmcp::model::ServerCapabilitiesTools {
                list_changed: Some(true),
            }),
            prompts: None,
            resources: None,
            logging: None,
        }
    }

    /// Access to indexing service
    pub fn indexing_service(&self) -> Arc<dyn IndexingServiceInterface> {
        Arc::clone(&self.indexing_service)
    }

    /// Access to context service
    pub fn context_service(&self) -> Arc<dyn ContextServiceInterface> {
        Arc::clone(&self.context_service)
    }

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.search_service)
    }
}

#[async_trait]
impl ServerHandler for McpServer {
    /// Handle server initialization
    async fn initialize(&self) -> Result<(), rmcp::Error> {
        info!("MCP Context Browser server initialized");
        Ok(())
    }

    /// Get available tools
    async fn tools(&self) -> Result<Vec<Tool>, rmcp::Error> {
        Ok(vec![
            Tool {
                name: "search_code".to_string(),
                description: "Search for code using semantic similarity and lexical matching".to_string(),
                input_schema: schemars::schema_for!(crate::handlers::SearchCodeArgs),
            },
            Tool {
                name: "index_codebase".to_string(),
                description: "Index a codebase for semantic search".to_string(),
                input_schema: schemars::schema_for!(crate::handlers::IndexCodebaseArgs),
            },
            Tool {
                name: "get_indexing_status".to_string(),
                description: "Get the current indexing status".to_string(),
                input_schema: schemars::schema_for!(crate::handlers::GetIndexingStatusArgs),
            },
            Tool {
                name: "clear_index".to_string(),
                description: "Clear the search index".to_string(),
                input_schema: schemars::schema_for!(crate::handlers::ClearIndexArgs),
            },
        ])
    }

    /// Handle tool calls
    #[instrument(skip(self), fields(tool = %request.name))]
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequest,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::Error> {
        info!("Handling tool call: {}", request.name);

        match request.name.as_str() {
            "search_code" => {
                crate::handlers::search_code::handle_search_code(self, request).await
            }
            "index_codebase" => {
                crate::handlers::index_codebase::handle_index_codebase(self, request).await
            }
            "get_indexing_status" => {
                crate::handlers::get_indexing_status::handle_get_indexing_status(self, request).await
            }
            "clear_index" => {
                crate::handlers::clear_index::handle_clear_index(self, request).await
            }
            _ => {
                error!("Unknown tool: {}", request.name);
                Err(rmcp::Error::invalid_request(format!("Unknown tool: {}", request.name)))
            }
        }
    }
}