//! MCP Context Browser - Main entry point

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
