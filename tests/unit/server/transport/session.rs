//! Tests for Session Manager
//!
//! Tests migrated from src/server/transport/session.rs

use mcp_context_browser::server::transport::{SessionConfig, SessionManager, SessionState};

#[test]
fn test_session_creation() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    assert!(session.id.starts_with("mcp_"));
    assert_eq!(session.state, SessionState::Initializing);
}

#[test]
fn test_session_id_format() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    // Session ID should be "mcp_" followed by UUID without dashes
    assert!(session.id.starts_with("mcp_"));
    assert!(!session.id.contains('-'));
    assert!(session.id.len() > 4); // "mcp_" + at least some UUID chars
}

#[test]
fn test_session_activation() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    manager
        .activate_session(&session.id)
        .expect("Should activate session");

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");
    assert_eq!(updated.state, SessionState::Active);
}

#[test]
fn test_session_touch() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    std::thread::sleep(std::time::Duration::from_millis(10));

    assert!(manager.touch_session(&session.id));

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");
    assert!(updated.last_activity > session.last_activity);
}

#[test]
fn test_session_touch_nonexistent() {
    let manager = SessionManager::with_defaults();

    // Touch non-existent session should return false
    assert!(!manager.touch_session("nonexistent_session"));
}

#[test]
fn test_message_buffering() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    let msg = serde_json::json!({"test": "message"});
    let event_id = manager.buffer_message(&session.id, msg.clone());

    assert!(event_id.is_some());

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");
    assert_eq!(updated.message_buffer.len(), 1);
    assert_eq!(updated.message_buffer[0].message, msg);
}

#[test]
fn test_message_buffer_limit() {
    // Create manager with small buffer
    let config = SessionConfig {
        resumption_buffer_size: 2,
        ..Default::default()
    };
    let manager = SessionManager::new(config, "0.1.0".to_string());
    let session = manager.create_session().expect("Should create session");

    // Buffer 3 messages (exceeds limit of 2)
    let msg1 = serde_json::json!({"id": 1});
    let msg2 = serde_json::json!({"id": 2});
    let msg3 = serde_json::json!({"id": 3});

    manager.buffer_message(&session.id, msg1);
    manager.buffer_message(&session.id, msg2);
    manager.buffer_message(&session.id, msg3);

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");

    // Buffer should be trimmed to 2 messages
    assert_eq!(updated.message_buffer.len(), 2);
}

#[test]
fn test_session_termination() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    assert!(manager.terminate_session(&session.id));
    assert!(manager.get_session(&session.id).is_none());
}

#[test]
fn test_session_termination_nonexistent() {
    let manager = SessionManager::with_defaults();

    // Terminating non-existent session should return false
    assert!(!manager.terminate_session("nonexistent_session"));
}

#[test]
fn test_session_suspension() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");
    manager
        .activate_session(&session.id)
        .expect("Should activate");

    assert!(manager.suspend_session(&session.id));

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");
    assert_eq!(updated.state, SessionState::Suspended);
}

#[test]
fn test_session_counts() {
    let manager = SessionManager::with_defaults();

    assert_eq!(manager.active_session_count(), 0);
    assert_eq!(manager.total_session_count(), 0);

    let session1 = manager.create_session().expect("Should create session");
    let session2 = manager.create_session().expect("Should create session");

    // Both are initializing
    assert_eq!(manager.active_session_count(), 0);
    assert_eq!(manager.total_session_count(), 2);

    // Activate session1
    manager
        .activate_session(&session1.id)
        .expect("Should activate");
    assert_eq!(manager.active_session_count(), 1);
    assert_eq!(manager.total_session_count(), 2);

    // Activate session2
    manager
        .activate_session(&session2.id)
        .expect("Should activate");
    assert_eq!(manager.active_session_count(), 2);
    assert_eq!(manager.total_session_count(), 2);
}

#[test]
fn test_set_client_info() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    let client_info = serde_json::json!({
        "name": "test-client",
        "version": "1.0.0"
    });

    assert!(manager.set_client_info(&session.id, client_info.clone()));

    let updated = manager
        .get_session(&session.id)
        .expect("Session should exist");
    assert_eq!(updated.client_info, Some(client_info));
}

#[test]
fn test_set_client_info_nonexistent() {
    let manager = SessionManager::with_defaults();

    let client_info = serde_json::json!({"name": "test"});
    assert!(!manager.set_client_info("nonexistent", client_info));
}

#[test]
fn test_get_messages_after() {
    let manager = SessionManager::with_defaults();
    let session = manager.create_session().expect("Should create session");

    // Buffer some messages
    let msg1 = serde_json::json!({"id": 1});
    let msg2 = serde_json::json!({"id": 2});
    let msg3 = serde_json::json!({"id": 3});

    let event_id1 = manager
        .buffer_message(&session.id, msg1)
        .expect("Should buffer");
    let _event_id2 = manager
        .buffer_message(&session.id, msg2)
        .expect("Should buffer");
    let _event_id3 = manager
        .buffer_message(&session.id, msg3)
        .expect("Should buffer");

    // Get messages after the first event
    let messages = manager.get_messages_after(&session.id, &event_id1);

    // Should return messages 2 and 3
    assert_eq!(messages.len(), 2);
}

#[test]
fn test_session_config_defaults() {
    let config = SessionConfig::default();

    assert_eq!(config.ttl_secs, 3600);
    assert!(config.resumption_enabled);
    assert_eq!(config.resumption_buffer_size, 100);
}

#[test]
fn test_session_config_disabled_resumption() {
    let config = SessionConfig {
        resumption_enabled: false,
        ..Default::default()
    };
    let manager = SessionManager::new(config, "0.1.0".to_string());
    let session = manager.create_session().expect("Should create session");

    // Buffer message should return None when resumption is disabled
    let msg = serde_json::json!({"test": "message"});
    let event_id = manager.buffer_message(&session.id, msg);

    assert!(event_id.is_none());
}
