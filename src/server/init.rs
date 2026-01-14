//! Server initialization utilities
//!
//! This module contains server initialization logic extracted from the main
//! server implementation to improve code organization and testability.
//!
//! # Helper Functions
//!
//! - [`init_cache_provider`] - Initialize cache provider with fallback
//! - [`init_rate_limiter`] - Initialize rate limiter with appropriate backend
//! - [`setup_admin_interface`] - Configure admin API with first-run support

use crate::adapters::providers::routing::health::HealthMonitor;
use crate::infrastructure::cache::{create_cache_provider, SharedCacheProvider};
use crate::infrastructure::config::ConfigLoader;
use crate::infrastructure::connection_tracker::{ConnectionTracker, ConnectionTrackerConfig};
use crate::infrastructure::daemon::types::RecoveryConfig;
use crate::infrastructure::di::DiContainer;
use crate::infrastructure::health::ActiveHealthMonitor;
use crate::infrastructure::limits::ResourceLimits;
use crate::infrastructure::metrics::MetricsApiServer;
use crate::infrastructure::provider_lifecycle::ProviderLifecycleManager;
use crate::infrastructure::recovery::{RecoveryManager, SharedRecoveryManager};
use crate::infrastructure::resilience::{
    create_rate_limiter, determine_rate_limiter_backend, SharedRateLimiter,
};
use crate::server::transport::{
    create_mcp_router, HttpTransportState, SessionManager, TransportConfig, TransportMode,
    VersionChecker,
};
use crate::server::McpServer;
use arc_swap::ArcSwap;
use rmcp::transport::stdio;
use rmcp::ServerHandler;
use rmcp::ServiceExt;
use std::sync::Arc;
use tracing_subscriber::{self, EnvFilter};

use crate::infrastructure::events::{create_event_bus, EventBusConfig};
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

/// Initialize cache provider with fallback to None on error
async fn init_cache_provider(
    config: &crate::infrastructure::config::Config,
) -> Option<SharedCacheProvider> {
    if !config.cache.enabled {
        tracing::info!("ℹ️  Caching disabled");
        return None;
    }

    tracing::info!("🗄️  Initializing cache provider...");
    match create_cache_provider(&config.cache).await {
        Ok(provider) => {
            tracing::info!("✅ Cache provider initialized successfully");
            Some(provider)
        }
        Err(e) => {
            tracing::warn!(
                "⚠️  Failed to initialize cache provider: {}. Running without caching.",
                e
            );
            None
        }
    }
}

/// Initialize rate limiter with appropriate backend
fn init_rate_limiter(
    config: &crate::infrastructure::config::Config,
    cache_provider: Option<SharedCacheProvider>,
) -> Option<SharedRateLimiter> {
    if !config.metrics.rate_limiting.enabled {
        tracing::info!("ℹ️  Rate limiting disabled");
        return None;
    }

    let backend_type = determine_rate_limiter_backend(
        cache_provider.as_ref(),
        config.metrics.clustering_enabled,
    );
    tracing::info!(
        backend = ?backend_type,
        "🔒 Initializing rate limiter..."
    );
    let limiter = create_rate_limiter(
        backend_type,
        config.metrics.rate_limiting.clone(),
        cache_provider,
    );
    tracing::info!("✅ Rate limiter initialized successfully");
    Some(limiter)
}

/// Setup admin interface with first-run credential support
///
/// Returns the admin router to merge into the metrics server, or None if disabled/failed.
async fn setup_admin_interface(
    config: &crate::infrastructure::config::Config,
    server: &Arc<McpServer>,
) -> Option<axum::Router> {
    let data_dir = config.data.base_path();

    let (admin_config, generated_password) = match
        crate::server::admin::config::AdminConfig::load_with_first_run(&data_dir).await
    {
        Ok(result) => result,
        Err(e) => {
            tracing::warn!(
                "⚠️  Failed to load admin config: {}. Admin interface disabled.",
                e
            );
            return None;
        }
    };

    if !admin_config.enabled {
        tracing::info!("ℹ️  Admin interface disabled");
        return None;
    }

    // Display first-run message if password was generated
    if generated_password.is_some() {
        display_first_run_message(&admin_config, &data_dir);
    }

    let admin_api_server = crate::server::admin::api::AdminApiServer::new(
        admin_config,
        Arc::clone(server),
    );

    let auth_handler = server.auth_handler();
    match admin_api_server.create_router_with_auth(auth_handler) {
        Ok(router) => {
            tracing::info!("✅ Admin interface enabled");
            Some(router)
        }
        Err(e) => {
            tracing::error!("💥 Failed to create admin router: {}", e);
            None
        }
    }
}

/// Display first-run credentials message
fn display_first_run_message(
    admin_config: &crate::server::admin::config::AdminConfig,
    data_dir: &std::path::Path,
) {
    let credentials_file = data_dir.join("users.json");
    eprintln!();
    eprintln!("========================================");
    eprintln!("  FIRST RUN - Admin credentials created");
    eprintln!("========================================");
    eprintln!("  Username: {}", admin_config.username);
    eprintln!();
    eprintln!("  Credentials saved to:");
    eprintln!("    {}", credentials_file.display());
    eprintln!();
    eprintln!("  To view your password, run:");
    eprintln!("    cat {}", credentials_file.display());
    eprintln!();
    eprintln!("  IMPORTANT: Save the password securely!");
    eprintln!("========================================");
    eprintln!();
}

/// Initialize all server components and services
///
/// Returns a tuple containing:
/// - McpServer instance
/// - Metrics/Admin/MCP unified HTTP server handle
/// - Resource limits
/// - HTTP client provider
/// - Connection tracker for graceful shutdown
/// - Recovery manager for automatic component restart
async fn initialize_server_components(
    log_buffer: crate::infrastructure::logging::SharedLogBuffer,
    config_path: Option<&std::path::Path>,
) -> Result<
    (
        Arc<McpServer>,
        Option<tokio::task::JoinHandle<()>>,
        Arc<ResourceLimits>,
        Arc<dyn crate::adapters::http_client::HttpClientProvider>,
        Arc<ConnectionTracker>,
        SharedRecoveryManager,
        Option<SharedCacheProvider>,
    ),
    Box<dyn std::error::Error>,
> {
    // Load configuration: from file if specified, otherwise from XDG paths + environment
    let loader = ConfigLoader::new();
    let config = match config_path {
        Some(path) => {
            tracing::info!("📁 Loading configuration from: {}", path.display());
            loader.load_with_file(path).await
        }
        None => {
            tracing::info!("📁 Loading configuration from XDG paths and environment");
            loader.load().await
        }
    }
    .map_err(|e| format!("Failed to load configuration: {}", e))?;

    let event_bus_config = EventBusConfig::from_env();
    let event_bus = create_event_bus(&event_bus_config)
        .await
        .map_err(|e| format!("Failed to create event bus: {}", e))?;

    // Initialize resource limits
    let resource_limits = Arc::new(crate::infrastructure::limits::ResourceLimits::new(
        config.resource_limits.clone(),
    ));

    // Build DI container for component resolution
    tracing::info!("🔧 Building DI container...");
    let container =
        DiContainer::build().map_err(|e| format!("Failed to build DI container: {}", e))?;

    // Resolve HTTP client from DI container
    tracing::info!("🌐 Resolving HTTP client pool from DI container...");
    let http_client: Arc<dyn crate::adapters::http_client::HttpClientProvider> =
        container.resolve();
    tracing::info!("✅ HTTP client pool resolved successfully");

    // Initialize database pool (not used directly, but available for DI)
    if config.database.enabled {
        tracing::info!("🗄️  Database configuration loaded (used via dependency injection)");
    } else {
        tracing::info!("ℹ️  Database disabled");
    }

    // Initialize cache provider first (needed for distributed rate limiting)
    let cache_provider = init_cache_provider(&config).await;

    // Initialize rate limiter for HTTP API (uses cache if clustering enabled)
    let rate_limiter = init_rate_limiter(&config, cache_provider.clone());

    // Clone event_bus for recovery manager and health monitor before passing to builder
    let event_bus_for_recovery = event_bus.clone();
    let event_bus_for_health = event_bus.clone();

    // Wrap config in Arc<ArcSwap> for builder (clone first since we need config later)
    let config_arc = Arc::new(ArcSwap::from_pointee(config.clone()));

    // Create server instance using builder
    let mut builder = McpServerBuilder::new()
        .with_config(config_arc)
        .with_log_buffer(log_buffer)
        .with_event_bus(event_bus)
        .with_resource_limits(resource_limits.clone())
        .with_http_client(Arc::clone(&http_client));

    builder = builder.with_cache_provider(cache_provider.clone());

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

    // Initialize Recovery Manager for automatic component restart
    tracing::info!("🔄 Initializing Recovery Manager...");
    let recovery_config = RecoveryConfig::default();
    let recovery_manager: SharedRecoveryManager = Arc::new(RecoveryManager::new(
        recovery_config,
        event_bus_for_recovery.clone(),
    ));

    // Start the recovery manager background task
    if let Err(e) = recovery_manager.start().await {
        tracing::warn!(
            "⚠️  Failed to start Recovery Manager: {}. Automatic recovery disabled.",
            e
        );
    } else {
        tracing::info!("✅ Recovery Manager started successfully");
    }

    // Initialize Active Health Monitor for proactive provider health checking
    tracing::info!("🏥 Initializing Active Health Monitor...");

    // Get registry from service provider and wrap in Arc
    let registry_for_health: Arc<crate::infrastructure::di::registry::ProviderRegistry> =
        Arc::new(server.service_provider.registry().clone());

    // Create health monitor with the registry (automatically creates RealProviderHealthChecker)
    let health_monitor = Arc::new(HealthMonitor::with_registry(registry_for_health.clone()));

    // Create registry trait object for ActiveHealthMonitor
    let registry_trait: Arc<dyn crate::infrastructure::di::registry::ProviderRegistryTrait> =
        registry_for_health;

    // Create active health monitor with default configuration
    let active_monitor = Arc::new(ActiveHealthMonitor::with_defaults(
        health_monitor,
        registry_trait,
        event_bus_for_health,
    ));

    // Start the health monitoring loop
    active_monitor.start();
    tracing::info!("✅ Active Health Monitor started successfully");

    // Initialize Provider Lifecycle Manager for handling provider restarts
    tracing::info!("🔄 Initializing Provider Lifecycle Manager...");

    // Reuse the actual registry from the server (same registry used by health monitor and rest of system)
    let registry_for_lifecycle: Arc<crate::infrastructure::di::registry::ProviderRegistry> =
        Arc::new(server.service_provider.registry().clone());
    let registry_trait_for_lifecycle: Arc<
        dyn crate::infrastructure::di::registry::ProviderRegistryTrait,
    > = registry_for_lifecycle;

    let lifecycle_manager = ProviderLifecycleManager::new(
        Arc::clone(&server.service_provider),
        registry_trait_for_lifecycle,
        event_bus_for_recovery.clone(),
    );
    lifecycle_manager.start();
    tracing::info!("✅ Provider Lifecycle Manager started successfully");

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

        let mut metrics_server = MetricsApiServer::with_limits(
            config.metrics.port,
            server.system_collector(),
            server.performance_metrics(),
            rate_limiter.clone(),
            Some(resource_limits.clone()),
            cache_provider.clone(),
        );

        // Initialize admin API server with first-run support
        if let Some(admin_router) = setup_admin_interface(&config, &server).await {
            metrics_server = metrics_server.with_external_router(admin_router);
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
        recovery_manager,
        cache_provider,
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
pub async fn run_server(
    config_path: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    // Note: Admin credential initialization with first-run support happens inside initialize_server_components
    let (
        server,
        http_handle,
        _resource_limits,
        _http_client,
        connection_tracker,
        _recovery_manager,
        _cache_provider,
    ) = initialize_server_components(log_buffer, config_path).await?;

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
        TransportConfig {
            mode,
            ..Default::default()
        }
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
                            let _ = service.waiting().await;
                            tracing::info!("👋 MCP server shutdown complete");
                        }
                        Err(e) => {
                            tracing::error!("💥 Failed to start MCP service: {:?}", e);
                            return Err(Box::new(std::io::Error::other(format!("{:?}", e))));
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
            tracing::info!(
                "🎯 Ready to accept MCP client connections (stdio) and HTTP admin on port 3001"
            );

            // Start the MCP service with stdio transport in a separate task
            let server_clone = (*server).clone();
            let stdio_handle = tokio::spawn(async move {
                match server_clone.serve(stdio()).await {
                    Ok(service) => {
                        tracing::info!("🎉 MCP stdio server started successfully");
                        let _ = service.waiting().await;
                        tracing::info!(
                            "📡 MCP stdio connection closed (HTTP server continues running)"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ MCP stdio service ended: {:?} (HTTP server continues running)",
                            e
                        );
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
