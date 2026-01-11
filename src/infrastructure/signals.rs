//! Unix Signal Handlers for Graceful Server Management
//!
//! This module provides signal handling for:
//! - SIGHUP: Reload configuration
//! - SIGTERM: Graceful shutdown
//! - SIGUSR1: Trigger binary respawn
//! - SIGINT (Ctrl+C): Graceful shutdown

use crate::infrastructure::events::{SharedEventBus, SystemEvent};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, warn};

/// Signal handler configuration
#[derive(Debug, Clone)]
pub struct SignalConfig {
    /// Enable SIGHUP handling for config reload
    pub handle_sighup: bool,
    /// Enable SIGUSR1 handling for binary respawn
    pub handle_sigusr1: bool,
    /// Enable SIGTERM handling for graceful shutdown
    pub handle_sigterm: bool,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            handle_sighup: true,
            handle_sigusr1: true,
            handle_sigterm: true,
        }
    }
}

/// Signal handler that publishes events to the event bus
pub struct SignalHandler {
    event_bus: SharedEventBus,
    config: SignalConfig,
    running: Arc<AtomicBool>,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new(event_bus: SharedEventBus, config: SignalConfig) -> Self {
        Self {
            event_bus,
            config,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(event_bus: SharedEventBus) -> Self {
        Self::new(event_bus, SignalConfig::default())
    }

    /// Check if the handler is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Start listening for signals
    ///
    /// This function spawns an async task that listens for Unix signals
    /// and publishes appropriate events to the event bus.
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            warn!("Signal handler already running");
            return Ok(());
        }

        let event_bus = Arc::clone(&self.event_bus);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            if let Err(e) = run_signal_loop(event_bus, config, running).await {
                warn!("Signal handler error: {}", e);
            }
        });

        info!("Signal handlers registered (SIGHUP, SIGTERM, SIGUSR1)");
        Ok(())
    }

    /// Stop the signal handler
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Signal handler stopped");
    }
}

/// Internal signal loop
async fn run_signal_loop(
    event_bus: SharedEventBus,
    config: SignalConfig,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Create signal streams
    let mut sighup = if config.handle_sighup {
        Some(signal(SignalKind::hangup())?)
    } else {
        None
    };

    let mut sigterm = if config.handle_sigterm {
        Some(signal(SignalKind::terminate())?)
    } else {
        None
    };

    let mut sigusr1 = if config.handle_sigusr1 {
        Some(signal(SignalKind::user_defined1())?)
    } else {
        None
    };

    while running.load(Ordering::SeqCst) {
        tokio::select! {
            // Handle SIGHUP - Reload configuration
            Some(_) = async {
                match &mut sighup {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGHUP, triggering configuration reload");
                if let Err(e) = event_bus.publish(SystemEvent::Reload) {
                    warn!("Failed to publish Reload event: {}", e);
                }
            }

            // Handle SIGTERM - Graceful shutdown
            Some(_) = async {
                match &mut sigterm {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGTERM, initiating graceful shutdown");
                if let Err(e) = event_bus.publish(SystemEvent::Shutdown) {
                    warn!("Failed to publish Shutdown event: {}", e);
                }
                break;
            }

            // Handle SIGUSR1 - Binary respawn
            Some(_) = async {
                match &mut sigusr1 {
                    Some(s) => s.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                info!("Received SIGUSR1, triggering binary respawn");
                if let Err(e) = event_bus.publish(SystemEvent::Respawn) {
                    warn!("Failed to publish Respawn event: {}", e);
                }
            }

            // Check if we should stop
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Convenience function to start signal handlers with event bus
pub async fn start_signal_handlers(event_bus: SharedEventBus) -> Result<SignalHandler> {
    let handler = SignalHandler::with_defaults(event_bus);
    handler.start().await?;
    Ok(handler)
}
