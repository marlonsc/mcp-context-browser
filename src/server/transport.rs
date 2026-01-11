//! MCP Transport Layer
//!
//! Provides transport implementations for the MCP server:
//! - Stdio: Traditional child process pattern (stdin/stdout)
//! - HTTP: Streamable HTTP with SSE (per MCP spec)
//! - Hybrid: Both stdio and HTTP simultaneously
//!
//! The transport layer handles:
//! - Session management for HTTP clients
//! - Server version compatibility (±1 minor version)
//! - Connection tracking for graceful shutdown
//! - Message buffering for SSE resumption

pub mod config;
pub mod http;
pub mod session;
pub mod versioning;

// Re-exports for convenience
pub use config::{
    HttpTransportConfig, SessionConfig, TransportConfig, TransportMode, VersionConfig,
};
pub use http::{create_mcp_router, HttpTransportState, McpError};
pub use session::{
    create_shared_session_manager, BufferedMessage, McpSession, SessionError, SessionManager,
    SessionState, SharedSessionManager,
};
pub use versioning::{headers, CompatibilityResult, SemVer, VersionChecker, VersionInfo};
