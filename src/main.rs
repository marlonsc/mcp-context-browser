//! # MCP Context Browser - Main Entry Point
//!
//! This is the main entry point for the MCP Context Browser application.
//! It initializes the MCP server and starts listening for requests on stdio.
//!
//! ## Usage
//!
//! The application is designed to be used as an MCP server by AI assistants
//! like Claude Desktop. It communicates via stdio using the MCP protocol.
//!
//! ## Architecture
//!
//! The main function:
//! 1. Initializes logging with tracing
//! 2. Creates MCP tool handlers with dependency injection
//! 3. Sets up async stdio communication
//! 4. Processes MCP messages in a loop
//! 5. Handles graceful shutdown
//!
//! ## Error Handling
//!
//! All errors are properly logged and the application exits gracefully
//! on critical failures. Network and I/O errors are handled robustly.
//!
//! ## License
//!
//! Licensed under the MIT License.

use mcp_context_browser::daemon::ContextDaemon;
use mcp_context_browser::metrics::MetricsApiServer;
use mcp_context_browser::server::McpToolHandlers;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

/// MCP protocol message
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpMessage {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: Option<String>,
    params: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    error: Option<McpError>,
}

/// MCP error
#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!(
        "🚀 Starting MCP Context Browser v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration from environment
    let config = match mcp_context_browser::config::Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = config.validate() {
        eprintln!("❌ Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    config.print_summary();

    // Start metrics HTTP server in background (if enabled)
    let metrics_handle = if config.metrics_enabled() {
        let metrics_server = MetricsApiServer::new(config.metrics_port());
        let handle = tokio::spawn(async move {
            if let Err(e) = metrics_server.start().await {
                eprintln!("❌ Metrics API server error: {}", e);
            }
        });
        println!(
            "📊 Metrics API available at http://localhost:{}",
            config.metrics_port()
        );
        Some(handle)
    } else {
        println!("📊 Metrics API disabled");
        None
    };

    // Start background daemon for lock cleanup and monitoring
    let daemon = ContextDaemon::with_config(config.daemon.clone());
    let daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.start().await {
            eprintln!("❌ Background daemon error: {}", e);
        }
    });

    println!(
        "🤖 Background daemon started (cleanup: {}s, monitoring: {}s)",
        config.daemon.cleanup_interval_secs, config.daemon.monitoring_interval_secs
    );

    // Create tool handlers
    let tool_handlers = match McpToolHandlers::new() {
        Ok(handlers) => handlers,
        Err(e) => {
            eprintln!("❌ Failed to initialize tool handlers: {}", e);
            std::process::exit(1);
        }
    };

    // Setup MCP server
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut reader = BufReader::new(stdin);
    let mut writer = BufWriter::new(stdout);

    let mut buffer = String::new();

    println!("✅ MCP Context Browser ready - listening on stdio");

    println!("✅ MCP Context Browser ready - listening on stdio");

    loop {
        buffer.clear();
        match reader.read_line(&mut buffer).await {
            Ok(0) => {
                // EOF reached
                println!("Received EOF, shutting down");
                break;
            }
            Ok(_) => {
                let message: Result<McpMessage, _> = serde_json::from_str(buffer.trim());

                match message {
                    Ok(msg) => {
                        let response = handle_message(msg, &tool_handlers).await;
                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if let Err(e) = writer
                                .write_all(format!("{}\n", response_json).as_bytes())
                                .await
                            {
                                eprintln!("Failed to write response: {}", e);
                                break;
                            }
                            if let Err(e) = writer.flush().await {
                                eprintln!("Failed to flush writer: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse message: {}", e);
                        let error_response = McpMessage {
                            jsonrpc: "2.0".to_string(),
                            id: Some(serde_json::Value::Null),
                            method: None,
                            params: None,
                            result: None,
                            error: Some(McpError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(serde_json::json!({"details": e.to_string()})),
                            }),
                        };
                        if let Ok(response_json) = serde_json::to_string(&error_response) {
                            let _ = writer
                                .write_all(format!("{}\n", response_json).as_bytes())
                                .await;
                            let _ = writer.flush().await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stdin: {}", e);
                break;
            }
        }
    }

    // Wait for background services to finish
    if let Some(handle) = metrics_handle {
        let _ = handle.await;
    }
    let _ = daemon_handle.await;

    println!("👋 MCP Context Browser shutdown complete");
    Ok(())
}

/// Handle an MCP message
async fn handle_message(message: McpMessage, tool_handlers: &McpToolHandlers) -> McpMessage {
    match message.method.as_deref() {
        Some("initialize") => handle_initialize(message).await,
        Some("tools/list") => handle_list_tools(message).await,
        Some("tools/call") => handle_call_tool(message, tool_handlers).await,
        _ => McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: None,
            error: Some(McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    }
}

/// Handle initialize request
async fn handle_initialize(message: McpMessage) -> McpMessage {
    println!("Received initialize request");

    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "MCP Context Browser",
            "version": env!("CARGO_PKG_VERSION")
        }
    });

    McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(result),
        error: None,
    }
}

/// Handle tools/list request
async fn handle_list_tools(message: McpMessage) -> McpMessage {
    let tools = McpToolHandlers::get_tools();

    let result = serde_json::json!({
        "tools": tools
    });

    McpMessage {
        jsonrpc: "2.0".to_string(),
        id: message.id,
        method: None,
        params: None,
        result: Some(result),
        error: None,
    }
}

/// Handle tools/call request
async fn handle_call_tool(message: McpMessage, tool_handlers: &McpToolHandlers) -> McpMessage {
    let params = match message.params {
        Some(p) => p,
        None => {
            return McpMessage {
                jsonrpc: "2.0".to_string(),
                id: message.id,
                method: None,
                params: None,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: "Invalid params".to_string(),
                    data: None,
                }),
            };
        }
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return McpMessage {
                jsonrpc: "2.0".to_string(),
                id: message.id,
                method: None,
                params: None,
                result: None,
                error: Some(McpError {
                    code: -32602,
                    message: "Missing tool name".to_string(),
                    data: None,
                }),
            };
        }
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    match tool_handlers.handle_tool_call(name, arguments).await {
        Ok(result) => {
            let result_json = serde_json::to_value(result).unwrap();
            McpMessage {
                jsonrpc: "2.0".to_string(),
                id: message.id,
                method: None,
                params: None,
                result: Some(result_json),
                error: None,
            }
        }
        Err(e) => {
            eprintln!("Tool call failed: {}", e);
            McpMessage {
                jsonrpc: "2.0".to_string(),
                id: message.id,
                method: None,
                params: None,
                result: None,
                error: Some(McpError {
                    code: -32000,
                    message: "Tool execution failed".to_string(),
                    data: Some(serde_json::json!({"details": e.to_string()})),
                }),
            }
        }
    }
}
