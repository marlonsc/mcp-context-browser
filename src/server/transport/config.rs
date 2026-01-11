//! Transport Configuration Types
//!
//! Configuration for MCP transport modes (stdio, HTTP, hybrid).

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Transport mode selector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// Child process pattern (stdin/stdout)
    Stdio,
    /// Independent HTTP server
    Http,
    /// Both stdio and HTTP simultaneously
    #[default]
    Hybrid,
}

/// Main transport configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, Validate)]
pub struct TransportConfig {
    /// Transport mode
    #[serde(default)]
    pub mode: TransportMode,

    /// HTTP transport settings
    #[serde(default)]
    #[validate(nested)]
    pub http: HttpTransportConfig,

    /// Session management settings
    #[serde(default)]
    #[validate(nested)]
    pub session: SessionConfig,

    /// Version compatibility settings
    #[serde(default)]
    #[validate(nested)]
    pub versioning: VersionConfig,
}

/// HTTP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HttpTransportConfig {
    /// Port for MCP HTTP transport
    #[validate(range(min = 1024, max = 65535))]
    #[serde(default = "default_mcp_port")]
    pub port: u16,

    /// Bind address (localhost by default for security)
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// Enable Server-Sent Events for streaming
    #[serde(default = "default_sse_enabled")]
    pub sse_enabled: bool,

    /// Maximum concurrent sessions
    #[validate(range(min = 1, max = 10000))]
    #[serde(default = "default_max_sessions")]
    pub max_sessions: usize,

    /// Request timeout in seconds
    #[validate(range(min = 5, max = 300))]
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            port: default_mcp_port(),
            bind_address: default_bind_address(),
            sse_enabled: default_sse_enabled(),
            max_sessions: default_max_sessions(),
            request_timeout_secs: default_request_timeout(),
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SessionConfig {
    /// Session TTL in seconds
    #[validate(range(min = 60, max = 86400))]
    #[serde(default = "default_session_ttl")]
    pub ttl_secs: u64,

    /// Enable session resumption after reconnection
    #[serde(default = "default_resumption_enabled")]
    pub resumption_enabled: bool,

    /// Maximum messages to buffer for resumption
    #[validate(range(min = 0, max = 1000))]
    #[serde(default = "default_resumption_buffer")]
    pub resumption_buffer_size: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_session_ttl(),
            resumption_enabled: default_resumption_enabled(),
            resumption_buffer_size: default_resumption_buffer(),
        }
    }
}

/// Version compatibility configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VersionConfig {
    /// Version tolerance (±N minor versions)
    #[validate(range(min = 0, max = 5))]
    #[serde(default = "default_version_tolerance")]
    pub version_tolerance: u32,

    /// Whether to warn on version mismatch (vs hard reject)
    #[serde(default = "default_warn_only")]
    pub warn_only: bool,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            version_tolerance: default_version_tolerance(),
            warn_only: default_warn_only(),
        }
    }
}

// Default value functions
fn default_mcp_port() -> u16 {
    3002
}
fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}
fn default_sse_enabled() -> bool {
    true
}
fn default_max_sessions() -> usize {
    1000
}
fn default_request_timeout() -> u64 {
    30
}
fn default_session_ttl() -> u64 {
    3600
}
fn default_resumption_enabled() -> bool {
    true
}
fn default_resumption_buffer() -> usize {
    100
}
fn default_version_tolerance() -> u32 {
    1
}
fn default_warn_only() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_defaults() {
        let config = TransportConfig::default();
        assert_eq!(config.mode, TransportMode::Hybrid);
        assert_eq!(config.http.port, 3002);
        assert_eq!(config.http.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_http_config_defaults() {
        let config = HttpTransportConfig::default();
        assert_eq!(config.port, 3002);
        assert!(config.sse_enabled);
        assert_eq!(config.max_sessions, 1000);
    }

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfig::default();
        assert_eq!(config.ttl_secs, 3600);
        assert!(config.resumption_enabled);
        assert_eq!(config.resumption_buffer_size, 100);
    }

    #[test]
    fn test_version_config_defaults() {
        let config = VersionConfig::default();
        assert_eq!(config.version_tolerance, 1);
        assert!(config.warn_only);
    }

    #[test]
    fn test_transport_mode_serde() {
        let json = r#""hybrid""#;
        let mode: TransportMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, TransportMode::Hybrid);

        let json = r#""stdio""#;
        let mode: TransportMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, TransportMode::Stdio);
    }
}
