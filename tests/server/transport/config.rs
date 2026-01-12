//! Tests for Transport Configuration
//!
//! Tests migrated from src/server/transport/config.rs
//!
//! Note: In unified port architecture (ADR-007), MCP HTTP is served from the
//! same port as Admin and Metrics (default: 3001, configured via MCP_PORT).

use mcp_context_browser::server::transport::{
    HttpTransportConfig, SessionConfig, TransportConfig, TransportMode, VersionConfig,
};

#[test]
fn test_transport_config_defaults() {
    let config = TransportConfig::default();
    assert_eq!(config.mode, TransportMode::Hybrid);
    assert_eq!(config.http.bind_address, "127.0.0.1");
}

#[test]
fn test_http_config_defaults() {
    let config = HttpTransportConfig::default();
    assert_eq!(config.bind_address, "127.0.0.1");
    assert!(config.sse_enabled);
    assert_eq!(config.max_sessions, 1000);
    assert_eq!(config.request_timeout_secs, 30);
}

#[test]
fn test_http_config_bind_address() {
    let config = HttpTransportConfig::default();
    assert_eq!(config.bind_address, "127.0.0.1");
}

#[test]
fn test_session_config_defaults() {
    let config = SessionConfig::default();
    assert_eq!(config.ttl_secs, 3600);
    assert!(config.resumption_enabled);
    assert_eq!(config.resumption_buffer_size, 100);
}

#[test]
fn test_version_config_defaults() {
    let config = VersionConfig::default();
    assert_eq!(config.version_tolerance, 1);
    assert!(config.warn_only);
}

#[test]
fn test_transport_mode_serde() {
    let json = r#""hybrid""#;
    let mode: TransportMode =
        serde_json::from_str(json).expect("Failed to deserialize hybrid mode");
    assert_eq!(mode, TransportMode::Hybrid);

    let json = r#""stdio""#;
    let mode: TransportMode = serde_json::from_str(json).expect("Failed to deserialize stdio mode");
    assert_eq!(mode, TransportMode::Stdio);

    let json = r#""http""#;
    let mode: TransportMode = serde_json::from_str(json).expect("Failed to deserialize http mode");
    assert_eq!(mode, TransportMode::Http);
}

#[test]
fn test_transport_mode_serialize() {
    let mode = TransportMode::Hybrid;
    let json = serde_json::to_string(&mode).expect("Failed to serialize");
    assert_eq!(json, r#""hybrid""#);

    let mode = TransportMode::Stdio;
    let json = serde_json::to_string(&mode).expect("Failed to serialize");
    assert_eq!(json, r#""stdio""#);

    let mode = TransportMode::Http;
    let json = serde_json::to_string(&mode).expect("Failed to serialize");
    assert_eq!(json, r#""http""#);
}

#[test]
fn test_transport_config_serde_roundtrip() {
    let config = TransportConfig::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: TransportConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.mode, config.mode);
    assert_eq!(deserialized.http.bind_address, config.http.bind_address);
}

#[test]
fn test_http_transport_config_custom_values() {
    let json = r#"{
        "bind_address": "0.0.0.0",
        "sse_enabled": false,
        "max_sessions": 500,
        "request_timeout_secs": 60
    }"#;

    let config: HttpTransportConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.bind_address, "0.0.0.0");
    assert!(!config.sse_enabled);
    assert_eq!(config.max_sessions, 500);
    assert_eq!(config.request_timeout_secs, 60);
}

#[test]
fn test_session_config_custom_values() {
    let json = r#"{
        "ttl_secs": 7200,
        "resumption_enabled": false,
        "resumption_buffer_size": 50
    }"#;

    let config: SessionConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.ttl_secs, 7200);
    assert!(!config.resumption_enabled);
    assert_eq!(config.resumption_buffer_size, 50);
}

#[test]
fn test_version_config_custom_values() {
    let json = r#"{
        "version_tolerance": 2,
        "warn_only": false
    }"#;

    let config: VersionConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.version_tolerance, 2);
    assert!(!config.warn_only);
}

#[test]
fn test_transport_mode_default() {
    let mode = TransportMode::default();
    assert_eq!(mode, TransportMode::Hybrid);
}

#[test]
fn test_transport_config_partial_json() {
    // Only specify mode, rest should use defaults
    let json = r#"{"mode": "stdio"}"#;
    let config: TransportConfig = serde_json::from_str(json).expect("Failed to deserialize");

    assert_eq!(config.mode, TransportMode::Stdio);
    // Check defaults are applied
    assert_eq!(config.http.bind_address, "127.0.0.1");
    assert_eq!(config.session.ttl_secs, 3600);
}
