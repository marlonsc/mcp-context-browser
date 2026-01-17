//! Configuration propagation for runtime config changes
//!
//! Handles propagating configuration changes to services that support hot-reload.
//! Uses the ConfigWatcher event subscription mechanism.

use mcb_infrastructure::config::watcher::{ConfigWatchEvent, ConfigWatcher};
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tracing::{debug, error, info, warn};

/// Configuration change handler callback type
pub type ConfigChangeCallback = Box<dyn Fn(&mcb_infrastructure::config::AppConfig) + Send + Sync>;

/// Configuration propagator that handles runtime config changes
pub struct ConfigPropagator {
    callbacks: Vec<ConfigChangeCallback>,
}

impl ConfigPropagator {
    /// Create a new config propagator
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    /// Register a callback to be called when config changes
    pub fn on_config_change(mut self, callback: ConfigChangeCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    /// Start listening for config changes from the watcher
    ///
    /// This spawns a background task that processes config change events.
    /// Returns immediately after spawning the task.
    pub fn start(self, config_watcher: Arc<ConfigWatcher>) -> PropagatorHandle {
        let receiver = config_watcher.subscribe();
        let callbacks = Arc::new(self.callbacks);

        let handle = tokio::spawn(Self::run_event_loop(receiver, callbacks));

        PropagatorHandle { handle }
    }

    /// Run the config change event loop
    async fn run_event_loop(
        mut receiver: Receiver<ConfigWatchEvent>,
        callbacks: Arc<Vec<ConfigChangeCallback>>,
    ) {
        info!("Config propagator started, listening for config changes");

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    Self::handle_event(&event, &callbacks);
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                    warn!(
                        count = count,
                        "Config propagator lagged, skipped {} events", count
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    info!("Config watcher channel closed, stopping propagator");
                    break;
                }
            }
        }

        info!("Config propagator stopped");
    }

    /// Handle a single config watch event
    fn handle_event(event: &ConfigWatchEvent, callbacks: &[ConfigChangeCallback]) {
        match event {
            ConfigWatchEvent::Reloaded(config) => {
                info!("Configuration reloaded, propagating to {} listeners", callbacks.len());
                debug!(
                    transport_mode = ?config.server.transport_mode,
                    cache_enabled = config.system.infrastructure.cache.enabled,
                    "New configuration applied"
                );

                // Call all registered callbacks
                for callback in callbacks {
                    callback(config);
                }

                info!("Configuration propagation complete");
            }
            ConfigWatchEvent::ReloadFailed(error) => {
                error!(error = %error, "Configuration reload failed");
            }
            ConfigWatchEvent::Started => {
                debug!("Config watcher started");
            }
            ConfigWatchEvent::Stopped => {
                debug!("Config watcher stopped");
            }
        }
    }
}

impl Default for ConfigPropagator {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to the running config propagator task
pub struct PropagatorHandle {
    handle: tokio::task::JoinHandle<()>,
}

impl PropagatorHandle {
    /// Wait for the propagator task to complete
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.handle.await
    }

    /// Abort the propagator task
    pub fn abort(self) {
        self.handle.abort();
    }

    /// Check if the propagator task is still running
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }
}

/// Pre-built config change callbacks for common services
pub mod callbacks {
    use super::ConfigChangeCallback;
    use tracing::info;

    /// Create a callback that logs all config changes
    pub fn logging_callback() -> ConfigChangeCallback {
        Box::new(|config| {
            info!(
                transport_mode = ?config.server.transport_mode,
                http_port = config.server.network.port,
                admin_port = config.server.network.admin_port,
                cache_enabled = config.system.infrastructure.cache.enabled,
                "Configuration change logged"
            );
        })
    }

    /// Create a callback that updates logging level (if supported)
    pub fn log_level_callback() -> ConfigChangeCallback {
        Box::new(|config| {
            // Note: Changing log level at runtime requires tracing_subscriber reload
            // which is not straightforward. This logs the new level for awareness.
            info!(
                log_level = config.logging.level,
                "Log level configuration changed (requires restart to take effect)"
            );
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_config_propagator_creation() {
        let propagator = ConfigPropagator::new();
        assert!(propagator.callbacks.is_empty());
    }

    #[tokio::test]
    async fn test_config_propagator_with_callback() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let propagator = ConfigPropagator::new().on_config_change(Box::new(move |_config| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        assert_eq!(propagator.callbacks.len(), 1);
    }

    #[test]
    fn test_propagator_handle_is_running() {
        // Test that the handle properly tracks task state
        let handle = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let handle = tokio::spawn(async {
                    // Simulate some work
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                });
                PropagatorHandle { handle }
            });

        // The task should either be running or have completed
        // We just verify the method doesn't panic
        let _ = handle.is_running();
    }
}
