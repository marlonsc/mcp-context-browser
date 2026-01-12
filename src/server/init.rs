//! Server initialization utilities
//!
//! This module contains server initialization logic extracted from the main
//! server implementation to improve code organization and testability.

use crate::adapters::http_client::{HttpClientConfig, HttpClientPool};
use crate::infrastructure::cache::CacheManager;
use crate::infrastructure::config::ConfigLoader;
use crate::infrastructure::connection_tracker::{ConnectionTracker, ConnectionTrackerConfig};
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::metrics::MetricsApiServer;
use crate::infrastructure::rate_limit::RateLimiter;
use crate::server::transport::{
    create_mcp_router, HttpTransportState, SessionManager, TransportConfig, TransportMode,
    VersionChecker,
};
use crate::server::McpServer;
use rmcp::transport::stdio;
use rmcp::ServerHandler;
use rmcp::ServiceExt;
use std::sync::Arc;
use tracing_subscriber::{self, EnvFilter};

use crate::infrastructure::events::create_shared_event_bus;
use crate::infrastructure::logging::{create_shared_log_buffer, RingBufferLayer};
use crate::server::McpServerBuilder;
use tracing_subscriber::prelude::*;

/// Initialize logging and tracing for the MCP server
fn init_tracing(
    log_buffer: crate::infrastructure::logging::SharedLogBuffer,
) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into())
        .add_directive("mcp_context_browser=debug".parse()?)
        .add_directive("rmcp=info".parse()?);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    let ring_buffer_layer = RingBufferLayer::new(log_buffer, tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(ring_buffer_layer)
        .init();

    Ok(())
}

/// Initialize all server components and services
///
/// Returns a tuple containing:
/// - McpServer instance
/// - Metrics/Admin/MCP unified HTTP server handle
/// - Resource limits
/// - HTTP client provider
/// - Connection tracker for graceful shutdown
async fn initialize_server_components(
    cache_manager: Option<Arc<CacheManager>>,
    log_buffer: crate::infrastructure::logging::SharedLogBuffer,
) -> Result<
    (
        Arc<McpServer>,
        Option<tokio::task::JoinHandle<()>>,
        Arc<ResourceLimits>,
        Arc<dyn crate::adapters::http_client::HttpClientProvider>,
        Arc<ConnectionTracker>,
    ),
    Box<dyn std::error::Error>,
> {
    // Load configuration from environment
    let loader = ConfigLoader::new();
    let config = loader
        .load()
        .await
        .map_err(|e| format!("Failed to load configuration: {}", e))?;

    let event_bus = create_shared_event_bus();

    // Initialize resource limits
    let resource_limits = Arc::new(crate::infrastructure::limits::ResourceLimits::new(
        config.resource_limits.clone(),
    ));

    // Initialize HTTP client pool
    tracing::info!("🌐 Initializing HTTP client pool...");
    let http_client = match HttpClientPool::with_config(HttpClientConfig::default()) {
        Ok(pool) => {
            tracing::info!("✅ HTTP client pool initialized successfully");
            Arc::new(pool) as Arc<dyn crate::adapters::http_client::HttpClientProvider>
        }
        Err(e) => {
            tracing::warn!(
                "⚠️  Failed to initialize HTTP client pool: {}. Using null client.",
                e
            );
            Arc::new(crate::adapters::http_client::NullHttpClientPool::new())
                as Arc<dyn crate::adapters::http_client::HttpClientProvider>
        }
    };

    // Initialize database pool (not used directly, but available for DI)
    if config.database.enabled {
        tracing::info!("🗄️  Database configuration loaded (used via dependency injection)");
    } else {
        tracing::info!("ℹ️  Database disabled");
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
        match CacheManager::new(config.cache.clone(), Some(event_bus.clone())).await {
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

    // Create server instance using builder
    let mut builder = McpServerBuilder::new()
        .with_log_buffer(log_buffer)
        .with_event_bus(event_bus)
        .with_resource_limits(resource_limits.clone())
        .with_http_client(Arc::clone(&http_client));

    if let Some(ref cm) = cache_manager {
        builder = builder.with_cache(Arc::clone(cm));
    }

    let server = match builder.build().await {
        Ok(server) => {
            tracing::info!("✅ Service providers initialized successfully");
            Arc::new(server)
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

    // Initialize HTTP transport components for unified port architecture
    let transport_config = TransportConfig::default();
    let session_manager = Arc::new(SessionManager::with_defaults());
    let version_checker = Arc::new(VersionChecker::with_defaults());
    let connection_tracker = Arc::new(ConnectionTracker::new(ConnectionTrackerConfig::default()));

    // Initialize unified HTTP server (Metrics + Admin + MCP on single port)
    let metrics_handle = if config.metrics.enabled {
        tracing::info!(
            "📊 Starting unified HTTP server on port {} (Metrics + Admin + MCP)",
            config.metrics.port
        );

        // Initialize admin API server
        let admin_api_server =
            crate::admin::api::AdminApiServer::new(config.admin.clone(), Arc::clone(&server));

        let mut metrics_server = MetricsApiServer::with_limits(
            config.metrics.port,
            server.system_collector(),
            server.performance_metrics(),
            rate_limiter.clone(),
            Some(resource_limits.clone()),
            cache_manager.clone(),
        );

        // Merge admin router into metrics server
        match admin_api_server.create_router() {
            Ok(router) => {
                metrics_server = metrics_server.with_external_router(router);
            }
            Err(e) => {
                tracing::error!("💥 Failed to create admin router: {}", e);
            }
        }

        // Create and merge MCP router for unified port architecture
        // MCP HTTP transport is now served from the same port as metrics/admin
        if matches!(
            transport_config.mode,
            TransportMode::Http | TransportMode::Hybrid
        ) {
            let http_state = HttpTransportState {
                server: Arc::clone(&server),
                session_manager: Arc::clone(&session_manager),
                version_checker: Arc::clone(&version_checker),
                connection_tracker: Arc::clone(&connection_tracker),
                config: transport_config.clone(),
            };

            let mcp_router = create_mcp_router(http_state);
            metrics_server = metrics_server.with_mcp_router(mcp_router);
            tracing::info!("🔗 MCP HTTP transport merged into unified server");
        }

        Some(tokio::spawn(async move {
            if let Err(e) = metrics_server.start().await {
                tracing::error!("💥 Unified HTTP server failed: {}", e);
            }
        }))
    } else {
        tracing::info!("ℹ️  HTTP server disabled");
        None
    };

    Ok((
        server,
        metrics_handle,
        resource_limits,
        http_client,
        connection_tracker,
    ))
}

/// Run the MCP Context Browser server
///
/// This is the main entry point that starts the MCP server with configurable transport.
/// The server implements the MCP protocol for semantic code search and indexing.
///
/// # Transport Modes
/// - **Stdio**: Traditional child process pattern (stdin/stdout)
/// - **HTTP**: Independent HTTP server (Streamable HTTP per MCP spec)
/// - **Hybrid**: Both stdio and HTTP simultaneously (default)
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
/// - Server version compatibility (±1 minor version)
/// - Session management with resumption support
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let log_buffer = create_shared_log_buffer(1000);
    // Initialize tracing first for proper error reporting
    init_tracing(log_buffer.clone())?;

    tracing::info!(
        "🚀 Starting MCP Context Browser v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::info!(
        "📋 System Info: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    // Initialize all server components (unified HTTP server with MCP + Admin + Metrics)
    let (server, http_handle, _resource_limits, _http_client, connection_tracker) =
        initialize_server_components(None, log_buffer).await?;

    // Get transport mode from environment variable or default
    let transport_config = {
        let mode = std::env::var("MCP__TRANSPORT__MODE")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "stdio" => Some(TransportMode::Stdio),
                "http" => Some(TransportMode::Http),
                "hybrid" => Some(TransportMode::Hybrid),
                _ => None,
            })
            .unwrap_or_default();
        TransportConfig { mode, ..Default::default() }
    };
    tracing::info!("🔧 Transport mode: {:?}", transport_config.mode);

    // Handle graceful shutdown signals
    let connection_tracker_shutdown = Arc::clone(&connection_tracker);
    let shutdown_signal = async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for shutdown signal: {}", e);
            return;
        }
        tracing::info!("🛑 Received shutdown signal, initiating graceful shutdown...");

        // Drain HTTP connections gracefully
        let drained = connection_tracker_shutdown.drain(None).await;
        if drained {
            tracing::info!("✅ All HTTP connections drained successfully");
        } else {
            tracing::warn!("⚠️ HTTP connection drain timed out");
        }
    };

    // Start the MCP service based on transport mode
    match transport_config.mode {
        TransportMode::Stdio => {
            tracing::info!("📡 Starting MCP protocol server on stdio transport");
            tracing::info!("🎯 Ready to accept MCP client connections");

            // Start the MCP service with stdio transport
            let service_future = (*server).clone().serve(stdio());

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
        }
        TransportMode::Hybrid => {
            tracing::info!("📡 Starting MCP protocol server on stdio + HTTP transport");
            tracing::info!("🎯 Ready to accept MCP client connections (stdio) and HTTP admin on port 3001");

            // Start the MCP service with stdio transport in a separate task
            let server_clone = (*server).clone();
            let stdio_handle = tokio::spawn(async move {
                match server_clone.serve(stdio()).await {
                    Ok(service) => {
                        tracing::info!("🎉 MCP stdio server started successfully");
                        let _ = service.waiting().await;
                        tracing::info!("📡 MCP stdio connection closed (HTTP server continues running)");
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ MCP stdio service ended: {:?} (HTTP server continues running)", e);
                    }
                }
            });

            // In Hybrid mode, wait for shutdown signal - don't exit when stdio closes
            // HTTP server continues running independently
            shutdown_signal.await;

            // Cancel stdio task if still running
            stdio_handle.abort();
            tracing::info!("🔄 Graceful shutdown initiated");
        }
        TransportMode::Http => {
            tracing::info!("📡 Running in HTTP-only mode (no stdio)");
            tracing::info!("🎯 Ready to accept MCP HTTP connections on unified port");

            // In HTTP-only mode, just wait for shutdown
            shutdown_signal.await;
        }
    }

    // Wait for unified HTTP server to finish if it was started
    if let Some(handle) = http_handle {
        tracing::info!("⏳ Waiting for unified HTTP server to shutdown...");
        let _ = handle.await;
        tracing::info!("✅ Unified HTTP server shutdown complete");
    }

    Ok(())
}
