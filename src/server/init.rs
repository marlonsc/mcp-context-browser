//! Server initialization utilities
//!
//! This module contains server initialization logic extracted from the main
//! server implementation to improve code organization and testability.

use crate::core::cache::CacheManager;
use crate::core::database::init_global_database_pool;
use crate::core::http_client::{HttpClientConfig, init_global_http_client};
use crate::core::limits::{init_global_resource_limits, ResourceLimits};
use crate::core::rate_limit::RateLimiter;
use crate::metrics::MetricsApiServer;
use crate::server::McpServer;
use std::sync::Arc;
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use rmcp::ServerHandler;
use tracing_subscriber::{self, EnvFilter};

/// Initialize logging and tracing for the MCP server
///
/// Sets up structured logging with appropriate levels for production use.
/// Uses stderr for logs to avoid interfering with stdio MCP protocol.
fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
                .add_directive("mcp_context_browser=debug".parse()?)
                .add_directive("rmcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false) // Disable ANSI colors for better log parsing
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    Ok(())
}

/// Initialize all server components and services
async fn initialize_server_components(
    cache_manager: Option<Arc<CacheManager>>,
) -> Result<(McpServer, Option<tokio::task::JoinHandle<()>>, Arc<ResourceLimits>), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = crate::config::Config::from_env()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    // Initialize resource limits
    let resource_limits = Arc::new(crate::core::limits::ResourceLimits::new(config.resource_limits.clone()));
    crate::core::limits::init_global_resource_limits(config.resource_limits.clone())?;

    // Initialize global HTTP client pool
    tracing::info!("🌐 Initializing HTTP client pool...");
    if let Err(e) = init_global_http_client(HttpClientConfig::default()) {
        tracing::warn!(
            "⚠️  Failed to initialize HTTP client pool: {}. Using default clients.",
            e
        );
    } else {
        tracing::info!("✅ HTTP client pool initialized successfully");
    }

    // Initialize global database pool
    if config.database.enabled {
        tracing::info!("🗄️  Initializing database connection pool...");
        match init_global_database_pool(config.database.clone()) {
            Ok(_) => tracing::info!("✅ Database connection pool initialized successfully"),
            Err(e) => {
                tracing::error!("💥 Failed to initialize database connection pool: {}", e);
                return Err(e.into());
            }
        }
    } else {
        tracing::info!("ℹ️  Database connection pool disabled");
    }

    // Initialize rate limiter for HTTP API
    let rate_limiter = if config.metrics.rate_limiting.enabled {
        tracing::info!("🔒 Initializing rate limiter...");
        let limiter = Arc::new(RateLimiter::new(config.metrics.rate_limiting.clone()));
        if let Err(e) = limiter.init().await {
            tracing::warn!(
                "⚠️  Failed to initialize Redis rate limiter: {}. Running without rate limiting.",
                e
            );
            None
        } else {
            tracing::info!("✅ Rate limiter initialized successfully");
            Some(limiter)
        }
    } else {
        tracing::info!("ℹ️  Rate limiting disabled");
        None
    };

    // Initialize cache manager
    let cache_manager = if config.cache.enabled {
        tracing::info!("🗄️  Initializing cache manager...");
        match CacheManager::new(config.cache.clone()).await {
            Ok(manager) => {
                tracing::info!("✅ Cache manager initialized successfully");
                Some(Arc::new(manager))
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  Failed to initialize cache manager: {}. Running without caching.",
                    e
                );
                cache_manager // Use provided cache manager if available
            }
        }
    } else {
        tracing::info!("ℹ️  Caching disabled");
        cache_manager
    };

    // Create server instance
    let server = match McpServer::new(cache_manager) {
        Ok(server) => {
            tracing::info!("✅ Service providers initialized successfully");
            server
        }
        Err(e) => {
            tracing::error!("❌ Failed to initialize server: {}", e);
            return Err(e);
        }
    };

    // Log server capabilities
    let capabilities = server.get_info().capabilities;
    tracing::info!(
        "🔧 Server capabilities: tools={}, prompts={}, resources={}",
        capabilities.tools.is_some(),
        capabilities.prompts.is_some(),
        capabilities.resources.is_some()
    );

    // Initialize metrics server if enabled
    let metrics_handle = if config.metrics.enabled {
        tracing::info!(
            "📊 Starting metrics HTTP server on port {}",
            config.metrics.port
        );
        let metrics_server = MetricsApiServer::with_limits(
            config.metrics.port,
            rate_limiter.clone(),
            Some(resource_limits.clone()),
        );

        Some(tokio::spawn(async move {
            if let Err(e) = metrics_server.start().await {
                tracing::error!("💥 Metrics server failed: {}", e);
            }
        }))
    } else {
        tracing::info!("ℹ️  Metrics server disabled");
        None
    };

    Ok((server, metrics_handle, resource_limits))
}

/// Run the MCP Context Browser server
///
/// This is the main entry point that starts the MCP server with stdio transport.
/// The server implements the MCP protocol for semantic code search and indexing.
///
/// # Architecture Notes
/// - Async-first design using Tokio runtime
/// - Provider pattern for extensibility
/// - Structured concurrency with proper error handling
/// - Comprehensive logging and observability
/// - Rate limiting for production security
///
/// # Features
/// - Semantic code search using vector embeddings
/// - Multi-language support with AST parsing
/// - Enterprise-grade error handling and timeouts
/// - SOC 2 compliant logging and security
/// - Production-ready rate limiting
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing first for proper error reporting
    init_tracing()?;

    tracing::info!(
        "🚀 Starting MCP Context Browser v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!(
        "📋 System Info: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    // Initialize all server components
    let (server, metrics_handle, resource_limits) = initialize_server_components(None).await?;

    tracing::info!("📡 Starting MCP protocol server on stdio transport");
    tracing::info!("🎯 Ready to accept MCP client connections");

    // Handle graceful shutdown signals
    let shutdown_signal = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for shutdown signal: {}", e);
            return;
        }
        tracing::info!("🛑 Received shutdown signal, initiating graceful shutdown...");
    };

    // Start the MCP service with stdio transport
    let service_future = server.serve(stdio());

    tokio::select! {
        result = service_future => {
            match result {
                Ok(service) => {
                    tracing::info!("🎉 MCP server started successfully, waiting for connections...");
                    service.waiting().await?;
                    tracing::info!("👋 MCP server shutdown complete");
                }
                Err(e) => {
                    tracing::error!("💥 Failed to start MCP service: {:?}", e);
                    return Err(e.into());
                }
            }
        }
        _ = shutdown_signal => {
            tracing::info!("🔄 Graceful shutdown initiated");
        }
    }

    // Wait for metrics server to finish if it was started
    if let Some(handle) = metrics_handle {
        let _ = handle.await;
    }

    Ok(())
}