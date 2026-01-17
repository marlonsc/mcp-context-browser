//! Server configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Transport mode for MCP server
///
/// Defines how the MCP server communicates with clients.
///
/// # Modes
///
/// | Mode | Description | Use Case |
/// |------|-------------|----------|
/// | `Stdio` | Standard I/O streams | CLI tools, IDE integrations |
/// | `Http` | HTTP with SSE | Web clients, REST APIs |
/// | `Hybrid` | Both simultaneously | Dual-interface servers |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Standard I/O transport (traditional MCP protocol)
    /// Used for CLI tools and IDE integrations (e.g., Claude Code)
    #[default]
    Stdio,
    /// HTTP transport with Server-Sent Events
    /// Used for web clients and REST API access
    Http,
    /// Both Stdio and HTTP simultaneously
    /// Allows serving both CLI and web clients from the same process
    Hybrid,
}

/// Network configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNetworkConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Admin API port (separate from main server port)
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
}

/// SSL/TLS configuration for server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerSslConfig {
    /// HTTPS enabled
    pub https: bool,

    /// SSL certificate path (if HTTPS enabled)
    pub ssl_cert_path: Option<PathBuf>,

    /// SSL key path (if HTTPS enabled)
    pub ssl_key_path: Option<PathBuf>,
}

/// Timeout configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTimeoutConfig {
    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Maximum request body size in bytes
    pub max_request_body_size: usize,
}

/// CORS configuration for server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCorsConfig {
    /// Enable CORS
    pub cors_enabled: bool,

    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    /// Transport mode (stdio, http, hybrid)
    #[serde(default)]
    pub transport_mode: TransportMode,

    /// Network configuration
    pub network: ServerNetworkConfig,

    /// SSL/TLS configuration
    pub ssl: ServerSslConfig,

    /// Timeout configuration
    pub timeouts: ServerTimeoutConfig,

    /// CORS configuration
    pub cors: ServerCorsConfig,
}

/// Default admin port (9090)
fn default_admin_port() -> u16 {
    9090
}

// Default implementations for config structs
impl Default for ServerNetworkConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: DEFAULT_HTTP_PORT,
            admin_port: default_admin_port(),
        }
    }
}

impl Default for ServerTimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout_secs: REQUEST_TIMEOUT_SECS,
            connection_timeout_secs: CONNECTION_TIMEOUT_SECS,
            max_request_body_size: MAX_REQUEST_BODY_SIZE,
        }
    }
}

impl Default for ServerCorsConfig {
    fn default() -> Self {
        Self {
            cors_enabled: true,
            cors_origins: vec!["*".to_string()],
        }
    }
}