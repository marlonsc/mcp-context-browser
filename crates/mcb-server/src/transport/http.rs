//! HTTP Transport for MCP
//!
//! Implements MCP protocol over HTTP using Server-Sent Events (SSE).
//! This transport allows web clients to connect to the MCP server.
//!
//! # Supported Methods
//!
//! | Method | Description |
//! |--------|-------------|
//! | `initialize` | Initialize the MCP session |
//! | `tools/list` | List available tools |
//! | `tools/call` | Call a tool with arguments |
//! | `ping` | Health check |
//!
//! # Example
//!
//! ```ignore
//! // POST /mcp with JSON-RPC request
//! {
//!     "jsonrpc": "2.0",
//!     "method": "tools/list",
//!     "id": 1
//! }
//! ```

use super::types::{McpRequest, McpResponse};
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{create_tool_list, route_tool_call, ToolHandlers};
use crate::McpServer;
use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use rmcp::model::CallToolRequestParam;
use rmcp::ServerHandler;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Enable CORS for browser access
    pub enable_cors: bool,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
        }
    }
}

impl HttpTransportConfig {
    /// Create config for localhost with specified port
    pub fn localhost(port: u16) -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port,
            enable_cors: true,
        }
    }

    /// Get the socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], self.port)))
    }
}

/// Shared state for HTTP transport
#[derive(Clone)]
pub struct HttpTransportState {
    /// Broadcast channel for SSE events
    pub event_tx: broadcast::Sender<String>,
    /// MCP server reference (for handling requests)
    pub server: Arc<McpServer>,
}

/// HTTP transport server
pub struct HttpTransport {
    config: HttpTransportConfig,
    state: HttpTransportState,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(config: HttpTransportConfig, server: Arc<McpServer>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            config,
            state: HttpTransportState { event_tx, server },
        }
    }

    /// Create the HTTP router
    pub fn router(&self) -> Router {
        let state = self.state.clone();

        let router = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/events", get(handle_sse))
            .with_state(state);

        if self.config.enable_cors {
            // Add CORS headers
            router.layer(
                tower_http::cors::CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                    .allow_headers(tower_http::cors::Any),
            )
        } else {
            router
        }
    }

    /// Start the HTTP transport server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        info!("HTTP transport listening on {}", addr);

        let router = self.router();
        axum::serve(listener, router).await?;

        Ok(())
    }

    /// Start with graceful shutdown
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        info!("HTTP transport listening on {}", addr);

        let router = self.router();
        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        Ok(())
    }
}

/// Handle MCP request via HTTP POST
///
/// Routes MCP JSON-RPC requests to the appropriate handlers based on method name.
///
/// # Supported Methods
///
/// - `initialize`: Returns server info and capabilities
/// - `tools/list`: Returns list of available tools
/// - `tools/call`: Executes a tool with provided arguments
/// - `ping`: Returns empty success response for health checks
async fn handle_mcp_request(
    State(state): State<HttpTransportState>,
    Json(request): Json<McpRequest>,
) -> impl IntoResponse {
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, &request).await,
        "tools/list" => handle_tools_list(&state, &request).await,
        "tools/call" => handle_tools_call(&state, &request).await,
        "ping" => McpResponse::success(request.id.clone(), serde_json::json!({})),
        _ => McpResponse::error(
            request.id.clone(),
            JSONRPC_METHOD_NOT_FOUND,
            format!("Unknown method: {}", request.method),
        ),
    };

    Json(response)
}

/// Handle the `initialize` method
///
/// Returns server information and capabilities.
async fn handle_initialize(state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    let server_info = state.server.get_info();

    let result = serde_json::json!({
        "protocolVersion": format!("{:?}", server_info.protocol_version),
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": server_info.server_info.name,
            "version": server_info.server_info.version
        },
        "instructions": server_info.instructions
    });

    McpResponse::success(request.id.clone(), result)
}

/// Handle the `tools/list` method
///
/// Returns all available tools with their schemas.
async fn handle_tools_list(_state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    match create_tool_list() {
        Ok(tools) => {
            let tools_json: Vec<serde_json::Value> = tools
                .into_iter()
                .map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema.as_ref()
                    })
                })
                .collect();

            McpResponse::success(
                request.id.clone(),
                serde_json::json!({ "tools": tools_json }),
            )
        }
        Err(e) => {
            error!(error = ?e, "Failed to list tools");
            McpResponse::error(
                request.id.clone(),
                JSONRPC_INTERNAL_ERROR,
                format!("Failed to list tools: {:?}", e),
            )
        }
    }
}

/// Parse tool call parameters from the request
fn parse_tool_call_params(
    params: &serde_json::Value,
) -> Result<CallToolRequestParam, (i32, &'static str)> {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or((
            JSONRPC_INVALID_PARAMS,
            "Missing 'name' parameter for tools/call",
        ))?
        .to_string();

    let arguments = params.get("arguments").and_then(|v| v.as_object().cloned());

    Ok(CallToolRequestParam {
        name: tool_name.into(),
        arguments,
    })
}

/// Convert tool call result to JSON response
fn tool_result_to_json(result: rmcp::model::CallToolResult) -> serde_json::Value {
    let content_json: Vec<serde_json::Value> = result
        .content
        .iter()
        .map(|content| {
            serde_json::to_value(content).unwrap_or(serde_json::json!({
                "type": "text",
                "text": "Error serializing content"
            }))
        })
        .collect();

    serde_json::json!({
        "content": content_json,
        "isError": result.is_error.unwrap_or(false)
    })
}

/// Handle the `tools/call` method
///
/// Executes the specified tool with the provided arguments.
async fn handle_tools_call(state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    let params = match &request.params {
        Some(params) => params,
        None => {
            return McpResponse::error(
                request.id.clone(),
                JSONRPC_INVALID_PARAMS,
                "Missing params for tools/call",
            );
        }
    };

    let call_request = match parse_tool_call_params(params) {
        Ok(req) => req,
        Err((code, msg)) => return McpResponse::error(request.id.clone(), code, msg),
    };

    let handlers = ToolHandlers {
        index_codebase: state.server.index_codebase_handler(),
        search_code: state.server.search_code_handler(),
        get_indexing_status: state.server.get_indexing_status_handler(),
        clear_index: state.server.clear_index_handler(),
    };

    match route_tool_call(call_request, &handlers).await {
        Ok(result) => McpResponse::success(request.id.clone(), tool_result_to_json(result)),
        Err(e) => {
            error!(error = ?e, "Tool call failed");
            McpResponse::error(
                request.id.clone(),
                JSONRPC_INTERNAL_ERROR,
                format!("Tool call failed: {:?}", e),
            )
        }
    }
}

/// Handle SSE connection for server-to-client events
async fn handle_sse(
    State(state): State<HttpTransportState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();

    let stream = async_stream::stream! {
        let mut rx = rx;
        while let Ok(data) = rx.recv().await {
            yield Ok(Event::default().data(data));
        }
    };

    Sse::new(stream)
}
