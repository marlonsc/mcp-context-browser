// Module declarations
pub mod admin;
pub mod args;
pub mod auth;
pub mod builder;
pub mod formatter;
pub mod handlers;
pub mod init;
pub mod mcp_server;
pub mod metrics;
pub mod operations;
pub mod rate_limit_middleware;
pub mod security;
pub mod transport;

// Re-exports for public API
pub use args::*;
pub use auth::AuthHandler;
pub use builder::McpServerBuilder;
pub use formatter::ResponseFormatter;
pub use handlers::*;
pub use init::run_server;
pub use mcp_server::McpServer;
pub use metrics::{McpPerformanceMetrics, PerformanceMetricsInterface};
pub use operations::{IndexingOperation, IndexingOperationsInterface, McpIndexingOperations};
