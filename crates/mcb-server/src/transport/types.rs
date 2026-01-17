//! Transport layer types
//!
//! Common types used across transport implementations for MCP protocol messages.

use serde::{Deserialize, Serialize};

/// MCP request payload (JSON-RPC format)
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    /// JSON-RPC method
    pub method: String,
    /// Request parameters
    pub params: Option<serde_json::Value>,
    /// Request ID
    pub id: Option<serde_json::Value>,
}

/// MCP response payload (JSON-RPC format)
#[derive(Debug, Serialize)]
pub struct McpResponse {
    /// JSON-RPC version
    pub jsonrpc: &'static str,
    /// Response result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
    /// Request ID
    pub id: Option<serde_json::Value>,
}

/// MCP error response (JSON-RPC format)
#[derive(Debug, Serialize)]
pub struct McpError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
}

impl McpResponse {
    /// Create a success response
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(id: Option<serde_json::Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            result: None,
            error: Some(McpError {
                code,
                message: message.into(),
            }),
            id,
        }
    }
}
