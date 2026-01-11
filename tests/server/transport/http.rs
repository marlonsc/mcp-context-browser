//! Tests for HTTP Transport
//!
//! Tests migrated from src/server/transport/http.rs

use mcp_context_browser::infrastructure::connection_tracker::{
    ConnectionTracker, ConnectionTrackerConfig,
};
use mcp_context_browser::server::transport::{
    HttpTransportState, SessionManager, TransportConfig, VersionChecker,
};
use std::sync::Arc;

fn create_test_state() -> HttpTransportState {
    HttpTransportState {
        session_manager: Arc::new(SessionManager::with_defaults()),
        version_checker: Arc::new(VersionChecker::with_defaults()),
        connection_tracker: Arc::new(ConnectionTracker::new(ConnectionTrackerConfig::default())),
        config: TransportConfig::default(),
    }
}

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
fn test_http_transport_state_creation() {
    let state = create_test_state();

    // Verify components are created correctly
    assert_eq!(state.session_manager.active_session_count(), 0);
    assert_eq!(state.session_manager.total_session_count(), 0);
    assert!(!state.connection_tracker.is_draining());
    assert_eq!(state.connection_tracker.active_count(), 0);
}

#[test]
fn test_version_checker_in_transport() {
    let state = create_test_state();

    // Version string should not be empty
    let version = state.version_checker.version_string();
    assert!(!version.is_empty());
}

#[test]
fn test_transport_config_defaults() {
    let config = TransportConfig::default();

    // Check default values
    assert_eq!(
        config.mode,
        mcp_context_browser::server::transport::TransportMode::Hybrid
    );
}

#[tokio::test]
async fn test_session_manager_in_transport() {
    let state = create_test_state();

    // Create a session through the session manager
    let session = state
        .session_manager
        .create_session()
        .expect("Should create session");

    assert!(session.id.starts_with("mcp_"));
    assert_eq!(state.session_manager.total_session_count(), 1);

    // Terminate session
    assert!(state.session_manager.terminate_session(&session.id));
    assert_eq!(state.session_manager.total_session_count(), 0);
}

#[tokio::test]
async fn test_connection_tracker_in_transport() {
    let state = create_test_state();

    // Start tracking a request
    let guard = state
        .connection_tracker
        .request_start()
        .expect("Should accept request");
    assert_eq!(state.connection_tracker.active_count(), 1);

    // Drop guard to end tracking
    drop(guard);
    assert_eq!(state.connection_tracker.active_count(), 0);
}
