//! Tests for AdminAuthConfig
//!
//! Tests admin authentication configuration and validation.

use mcb_server::admin::auth::{AdminAuthConfig, is_unauthenticated_route};

#[test]
fn test_admin_auth_config_default() {
    let config = AdminAuthConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.header_name, "X-Admin-Key");
    assert!(config.api_key.is_none());
}

#[test]
fn test_admin_auth_config_validation() {
    let config = AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_string(),
        api_key: Some("secret-key".to_string()),
    };

    assert!(config.validate_key("secret-key"));
    assert!(!config.validate_key("wrong-key"));
    assert!(config.is_configured());
}

#[test]
fn test_admin_auth_config_no_key() {
    let config = AdminAuthConfig {
        enabled: true,
        header_name: "X-Admin-Key".to_string(),
        api_key: None,
    };

    assert!(!config.validate_key("any-key"));
    assert!(!config.is_configured());
}

#[test]
fn test_is_unauthenticated_route() {
    assert!(is_unauthenticated_route("/live"));
    assert!(is_unauthenticated_route("/ready"));
    assert!(!is_unauthenticated_route("/health"));
    assert!(!is_unauthenticated_route("/config"));
    assert!(!is_unauthenticated_route("/metrics"));
    assert!(!is_unauthenticated_route("/shutdown"));
}
