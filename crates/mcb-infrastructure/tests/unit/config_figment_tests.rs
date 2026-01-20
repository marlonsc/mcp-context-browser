//! Tests verifying Figment configuration pattern compliance (ADR-025)
//!
//! These tests ensure the configuration system adheres to ADR-025 principles:
//! - All configuration flows through Figment
//! - Environment variables use `MCP__` prefix (double underscore)
//! - No implicit fallbacks or legacy `MCB_` prefix support
//! - Fail-fast on missing required configuration
//!
//! # Running These Tests
//!
//! These tests modify environment variables and must run sequentially:
//!
//! ```bash
//! cargo test -p mcb-infrastructure --test unit config_figment -- --test-threads=1 --ignored
//! ```
//!
//! # Safety
//!
//! Tests use `unsafe` blocks for `env::set_var`/`env::remove_var` because
//! Rust 2024 edition requires this for environment variable mutations.
//! Tests MUST run with `--test-threads=1` to prevent data races.

use mcb_infrastructure::config::loader::ConfigLoader;
use std::env;

/// Helper to set env var safely
fn set_env(key: &str, value: &str) {
    // SAFETY: Tests must run with --test-threads=1
    unsafe {
        env::set_var(key, value);
    }
}

/// Helper to remove env var safely
fn remove_env(key: &str) {
    // SAFETY: Tests must run with --test-threads=1
    unsafe {
        env::remove_var(key);
    }
}

/// Helper to disable auth to avoid JWT secret validation
fn disable_auth() {
    set_env("MCP__AUTH__ENABLED", "false");
}

/// Helper cleanup for auth
fn cleanup_auth() {
    remove_env("MCP__AUTH__ENABLED");
}

/// Verify env vars with MCP__ prefix are loaded correctly
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_mcp_prefix_env_vars_loaded() {
    // Disable auth to avoid JWT validation
    disable_auth();
    set_env("MCP__PROVIDERS__EMBEDDING__PROVIDER", "test-provider");

    let config = ConfigLoader::new().load().expect("Should load config");

    // Verify the provider value was loaded from env
    assert_eq!(
        config.providers.embedding.provider,
        Some("test-provider".to_string()),
        "MCP__ prefixed env vars should be loaded by Figment"
    );

    remove_env("MCP__PROVIDERS__EMBEDDING__PROVIDER");
    cleanup_auth();
}

/// Verify old MCB_ prefix is NOT loaded (breaking change per ADR-025)
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_old_mcb_prefix_not_loaded() {
    // Disable auth to avoid JWT validation
    disable_auth();
    // Set admin key with OLD prefix (should be ignored)
    set_env("MCB_ADMIN_API_KEY", "old-key-value");

    let config = ConfigLoader::new().load().expect("Should load config");

    // Old prefix should NOT be recognized - key should be None
    assert_eq!(
        config.auth.admin.key, None,
        "Old MCB_ prefix should NOT be recognized (ADR-025 breaking change)"
    );

    remove_env("MCB_ADMIN_API_KEY");
    cleanup_auth();
}

/// Verify new MCP__ admin key IS loaded correctly
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_new_admin_key_loaded() {
    // Disable auth to avoid JWT validation
    disable_auth();
    // Set admin key with NEW prefix
    set_env("MCP__AUTH__ADMIN__KEY", "new-key-value");

    let config = ConfigLoader::new().load().expect("Should load config");

    // New prefix should be recognized
    assert_eq!(
        config.auth.admin.key,
        Some("new-key-value".to_string()),
        "MCP__AUTH__ADMIN__KEY should be loaded by Figment"
    );

    remove_env("MCP__AUTH__ADMIN__KEY");
    cleanup_auth();
}

/// Verify JWT secret validation fails when empty and auth is enabled
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_jwt_secret_required_when_auth_enabled() {
    // Enable auth but don't set JWT secret
    set_env("MCP__AUTH__ENABLED", "true");
    // Deliberately NOT setting MCP__AUTH__JWT__SECRET

    let result = ConfigLoader::new().load();

    // Should fail validation
    assert!(
        result.is_err(),
        "Config should fail validation when auth.enabled=true but JWT secret is empty"
    );

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("JWT") || err.contains("secret"),
        "Error message should mention JWT secret requirement, got: {}",
        err
    );

    remove_env("MCP__AUTH__ENABLED");
}

/// Verify watching_enabled config is loaded from Figment, not direct env::var
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_watching_enabled_via_figment() {
    // Disable auth to avoid JWT validation
    disable_auth();
    set_env("MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED", "false");

    let config = ConfigLoader::new().load().expect("Should load config");

    // Should be false (default is true)
    assert!(
        !config.system.data.sync.watching_enabled,
        "watching_enabled should be loaded from MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED"
    );

    remove_env("MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED");
    cleanup_auth();
}

/// Verify that DISABLE_CONFIG_WATCHING env var is NOT supported (legacy removal)
///
/// Run with: `cargo test -p mcb-infrastructure config_figment -- --test-threads=1 --ignored`
#[test]
#[ignore = "requires --test-threads=1 due to env var mutations"]
fn test_legacy_disable_watching_not_supported() {
    // Disable auth to avoid JWT validation
    disable_auth();
    // Set OLD env var that should be ignored
    set_env("DISABLE_CONFIG_WATCHING", "true");

    let config = ConfigLoader::new().load().expect("Should load config");

    // Old env var should be ignored, watching_enabled should be default (true)
    assert!(
        config.system.data.sync.watching_enabled,
        "DISABLE_CONFIG_WATCHING should NOT affect watching_enabled (use MCP__SYSTEM__DATA__SYNC__WATCHING_ENABLED)"
    );

    remove_env("DISABLE_CONFIG_WATCHING");
    cleanup_auth();
}

// ============================================================================
// Non-ignored tests: These don't modify env vars, so they can run in parallel
// ============================================================================

/// Verify auth is ENABLED by default and requires JWT secret (fail-fast per ADR-025)
#[test]
fn test_auth_enabled_by_default_requires_jwt_secret() {
    // Load config without any env vars set - should FAIL because JWT secret is required
    let result = ConfigLoader::new().load();

    // Config should fail validation because auth.enabled=true by default
    // but JWT secret is empty (ADR-025: fail-fast, no implicit defaults)
    assert!(
        result.is_err(),
        "Config should fail when auth.enabled=true (default) but JWT secret is empty"
    );

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("JWT") || err.contains("secret"),
        "Error should mention JWT secret requirement, got: {}",
        err
    );
}
