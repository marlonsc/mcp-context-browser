//! Configuration Validation Tests
//!
//! Tests for configuration validation across all config types.

use mcb_infrastructure::config::data::{
    AuthConfig, CacheConfig, CacheProvider, ServerConfig, ServerSslConfig,
};
use mcb_infrastructure::config::server::{
    ServerConfigBuilder, ServerConfigPresets, ServerConfigUtils,
};
use std::path::PathBuf;

#[test]
fn test_server_config_port_validation() {
    // Port 0 is valid for random port allocation
    let random_port_config = ServerConfigBuilder::new().port(0).build();
    assert_eq!(random_port_config.network.port, 0);

    // Standard HTTP port
    let http_config = ServerConfigBuilder::new().port(80).build();
    assert_eq!(http_config.network.port, 80);

    // Standard HTTPS port
    let https_config = ServerConfigBuilder::new().port(443).build();
    assert_eq!(https_config.network.port, 443);

    // High port number (valid)
    let high_port_config = ServerConfigBuilder::new().port(65535).build();
    assert_eq!(high_port_config.network.port, 65535);

    // Verify address parsing works with different ports
    let config = ServerConfigBuilder::new()
        .host("127.0.0.1")
        .port(8080)
        .build();
    let addr = ServerConfigUtils::parse_address(&config).unwrap();
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_auth_config_jwt_secret_length() {
    // Default config generates a secure secret
    let default_auth = AuthConfig::default();
    // JWT secret should be at least 32 characters (256 bits)
    assert!(
        default_auth.jwt.secret.len() >= 32,
        "JWT secret should be at least 32 characters, got {}",
        default_auth.jwt.secret.len()
    );

    // Custom secret can be set
    let mut custom_auth = AuthConfig::default();
    custom_auth.jwt.secret = "custom_secret_at_least_32_chars_long!".to_string();
    assert_eq!(custom_auth.jwt.secret.len(), 37);

    // Expiration times should be reasonable
    assert!(default_auth.jwt.expiration_secs > 0);
    assert!(default_auth.jwt.refresh_expiration_secs > default_auth.jwt.expiration_secs);
}

#[test]
fn test_cache_config_ttl_when_enabled() {
    // When cache is enabled, TTL should be positive
    let enabled_cache = CacheConfig {
        enabled: true,
        provider: CacheProvider::Moka,
        default_ttl_secs: 300,
        max_size: 1024 * 1024,
        redis_url: None,
        redis_pool_size: 8,
        namespace: "test".to_string(),
    };
    assert!(enabled_cache.default_ttl_secs > 0);
    assert!(enabled_cache.max_size > 0);

    // Default cache config has reasonable TTL
    let default_cache = CacheConfig::default();
    assert!(
        default_cache.default_ttl_secs >= 60,
        "Default TTL should be at least 60 seconds"
    );
    assert!(
        default_cache.max_size >= 1024,
        "Default max size should be at least 1KB"
    );

    // Disabled cache still maintains valid config
    let disabled_cache = CacheConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(!disabled_cache.enabled);
    // TTL and size should still be valid even when disabled
    assert!(disabled_cache.default_ttl_secs > 0);

    // Redis provider config
    let redis_cache = CacheConfig {
        enabled: true,
        provider: CacheProvider::Redis,
        redis_url: Some("redis://localhost:6379".to_string()),
        redis_pool_size: 16,
        ..Default::default()
    };
    assert!(redis_cache.redis_url.is_some());
    assert!(redis_cache.redis_pool_size > 0);
}

#[test]
fn test_ssl_cert_required_for_https() {
    // HTTPS without SSL paths should fail validation
    let mut https_no_ssl = ServerConfig::default();
    https_no_ssl.ssl = ServerSslConfig {
        https: true,
        ssl_cert_path: None,
        ssl_key_path: None,
    };
    let result = ServerConfigUtils::validate_ssl_config(&https_no_ssl);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("certificate path is required"));

    // HTTPS with only cert path should fail
    let mut https_cert_only = ServerConfig::default();
    https_cert_only.ssl = ServerSslConfig {
        https: true,
        ssl_cert_path: Some(PathBuf::from("/path/to/cert.pem")),
        ssl_key_path: None,
    };
    let result = ServerConfigUtils::validate_ssl_config(&https_cert_only);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("key path is required"));

    // HTTP config doesn't require SSL
    let mut http_config = ServerConfig::default();
    http_config.ssl = ServerSslConfig {
        https: false,
        ssl_cert_path: None,
        ssl_key_path: None,
    };
    let result = ServerConfigUtils::validate_ssl_config(&http_config);
    assert!(result.is_ok());
}

#[test]
fn test_default_config_is_valid() {
    // Default server config should be parseable
    let server_config = ServerConfig::default();
    let addr_result = ServerConfigUtils::parse_address(&server_config);
    assert!(
        addr_result.is_ok(),
        "Default server config should have valid address"
    );

    // Default SSL config (HTTP) should be valid
    let ssl_result = ServerConfigUtils::validate_ssl_config(&server_config);
    assert!(
        ssl_result.is_ok(),
        "Default server config (HTTP) should pass SSL validation"
    );

    // Presets should produce valid configs
    let dev_config = ServerConfigPresets::development();
    assert!(ServerConfigUtils::parse_address(&dev_config).is_ok());
    assert!(ServerConfigUtils::validate_ssl_config(&dev_config).is_ok());

    let test_config = ServerConfigPresets::testing();
    assert!(ServerConfigUtils::parse_address(&test_config).is_ok());
    assert!(ServerConfigUtils::validate_ssl_config(&test_config).is_ok());

    // Production preset has HTTPS but no SSL paths - that's expected for "template"
    // Users must provide real SSL paths for production
    let prod_config = ServerConfigPresets::production();
    assert!(ServerConfigUtils::parse_address(&prod_config).is_ok());
    // Production config is a template, SSL paths must be added by user
    // So we just verify the address is valid

    // Default auth config should have valid values
    let auth_config = AuthConfig::default();
    assert!(auth_config.jwt.secret.len() >= 32);
    assert!(auth_config.jwt.expiration_secs > 0);

    // Default cache config should have valid values
    let cache_config = CacheConfig::default();
    assert!(cache_config.default_ttl_secs > 0);
    assert!(cache_config.max_size > 0);
}

#[test]
fn test_server_url_generation() {
    // HTTP URL generation
    let http_config = ServerConfigBuilder::new()
        .host("localhost")
        .port(8080)
        .https(false)
        .build();
    let url = ServerConfigUtils::get_server_url(&http_config);
    assert_eq!(url, "http://localhost:8080");

    // HTTPS URL generation
    let https_config = ServerConfigBuilder::new()
        .host("api.example.com")
        .port(443)
        .https(true)
        .build();
    let url = ServerConfigUtils::get_server_url(&https_config);
    assert_eq!(url, "https://api.example.com:443");
}

#[test]
fn test_cors_configuration() {
    // CORS disabled
    let no_cors = ServerConfigBuilder::new().cors(false, vec![]).build();
    assert!(!no_cors.cors.cors_enabled);
    assert!(no_cors.cors.cors_origins.is_empty());

    // CORS with specific origins
    let cors_config = ServerConfigBuilder::new()
        .cors(
            true,
            vec![
                "https://app.example.com".to_string(),
                "https://admin.example.com".to_string(),
            ],
        )
        .build();
    assert!(cors_config.cors.cors_enabled);
    assert_eq!(cors_config.cors.cors_origins.len(), 2);

    // Development preset has permissive CORS
    let dev_config = ServerConfigPresets::development();
    let (enabled, origins) = ServerConfigUtils::cors_settings(&dev_config);
    assert!(enabled);
    assert!(origins.contains(&"*".to_string()));
}

#[test]
fn test_timeout_configuration() {
    let config = ServerConfigBuilder::new()
        .request_timeout(120)
        .connection_timeout(30)
        .build();

    let request_timeout = ServerConfigUtils::request_timeout(&config);
    let connection_timeout = ServerConfigUtils::connection_timeout(&config);

    assert_eq!(request_timeout.as_secs(), 120);
    assert_eq!(connection_timeout.as_secs(), 30);

    // Request timeout should generally be longer than connection timeout
    assert!(request_timeout > connection_timeout);
}
