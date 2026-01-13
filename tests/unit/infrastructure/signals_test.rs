//! Signal handler tests
//!
//! Tests migrated from src/infrastructure/signals.rs

use mcp_context_browser::infrastructure::events::EventBus;
use mcp_context_browser::infrastructure::signals::{SignalConfig, SignalHandler};
use std::sync::Arc;

#[tokio::test]
async fn test_signal_handler_creation() {
    let event_bus = Arc::new(EventBus::default());
    let handler = SignalHandler::with_defaults(event_bus);
    assert!(!handler.is_running());
}

#[tokio::test]
async fn test_signal_config_defaults() {
    let config = SignalConfig::default();
    assert!(config.handle_sighup);
    assert!(config.handle_sigusr1);
    assert!(config.handle_sigterm);
}

#[tokio::test]
async fn test_signal_handler_start_stop() -> Result<(), Box<dyn std::error::Error>> {
    let event_bus = Arc::new(EventBus::default());
    let handler = SignalHandler::with_defaults(event_bus);

    handler.start().await?;
    assert!(handler.is_running());

    handler.stop();
    // Give the background task time to notice the stop
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    assert!(!handler.is_running());
    Ok(())
}
