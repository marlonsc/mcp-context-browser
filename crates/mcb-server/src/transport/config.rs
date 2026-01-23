//! Transport Configuration Module
//!
//! Configuration types and utilities for MCP server transports.

use mcb_infrastructure::config::{ServerConfig, TransportMode};

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

/// Returns default TransportConfig with Stdio mode and no HTTP configuration
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
    pub fn from_server_config(config: &ServerConfig) -> Self {
        Self {
            mode: config.transport_mode,
            http_port: Some(config.network.port),
            http_host: Some(config.network.host.clone()),
        }
    }
}
