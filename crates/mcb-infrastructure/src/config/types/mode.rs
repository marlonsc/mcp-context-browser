//! Operating mode configuration
//!
//! Defines how MCB operates: standalone (local providers), client (connects to server),
//! or server (daemon mode, triggered by --server flag).

use serde::{Deserialize, Serialize};

/// Default server URL for client mode
fn default_server_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

/// Operating mode for MCB
///
/// Determines how MCB behaves when started without the `--server` flag:
/// - `Standalone`: Run with local providers (default, backwards compatible)
/// - `Client`: Connect to a remote MCB server via HTTP
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OperatingMode {
    /// Standalone mode: run with local providers
    /// This is the default for backwards compatibility
    #[default]
    Standalone,

    /// Client mode: connect to remote MCB server
    /// Requires server_url to be configured
    Client,
}

/// Mode configuration section
///
/// Controls how MCB operates:
///
/// ```toml
/// [mode]
/// type = "client"                         # "standalone" or "client"
/// server_url = "http://127.0.0.1:8080"   # Server URL for client mode
/// session_prefix = "claude"               # Optional prefix for session isolation
/// ```
///
/// When `--server` flag is used, mode configuration is ignored and MCB runs as server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    /// Operating mode type
    #[serde(default, rename = "type")]
    pub mode_type: OperatingMode,

    /// Server URL for client mode
    /// Only used when mode_type = Client
    #[serde(default = "default_server_url")]
    pub server_url: String,

    /// Session prefix for context isolation
    /// Optional: if set, collections will be prefixed with this value
    pub session_prefix: Option<String>,

    /// Connection timeout in seconds for client mode
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,

    /// Enable automatic reconnection on connection loss
    #[serde(default = "default_auto_reconnect")]
    pub auto_reconnect: bool,

    /// Maximum reconnection attempts (0 = unlimited)
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
}

fn default_timeout_secs() -> u64 {
    30
}

fn default_auto_reconnect() -> bool {
    true
}

fn default_max_reconnect_attempts() -> u32 {
    5
}

/// Default configuration for standalone mode operation.
///
/// Provides sensible defaults for local development and backwards compatibility:
/// - Mode: Standalone (local providers)
/// - Server URL: http://127.0.0.1:8080 (used only in client mode)
/// - Timeout: 30 seconds
/// - Auto-reconnect: enabled with 5 max attempts
impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            mode_type: OperatingMode::default(),
            server_url: default_server_url(),
            session_prefix: None,
            timeout_secs: default_timeout_secs(),
            auto_reconnect: default_auto_reconnect(),
            max_reconnect_attempts: default_max_reconnect_attempts(),
        }
    }
}

impl ModeConfig {
    /// Check if running in client mode
    pub fn is_client(&self) -> bool {
        self.mode_type == OperatingMode::Client
    }

    /// Check if running in standalone mode
    pub fn is_standalone(&self) -> bool {
        self.mode_type == OperatingMode::Standalone
    }

    /// Get the server URL (only meaningful in client mode)
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get session prefix if configured
    pub fn session_prefix(&self) -> Option<&str> {
        self.session_prefix.as_deref()
    }
}

// Tests moved to crates/mcb-server/tests/integration/operating_modes_integration.rs
// See: test_mode_config_* tests for coverage
