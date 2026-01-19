//! MCP Server Implementation
//!
//! Core MCP protocol server that orchestrates semantic code search operations.
//! Follows Clean Architecture principles with dependency injection.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParam, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};

use mcb_application::{ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface};

use crate::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
};
use crate::tools::{ToolHandlers, create_tool_list, route_tool_call};

/// Core MCP server implementation
///
/// This server implements the MCP protocol for semantic code search.
/// It depends only on domain services and receives all dependencies through
/// constructor injection following Clean Architecture principles.
#[derive(Clone)]
pub struct McpServer {
    /// Service for indexing codebases
    indexing_service: Arc<dyn IndexingServiceInterface>,
    /// Service for providing code context
    context_service: Arc<dyn ContextServiceInterface>,
    /// Service for semantic code search
    search_service: Arc<dyn SearchServiceInterface>,
    /// Handler for indexing operations
    index_codebase_handler: Arc<IndexCodebaseHandler>,
    /// Handler for search operations
    search_code_handler: Arc<SearchCodeHandler>,
    /// Handler for indexing status operations
    get_indexing_status_handler: Arc<GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    clear_index_handler: Arc<ClearIndexHandler>,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    pub fn new(
        indexing_service: Arc<dyn IndexingServiceInterface>,
        context_service: Arc<dyn ContextServiceInterface>,
        search_service: Arc<dyn SearchServiceInterface>,
    ) -> Self {
        let index_codebase_handler = Arc::new(IndexCodebaseHandler::new(indexing_service.clone()));
        let search_code_handler = Arc::new(SearchCodeHandler::new(search_service.clone()));
        let get_indexing_status_handler =
            Arc::new(GetIndexingStatusHandler::new(indexing_service.clone()));
        let clear_index_handler = Arc::new(ClearIndexHandler::new(indexing_service.clone()));

        Self {
            indexing_service,
            context_service,
            search_service,
            index_codebase_handler,
            search_code_handler,
            get_indexing_status_handler,
            clear_index_handler,
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

    /// Access to index codebase handler (for HTTP transport)
    pub fn index_codebase_handler(&self) -> Arc<IndexCodebaseHandler> {
        Arc::clone(&self.index_codebase_handler)
    }

    /// Access to search code handler (for HTTP transport)
    pub fn search_code_handler(&self) -> Arc<SearchCodeHandler> {
        Arc::clone(&self.search_code_handler)
    }

    /// Access to get indexing status handler (for HTTP transport)
    pub fn get_indexing_status_handler(&self) -> Arc<GetIndexingStatusHandler> {
        Arc::clone(&self.get_indexing_status_handler)
    }

    /// Access to clear index handler (for HTTP transport)
    pub fn clear_index_handler(&self) -> Arc<ClearIndexHandler> {
        Arc::clone(&self.clear_index_handler)
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "MCP Context Browser - Semantic Code Search\n\n\
                 AI-powered code understanding for semantic search across large codebases.\n\n\
                 Tools:\n\
                 - index_codebase: Build a semantic index for a directory\n\
                 - search_code: Query indexed code using natural language\n\
                 - get_indexing_status: Inspect indexing progress\n\
                 - clear_index: Clear a collection before re-indexing\n"
                    .to_string(),
            ),
        }
    }

    /// List available tools
    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = create_tool_list()?;
        Ok(ListToolsResult {
            tools,
            meta: Default::default(),
            next_cursor: None,
        })
    }

    /// Call a tool
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let handlers = ToolHandlers {
            index_codebase: Arc::clone(&self.index_codebase_handler),
            search_code: Arc::clone(&self.search_code_handler),
            get_indexing_status: Arc::clone(&self.get_indexing_status_handler),
            clear_index: Arc::clone(&self.clear_index_handler),
        };
        route_tool_call(request, &handlers).await
    }
}
