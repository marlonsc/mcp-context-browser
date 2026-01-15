//! Stdio Transport for MCP
//!
//! Implements MCP protocol over standard input/output streams.
//! This is the traditional transport mechanism for MCP servers.

use crate::McpServer;
use rmcp::transport::stdio;
use rmcp::ServerHandler;
use tracing::info;

/// Extension trait for McpServer to add stdio serving capability
pub trait StdioServerExt {
    /// Serve the MCP server over stdio transport
    async fn serve_stdio(self) -> Result<(), Box<dyn std::error::Error>>;
}

impl StdioServerExt for McpServer {
    async fn serve_stdio(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("📡 Starting MCP protocol server on stdio transport");

        let service = self.serve(stdio()).await
            .map_err(|e| format!("Failed to start MCP service: {:?}", e))?;

        info!("🎉 MCP server started successfully, waiting for connections...");
        service.waiting().await
            .map_err(|e| format!("MCP service error: {:?}", e))?;

        info!("👋 MCP server shutdown complete");
        Ok(())
    }
}