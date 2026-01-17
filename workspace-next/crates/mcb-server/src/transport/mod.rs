//! MCP Transport Layer
//!
//! Transport implementations for the MCP protocol.
//! Handles different transport mechanisms (stdio, HTTP, etc.).
//!
//! ## Available Transports
//!
//! | Transport | Description | Use Case |
//! |-----------|-------------|----------|
//! | [`stdio`] | Standard I/O streams | CLI tools, IDE integrations |
//! | [`http`] | HTTP with SSE | Web clients, REST APIs |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use mcb_server::transport::{TransportConfig, TransportMode};
//! use mcb_server::McpServer;
//!
//! let server = McpServer::new(/* ... */);
//!
//! // Stdio transport (traditional MCP)
//! server.serve_stdio().await?;
//!
//! // HTTP transport (for web clients)
//! let http = HttpTransport::new(config, Arc::new(server));
//! http.start().await?;
//! ```

pub mod http;
pub mod stdio;
pub mod types;

// Re-export transport types
pub use http::{HttpTransport, HttpTransportConfig};
pub use stdio::StdioServerExt;
pub use types::{McpError, McpRequest, McpResponse};

// Re-export TransportMode from infrastructure config (single source of truth)
pub use mcb_infrastructure::config::TransportMode;

/// Transport configuration for MCP server
///
/// This struct provides convenience methods for creating transport configurations.
/// The canonical `TransportMode` is defined in `mcb_infrastructure::config`.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport mode (from mcb_infrastructure)
    pub mode: TransportMode,
    /// HTTP port (if applicable)
    pub http_port: Option<u16>,
    /// HTTP host (if applicable)
    pub http_host: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            mode: TransportMode::Stdio,
            http_port: None,
            http_host: None,
        }
    }
}

impl TransportConfig {
    /// Create stdio-only transport config
    pub fn stdio() -> Self {
        Self {
            mode: TransportMode::Stdio,
            http_port: None,
            http_host: None,
        }
    }

    /// Create HTTP-only transport config
    pub fn http(port: u16) -> Self {
        Self {
            mode: TransportMode::Http,
            http_port: Some(port),
            http_host: Some("127.0.0.1".to_string()),
        }
    }

    /// Create hybrid transport config (both stdio and HTTP)
    pub fn hybrid(http_port: u16) -> Self {
        Self {
            mode: TransportMode::Hybrid,
            http_port: Some(http_port),
            http_host: Some("127.0.0.1".to_string()),
        }
    }

    /// Create transport config from server configuration
    pub fn from_server_config(config: &mcb_infrastructure::config::ServerConfig) -> Self {
        Self {
            mode: config.transport_mode,
            http_port: Some(config.network.port),
            http_host: Some(config.network.host.clone()),
        }
    }
}
