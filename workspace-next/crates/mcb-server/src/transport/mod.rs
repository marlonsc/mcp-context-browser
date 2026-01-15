//! MCP Transport Layer
//!
//! Transport implementations for the MCP protocol.
//! Handles different transport mechanisms (stdio, HTTP, etc.).

pub mod stdio;

/// Transport configuration for MCP server
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport mode
    pub mode: TransportMode,
    /// HTTP port (if applicable)
    pub http_port: Option<u16>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            mode: TransportMode::Stdio,
            http_port: None,
        }
    }
}

/// Available transport modes
#[derive(Debug, Clone, Copy)]
pub enum TransportMode {
    /// Standard I/O transport (traditional MCP)
    Stdio,
    /// HTTP transport (for web clients)
    Http,
    /// Both stdio and HTTP simultaneously
    Hybrid,
}