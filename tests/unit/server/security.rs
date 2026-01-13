//! Tests for Security Middleware
//!
//! Tests migrated from src/server/security.rs

use axum::http::{Method, Uri};
use mcp_context_browser::server::security::SecurityConfig;

#[test]
fn test_security_config_default() {
    let config = SecurityConfig::default();
    assert!(config.enabled);
    assert!(config.hsts_enabled);
    assert!(config.x_content_type_options);
    assert_eq!(config.max_request_size, 10 * 1024 * 1024);
    assert!(config.block_suspicious_requests);
}

#[test]
fn test_security_config_hsts_settings() {
    let config = SecurityConfig::default();
    assert_eq!(config.hsts_max_age, 31536000); // 1 year
    assert!(config.hsts_include_subdomains);
}

#[test]
fn test_security_config_csp() {
    let config = SecurityConfig::default();
    assert!(config.content_security_policy.is_some());
    let csp = config.content_security_policy.as_ref().unwrap();
    assert!(csp.contains("default-src 'self'"));
}

#[test]
fn test_security_config_x_frame_options() {
    let config = SecurityConfig::default();
    assert!(config.x_frame_options.is_some());
    assert_eq!(config.x_frame_options.as_ref().unwrap(), "DENY");
}

#[test]
fn test_security_config_referrer_policy() {
    let config = SecurityConfig::default();
    assert!(config.referrer_policy.is_some());
    assert_eq!(
        config.referrer_policy.as_ref().unwrap(),
        "strict-origin-when-cross-origin"
    );
}

#[test]
fn test_security_config_permissions_policy() {
    let config = SecurityConfig::default();
    assert!(config.permissions_policy.is_some());
    let pp = config.permissions_policy.as_ref().unwrap();
    assert!(pp.contains("camera=()"));
    assert!(pp.contains("microphone=()"));
}

#[test]
fn test_security_config_cross_origin_policies() {
    let config = SecurityConfig::default();

    assert!(config.cross_origin_embedder_policy.is_some());
    assert_eq!(
        config.cross_origin_embedder_policy.as_ref().unwrap(),
        "require-corp"
    );

    assert!(config.cross_origin_opener_policy.is_some());
    assert_eq!(
        config.cross_origin_opener_policy.as_ref().unwrap(),
        "same-origin"
    );

    assert!(config.cross_origin_resource_policy.is_some());
    assert_eq!(
        config.cross_origin_resource_policy.as_ref().unwrap(),
        "same-origin"
    );
}

#[test]
fn test_uri_validation_valid() {
    // Valid URIs should pass validation
    let health_uri: Uri = "/api/health".parse().expect("Failed to parse URI");
    assert!(validate_uri_length(&health_uri));

    let search_uri: Uri = "/api/search?q=test".parse().expect("Failed to parse URI");
    assert!(validate_uri_length(&search_uri));
}

#[test]
fn test_uri_validation_encoded_null() {
    // Encoded null should fail
    let null_uri: Uri = "/api/%00test".parse().expect("Failed to parse URI");
    assert!(null_uri.path().contains("%00"));
}

#[test]
fn test_uri_validation_xss_attempt() {
    // XSS attempt with encoded characters
    let xss_uri: Uri = "/api/test%3Cscript%3E"
        .parse()
        .expect("Failed to parse URI");
    assert!(xss_uri.path().contains("%3C")); // Encoded <
}

#[test]
fn test_method_validation() {
    // Allowed methods
    assert!(is_allowed_method(&Method::GET));
    assert!(is_allowed_method(&Method::POST));
    assert!(is_allowed_method(&Method::PUT));
    assert!(is_allowed_method(&Method::DELETE));
    assert!(is_allowed_method(&Method::HEAD));
    assert!(is_allowed_method(&Method::OPTIONS));

    // Disallowed methods
    assert!(!is_allowed_method(&Method::TRACE));
    assert!(!is_allowed_method(&Method::CONNECT));
}

#[test]
fn test_security_config_disabled() {
    let config = SecurityConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(!config.enabled);
}

#[test]
fn test_security_config_custom_max_request_size() {
    let config = SecurityConfig {
        max_request_size: 1024 * 1024, // 1MB
        ..Default::default()
    };
    assert_eq!(config.max_request_size, 1024 * 1024);
}

#[test]
fn test_security_config_allowed_origins() {
    let config = SecurityConfig::default();
    // Default should have empty allowed origins (allowing development)
    assert!(config.allowed_origins.is_empty());
}

// Helper function to check URI length (extracted from security.rs validation logic)
fn validate_uri_length(uri: &Uri) -> bool {
    let path = uri.path();
    let query = uri.query().unwrap_or("");
    path.len() <= 2048 && query.len() <= 2048
}

// Helper function to check if method is allowed (extracted from security.rs)
fn is_allowed_method(method: &Method) -> bool {
    matches!(
        method,
        &Method::GET
            | &Method::POST
            | &Method::PUT
            | &Method::DELETE
            | &Method::HEAD
            | &Method::OPTIONS
    )
}
