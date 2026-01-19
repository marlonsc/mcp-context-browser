//! Admin API server
//!
//! HTTP server for the admin API running on a separate port.
//!
//! Migrated from Axum to Rocket in v0.1.2 (ADR-026).

use mcb_application::ports::admin::{IndexingOperationsInterface, PerformanceMetricsInterface};
use mcb_application::ports::infrastructure::EventBusProvider;
use mcb_infrastructure::config::watcher::ConfigWatcher;
use rocket::config::{Config as RocketConfig, LogLevel};
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

use super::auth::AdminAuthConfig;
use super::handlers::AdminState;
use super::routes::admin_rocket;

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

    /// Get the Rocket configuration
    pub fn rocket_config(&self) -> RocketConfig {
        let address: IpAddr = self
            .host
            .parse()
            .unwrap_or_else(|_| "127.0.0.1".parse().expect("valid IP"));
        RocketConfig {
            address,
            port: self.port,
            log_level: LogLevel::Normal,
            ..RocketConfig::default()
        }
    }
}

/// Admin API server
pub struct AdminApi {
    config: AdminApiConfig,
    state: AdminState,
    auth_config: Arc<AdminAuthConfig>,
}

impl AdminApi {
    /// Create a new admin API server
    pub fn new(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
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
                event_bus,
                service_manager: None,
                cache: None,
            },
            auth_config: Arc::new(AdminAuthConfig::default()),
        }
    }

    /// Create a new admin API server with authentication
    pub fn with_auth(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        event_bus: Arc<dyn EventBusProvider>,
        auth_config: AdminAuthConfig,
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
                event_bus,
                service_manager: None,
                cache: None,
            },
            auth_config: Arc::new(auth_config),
        }
    }

    /// Create a new admin API server with configuration watcher support
    pub fn with_config_watcher(
        config: AdminApiConfig,
        metrics: Arc<dyn PerformanceMetricsInterface>,
        indexing: Arc<dyn IndexingOperationsInterface>,
        config_watcher: Arc<ConfigWatcher>,
        config_path: PathBuf,
        event_bus: Arc<dyn EventBusProvider>,
        auth_config: AdminAuthConfig,
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
                event_bus,
                service_manager: None,
                cache: None,
            },
            auth_config: Arc::new(auth_config),
        }
    }

    /// Start the admin API server
    ///
    /// Returns a handle that can be used to gracefully shutdown the server.
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket = admin_rocket(self.state, self.auth_config).configure(rocket_config);

        rocket.launch().await.map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Rocket launch failed: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        Ok(())
    }

    /// Start the admin API server with graceful shutdown
    ///
    /// Note: Rocket handles graceful shutdown internally via Ctrl+C or SIGTERM.
    /// The shutdown_signal parameter is kept for API compatibility but Rocket
    /// manages its own shutdown lifecycle.
    pub async fn start_with_shutdown(
        self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rocket_config = self.config.rocket_config();

        tracing::info!(
            "Admin API server listening on {}:{}",
            rocket_config.address,
            rocket_config.port
        );

        let rocket = admin_rocket(self.state, self.auth_config)
            .configure(rocket_config)
            .ignite()
            .await
            .map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Rocket ignite failed: {}", e),
                )) as Box<dyn std::error::Error + Send + Sync>
            })?;

        // Spawn a task to handle the external shutdown signal
        let shutdown_handle = rocket.shutdown();
        tokio::spawn(async move {
            shutdown_signal.await;
            shutdown_handle.notify();
        });

        rocket.launch().await.map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Rocket launch failed: {}", e),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

        Ok(())
    }
}
