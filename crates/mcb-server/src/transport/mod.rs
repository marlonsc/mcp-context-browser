//! MCP Transport Layer
//!
//! Transport implementations for the MCP protocol.
//! Handles different transport mechanisms (stdio, HTTP, client, etc.).
//!
//! ## Available Transports
//!
//! | Transport | Description | Use Case |
//! |-----------|-------------|----------|
//! | [`stdio`] | Standard I/O streams | CLI tools, IDE integrations |
//! | [`http`] | HTTP server with SSE | Web clients, REST APIs |
//! | [`http_client`] | HTTP client (stdio bridge) | Client mode connecting to server |
//!
//! ## Usage
//!
//! Transport selection is typically handled by the initialization system.
//! The initialization system configures transports based on the operating mode.

pub mod config;
pub mod http;
pub mod http_client;
pub mod stdio;
pub mod types;

// Re-export transport types
pub use config::TransportConfig;
pub use http::{HttpTransport, HttpTransportConfig};
pub use http_client::{HttpClientTransport, McpClientConfig};
pub use stdio::StdioServerExt;
pub use types::{McpError, McpRequest, McpResponse};

// Re-export TransportMode from infrastructure config (single source of truth)
pub use mcb_infrastructure::config::TransportMode;
