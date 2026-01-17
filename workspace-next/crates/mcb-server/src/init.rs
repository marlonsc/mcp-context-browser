//! Server Initialization
//!
//! Handles server startup, dependency injection setup, and graceful shutdown.
//! Integrates with the infrastructure layer for configuration and DI container setup.
//!
//! # Transport Modes
//!
//! The server supports three transport modes configured via `ServerConfig.transport_mode`:
//!
//! - **Stdio**: Traditional MCP protocol over stdin/stdout (default)
//! - **Http**: HTTP REST API with Server-Sent Events for web clients
//! - **Hybrid**: Both Stdio and HTTP running simultaneously
//!
//! # Configuration
//!
//! Transport mode can be set via:
//! - Config file: `server.transport_mode = "http"`
//! - Environment variable: `MCB_SERVER_TRANSPORT_MODE=http`

use std::path::Path;
use std::sync::Arc;

use mcb_infrastructure::config::TransportMode;
use tracing::{error, info};

use crate::transport::http::{HttpTransport, HttpTransportConfig};
use crate::transport::stdio::StdioServerExt;
use crate::McpServer;
use crate::McpServerBuilder;

/// Run the MCP Context Browser server
///
/// This is the main entry point that initializes all components and starts the server.
/// It handles configuration loading, dependency injection, and MCP server startup.
///
/// # Transport Mode Selection
///
/// The transport mode is determined by `config.server.transport_mode`:
///
/// - `Stdio` (default): Runs MCP over stdin/stdout
/// - `Http`: Runs HTTP server on configured port
/// - `Hybrid`: Runs both Stdio and HTTP concurrently
///
/// # Example
///
/// ```rust,ignore
/// // Run with default config (stdio mode)
/// run_server(None).await?;
///
/// // Run with custom config file
/// run_server(Some(Path::new("config.toml"))).await?;
/// ```
pub async fn run_server(config_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let loader = match config_path {
        Some(path) => mcb_infrastructure::config::ConfigLoader::new().with_config_path(path),
        None => mcb_infrastructure::config::ConfigLoader::new(),
    };

    let config = loader.load()?;
    mcb_infrastructure::logging::init_logging(config.logging.clone())?;

    info!(
        transport_mode = ?config.server.transport_mode,
        host = %config.server.network.host,
        port = %config.server.network.port,
        "Starting MCP Context Browser server"
    );

    let transport_mode = config.server.transport_mode;
    let http_host = config.server.network.host.clone();
    let http_port = config.server.network.port;

    let container = mcb_infrastructure::di::bootstrap::FullContainer::new(config).await?;

    let server = McpServerBuilder::new()
        .with_indexing_service(container.indexing_service())
        .with_context_service(container.context_service())
        .with_search_service(container.search_service())
        .try_build()?;

    info!("MCP server initialized successfully");

    // Route to appropriate transport based on configuration
    match transport_mode {
        TransportMode::Stdio => {
            info!("Starting stdio transport");
            run_stdio_transport(server).await
        }
        TransportMode::Http => {
            info!(host = %http_host, port = http_port, "Starting HTTP transport");
            run_http_transport(server, &http_host, http_port).await
        }
        TransportMode::Hybrid => {
            info!(
                host = %http_host,
                port = http_port,
                "Starting hybrid transport (stdio + HTTP)"
            );
            run_hybrid_transport(server, &http_host, http_port).await
        }
    }
}

/// Run the server with stdio transport only
///
/// This is the traditional MCP transport mode, communicating over stdin/stdout.
/// Used for CLI tools and IDE integrations like Claude Code.
async fn run_stdio_transport(server: McpServer) -> Result<(), Box<dyn std::error::Error>> {
    server.serve_stdio().await
}

/// Run the server with HTTP transport only
///
/// Starts an HTTP server that handles MCP requests via REST API
/// and provides Server-Sent Events for server-to-client notifications.
async fn run_http_transport(
    server: McpServer,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_config = HttpTransportConfig {
        host: host.to_string(),
        port,
        enable_cors: true,
    };

    let http_transport = HttpTransport::new(http_config, Arc::new(server));
    http_transport
        .start()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { e })
}

/// Run the server with both stdio and HTTP transports simultaneously
///
/// Spawns both transports as concurrent tasks. This allows the server to:
/// - Serve CLI tools via stdin/stdout
/// - Serve web clients via HTTP
///
/// If either transport fails, the error is logged and the other continues.
/// The function returns when both transports have finished.
async fn run_hybrid_transport(
    server: McpServer,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    // Clone server for each transport (McpServer is Clone)
    let stdio_server = server.clone();
    let http_server = Arc::new(server);
    let http_host = host.to_string();

    // Spawn stdio transport task
    let stdio_handle = tokio::spawn(async move {
        info!("Hybrid: starting stdio transport");
        if let Err(e) = stdio_server.serve_stdio().await {
            error!(error = %e, "Hybrid: stdio transport failed");
        }
        info!("Hybrid: stdio transport finished");
    });

    // Spawn HTTP transport task
    let http_handle = tokio::spawn(async move {
        info!("Hybrid: starting HTTP transport on {}:{}", http_host, port);
        let http_config = HttpTransportConfig {
            host: http_host,
            port,
            enable_cors: true,
        };

        let http_transport = HttpTransport::new(http_config, http_server);
        if let Err(e) = http_transport.start().await {
            error!(error = %e, "Hybrid: HTTP transport failed");
        }
        info!("Hybrid: HTTP transport finished");
    });

    // Wait for either transport to finish (use select for graceful handling)
    tokio::select! {
        _ = stdio_handle => {
            info!("Hybrid: stdio transport task completed");
        }
        _ = http_handle => {
            info!("Hybrid: HTTP transport task completed");
        }
    }

    Ok(())
}
