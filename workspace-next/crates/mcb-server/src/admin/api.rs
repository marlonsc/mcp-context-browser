//! Admin API server
//!
//! HTTP server for the admin API running on a separate port.

use mcb_domain::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_infrastructure::config::watcher::ConfigWatcher;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;

use super::handlers::AdminState;
use super::routes::admin_router;

/// Admin API server configuration
#[derive(Debug, Clone)]
pub struct AdminApiConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
}

impl Default for AdminApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9090,
        }
    }
}

impl AdminApiConfig {
    /// Create config for localhost with specified port
    pub fn localhost(port: u16) -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port,
        }
    }

    /// Get the socket address
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], self.port)))
    }
}

/// Admin API server
pub struct AdminApi {
    config: AdminApiConfig,
    state: AdminState,
}

impl AdminApi {
    /// Create a new admin API server
    pub fn new(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
    ) -> Self {
        Self {
            config,
            state: AdminState {
                metrics,
                indexing,
                config_watcher: None,
                config_path: None,
                shutdown_coordinator: None,
                shutdown_timeout_secs: 30,
            },
        }
    }

    /// Create a new admin API server with configuration watcher support
    pub fn with_config_watcher(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        config_watcher: Arc<ConfigWatcher>,
        config_path: PathBuf,
    ) -> Self {
        Self {
            config,
            state: AdminState {
                metrics,
                indexing,
                config_watcher: Some(config_watcher),
                config_path: Some(config_path),
                shutdown_coordinator: None,
                shutdown_timeout_secs: 30,
            },
        }
    }

    /// Start the admin API server
    ///
    /// Returns a handle that can be used to gracefully shutdown the server.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        tracing::info!("Admin API server listening on {}", addr);

        let router = admin_router(self.state);

        axum::serve(listener, router).await?;

        Ok(())
    }

    /// Start the admin API server with graceful shutdown
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.config.socket_addr();
        let listener = TcpListener::bind(addr).await?;

        tracing::info!("Admin API server listening on {}", addr);

        let router = admin_router(self.state);

        axum::serve(listener, router)
            .with_graceful_shutdown(shutdown_signal)
            .await?;

        Ok(())
    }
}
