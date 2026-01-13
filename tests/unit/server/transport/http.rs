//! Tests for HTTP Transport Components
//!
//! Tests migrated from src/server/transport/http.rs
//! These tests focus on individual transport components rather than HttpTransportState
//! since constructing a full state requires McpServer which has complex dependencies.

use mcp_context_browser::infrastructure::connection_tracker::{
    ConnectionTracker, ConnectionTrackerConfig,
};
use mcp_context_browser::server::transport::{
    SessionManager, TransportConfig, TransportMode, VersionChecker,
};
use std::sync::Arc;

#[test]
fn test_is_initialize_request() {
    // Test via JSON-RPC method field
    let init = serde_json::json!({"method": "initialize"});
    assert!(init
        .get("method")
        .and_then(|m| m.as_str())
        .map(|m| m == "initialize")
        .unwrap_or(false));

    let other = serde_json::json!({"method": "tools/list"});
    assert!(!other
        .get("method")
        .and_then(|m| m.as_str())
        .map(|m| m == "initialize")
        .unwrap_or(false));
}

#[test]
fn test_session_manager_creation() {
    let session_manager = Arc::new(SessionManager::with_defaults());

    // Verify session manager is created correctly
    assert_eq!(session_manager.active_session_count(), 0);
    assert_eq!(session_manager.total_session_count(), 0);
}

#[test]
fn test_connection_tracker_creation() {
    let connection_tracker = Arc::new(ConnectionTracker::new(ConnectionTrackerConfig::default()));

    // Verify connection tracker is created correctly
    assert!(!connection_tracker.is_draining());
    assert_eq!(connection_tracker.active_count(), 0);
}

#[test]
fn test_version_checker_creation() {
    let version_checker = Arc::new(VersionChecker::with_defaults());

    // Version string should not be empty
    let version = version_checker.version_string();
    assert!(!version.is_empty());
}

#[test]
fn test_transport_config_defaults() {
    let config = TransportConfig::default();

    // Check default values
    assert_eq!(config.mode, TransportMode::Hybrid);
}

#[tokio::test]
async fn test_session_manager_lifecycle() {
    let session_manager = Arc::new(SessionManager::with_defaults());

    // Create a session
    let session = session_manager
        .create_session()
        .expect("Should create session");

    assert!(session.id.starts_with("mcp_"));
    assert_eq!(session_manager.total_session_count(), 1);

    // Terminate session
    assert!(session_manager.terminate_session(&session.id));
    assert_eq!(session_manager.total_session_count(), 0);
}

#[tokio::test]
async fn test_connection_tracker_lifecycle() {
    let connection_tracker = Arc::new(ConnectionTracker::new(ConnectionTrackerConfig::default()));

    // Start tracking a request
    let guard = connection_tracker
        .request_start()
        .expect("Should accept request");
    assert_eq!(connection_tracker.active_count(), 1);

    // Drop guard to end tracking
    drop(guard);
    assert_eq!(connection_tracker.active_count(), 0);
}
