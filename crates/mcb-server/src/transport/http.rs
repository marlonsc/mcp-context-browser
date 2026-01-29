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
//! ```text
//! POST /mcp HTTP/1.1
//! Content-Type: application/json
//!
//! {
//!     "jsonrpc": "2.0",
//!     "method": "tools/list",
//!     "id": 1
//! }
//! ```
//!
//! # Migration Note
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use super::types::{McpRequest, McpResponse};
use crate::McpServer;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{ToolHandlers, create_tool_list, route_tool_call};
use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::{Build, Request, Response, Rocket, State, get, post, routes};
use std::net::SocketAddr;
use std::sync::Arc;
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

    /// Build the Rocket application
    pub fn rocket(&self) -> Rocket<Build> {
        let mut rocket = rocket::build()
            .manage(self.state.clone())
            .mount("/", routes![handle_mcp_request, handle_sse]);

        if self.config.enable_cors {
            rocket = rocket.attach(Cors);
        }

        rocket
    }

    /// Start the HTTP transport server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        info!("HTTP transport listening on {}", addr);

        let figment = rocket::Config::figment()
            .merge(("address", self.config.host.clone()))
            .merge(("port", self.config.port));

        let rocket = self.rocket().configure(figment);

        rocket
            .launch()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }

    /// Start with graceful shutdown
    ///
    /// Note: Rocket handles graceful shutdown internally via Ctrl+C.
    /// The shutdown_signal parameter is kept for API compatibility but
    /// uses Rocket's built-in shutdown mechanism.
    pub async fn start_with_shutdown(
        self,
        _shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Rocket handles graceful shutdown internally
        self.start().await
    }
}

/// CORS Fairing for Rocket
///
/// Adds CORS headers to all responses to allow browser access.
pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
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
#[post("/mcp", format = "json", data = "<request>")]
async fn handle_mcp_request(
    state: &State<HttpTransportState>,
    request: Json<McpRequest>,
) -> Json<McpResponse> {
    let request = request.into_inner();
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(state, &request).await,
        "tools/list" => handle_tools_list(state, &request).await,
        "tools/call" => handle_tools_call(state, &request).await,
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
        "protocolVersion": server_info.protocol_version.to_string(),
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
                        "inputSchema": serde_json::to_value(tool.input_schema.as_ref()).ok()
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
) -> Result<CallToolRequestParams, (i32, &'static str)> {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or((
            JSONRPC_INVALID_PARAMS,
            "Missing 'name' parameter for tools/call",
        ))?
        .to_string();

    let arguments = params.get("arguments").and_then(|v| v.as_object().cloned());

    Ok(CallToolRequestParams {
        name: tool_name.into(),
        arguments,
        task: None,
        meta: None, // Meta is optional in MCP 2024-11-05+
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
#[get("/events")]
fn handle_sse(state: &State<HttpTransportState>) -> EventStream![] {
    let mut rx = state.event_tx.subscribe();

    EventStream! {
        loop {
            match rx.recv().await {
                Ok(data) => yield Event::data(data),
                Err(_) => break,
            }
        }
    }
}
