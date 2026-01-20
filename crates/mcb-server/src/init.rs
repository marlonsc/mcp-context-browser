//! Server Initialization
//!
//! Handles server startup, dependency injection setup, and graceful shutdown.
//! Integrates with the infrastructure layer for configuration and DI container setup.
//!
//! # Architecture (Clean Architecture + Handle-based DI)
//!
//! The server initialization follows a handle-based DI approach:
//!
//! 1. **Provider Handles** (Infrastructure): RwLock wrappers for runtime-swappable providers
//! 2. **Runtime Factory** (Application): Creates domain services with providers from handles
//!
//! Production providers are resolved via linkme registry using `AppConfig`,
//! wrapped in handles, and can be switched at runtime via admin API.
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
//! - Environment variable: `MCP__SERVER__TRANSPORT_MODE=http`

use std::path::Path;
use std::sync::Arc;

use mcb_infrastructure::cache::provider::SharedCacheProvider;
use mcb_infrastructure::config::TransportMode;
use mcb_infrastructure::crypto::CryptoService;
use tracing::{error, info};

use crate::McpServer;
use crate::McpServerBuilder;
use crate::transport::http::{HttpTransport, HttpTransportConfig};
use crate::transport::stdio::StdioServerExt;

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
pub async fn run_server(config_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_path)?;
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

    let server = create_mcp_server(config).await?;
    info!("MCP server initialized successfully");

    start_transport(server, transport_mode, &http_host, http_port).await
}

/// Load configuration from optional path
fn load_config(
    config_path: Option<&Path>,
) -> Result<mcb_infrastructure::config::AppConfig, Box<dyn std::error::Error>> {
    let loader = match config_path {
        Some(path) => mcb_infrastructure::config::ConfigLoader::new().with_config_path(path),
        None => mcb_infrastructure::config::ConfigLoader::new(),
    };
    Ok(loader.load()?)
}

/// Create and configure the MCP server with all services
async fn create_mcp_server(
    config: mcb_infrastructure::config::AppConfig,
) -> Result<McpServer, Box<dyn std::error::Error>> {
    // Create AppContext with resolved providers
    let app_context = mcb_infrastructure::di::bootstrap::init_app(config.clone()).await?;

    // Get all providers from handles (runtime-swappable via admin API)
    let embedding_provider = app_context.embedding_handle().get();
    let vector_store_provider = app_context.vector_store_handle().get();
    let cache_provider = app_context.cache_handle().get();
    let language_chunker = app_context.language_handle().get();

    // Create shared cache provider (conversion for domain services factory)
    let shared_cache = SharedCacheProvider::from_arc(cache_provider);
    let crypto = create_crypto_service(&config).await?;

    // Create domain services with providers
    let deps = mcb_infrastructure::di::modules::domain_services::ServiceDependencies {
        cache: shared_cache,
        crypto,
        config,
        embedding_provider,
        vector_store_provider,
        language_chunker,
    };
    let services =
        mcb_infrastructure::di::modules::domain_services::DomainServicesFactory::create_services(
            deps,
        )
        .await?;

    McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .try_build()
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
}

/// Start the appropriate transport based on configuration
async fn start_transport(
    server: McpServer,
    transport_mode: TransportMode,
    http_host: &str,
    http_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match transport_mode {
        TransportMode::Stdio => {
            info!("Starting stdio transport");
            run_stdio_transport(server).await
        }
        TransportMode::Http => {
            info!(host = %http_host, port = http_port, "Starting HTTP transport");
            run_http_transport(server, http_host, http_port).await
        }
        TransportMode::Hybrid => {
            info!(
                host = %http_host,
                port = http_port,
                "Starting hybrid transport (stdio + HTTP)"
            );
            run_hybrid_transport(server, http_host, http_port).await
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

    // Wait for both transports to finish (join keeps both running)
    let (stdio_result, http_result) = tokio::join!(stdio_handle, http_handle);

    if let Err(e) = stdio_result {
        error!(error = %e, "Hybrid: stdio transport task panicked");
    } else {
        info!("Hybrid: stdio transport task completed");
    }

    if let Err(e) = http_result {
        error!(error = %e, "Hybrid: HTTP transport task panicked");
    } else {
        info!("Hybrid: HTTP transport task completed");
    }

    Ok(())
}

// =============================================================================
// Crypto Service Creation - Only service not yet in Shaku modules
// =============================================================================

/// Create crypto service from configuration
///
/// Uses JWT secret from config if available (32+ bytes), otherwise generates a random key.
async fn create_crypto_service(
    config: &mcb_infrastructure::config::AppConfig,
) -> Result<CryptoService, Box<dyn std::error::Error>> {
    // AES-GCM requires exactly 32 bytes for the key
    let master_key = if config.auth.jwt.secret.len() >= 32 {
        config.auth.jwt.secret.as_bytes()[..32].to_vec()
    } else {
        CryptoService::generate_master_key()
    };

    CryptoService::new(master_key).map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
}
