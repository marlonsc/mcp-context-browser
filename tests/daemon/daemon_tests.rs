//! Tests for background daemon functionality
//!
//! Migrated from src/daemon/mod.rs inline tests.
//! Tests daemon configuration, creation, and lifecycle.

use mcp_context_browser::daemon::{ContextDaemon, DaemonConfig};

#[test]
fn test_daemon_config_default() {
    let config = DaemonConfig::default();
    assert_eq!(config.cleanup_interval_secs, 30);
    assert_eq!(config.monitoring_interval_secs, 30);
    assert_eq!(config.max_lock_age_secs, 300);
}

#[test]
fn test_daemon_config_from_env() {
    // Test with default values since no env vars are set
    let config = DaemonConfig::from_env();
    assert_eq!(config.cleanup_interval_secs, 30);
    assert_eq!(config.monitoring_interval_secs, 30);
    assert_eq!(config.max_lock_age_secs, 300);
}

#[tokio::test]
async fn test_daemon_creation() {
    let daemon = ContextDaemon::new();
    assert!(!daemon.is_running().await);

    let stats = daemon.get_stats().await;
    assert_eq!(stats.cleanup_cycles, 0);
    assert_eq!(stats.locks_cleaned, 0);
    assert_eq!(stats.monitoring_cycles, 0);
    assert_eq!(stats.active_locks, 0);
}

#[tokio::test]
async fn test_daemon_stop_before_start() {
    let daemon = ContextDaemon::new();
    assert!(daemon.stop().await.is_ok());
    assert!(!daemon.is_running().await);
}

#[tokio::test]
async fn test_daemon_with_custom_config() {
    let config = DaemonConfig {
        cleanup_interval_secs: 60,
        monitoring_interval_secs: 120,
        max_lock_age_secs: 600,
    };

    let daemon = ContextDaemon::with_config(config, None);
    let daemon_config = daemon.config();

    assert_eq!(daemon_config.cleanup_interval_secs, 60);
    assert_eq!(daemon_config.monitoring_interval_secs, 120);
    assert_eq!(daemon_config.max_lock_age_secs, 600);
}

#[tokio::test]
async fn test_daemon_stats_initial_values() {
    let daemon = ContextDaemon::new();
    let stats = daemon.get_stats().await;

    assert_eq!(stats.cleanup_cycles, 0);
    assert_eq!(stats.locks_cleaned, 0);
    assert_eq!(stats.monitoring_cycles, 0);
    assert_eq!(stats.active_locks, 0);
    assert!(stats.last_cleanup.is_none());
    assert!(stats.last_monitoring.is_none());
}

#[test]
fn test_daemon_config_validation_minimum_intervals() {
    // Test that config accepts minimum valid values
    let config = DaemonConfig {
        cleanup_interval_secs: 1,
        monitoring_interval_secs: 1,
        max_lock_age_secs: 1,
    };

    assert_eq!(config.cleanup_interval_secs, 1);
    assert_eq!(config.monitoring_interval_secs, 1);
    assert_eq!(config.max_lock_age_secs, 1);
}

#[tokio::test]
async fn test_daemon_default_implementation() {
    let daemon = ContextDaemon::default();
    assert!(!daemon.is_running().await);

    let config = daemon.config();
    assert_eq!(config.cleanup_interval_secs, 30);
}
