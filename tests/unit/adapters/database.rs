//! Tests for database connection pooling
//!
//! Tests for the database adapter module including connection pooling,
//! configuration, and the null database pool for testing.

use mcp_context_browser::adapters::database::{
    DatabaseConfig, DatabasePool, DatabasePoolProvider, DatabaseStats, NullDatabasePool,
};
use std::time::Duration;

#[test]
fn test_database_config_default() {
    let config = DatabaseConfig::default();
    assert_eq!(config.max_connections, 20);
    assert_eq!(config.min_idle, 5);
    assert_eq!(config.max_lifetime, Duration::from_secs(1800));
    assert_eq!(config.idle_timeout, Duration::from_secs(600));
    assert_eq!(config.connection_timeout, Duration::from_secs(30));
    assert!(!config.enabled);
}

#[test]
fn test_database_pool_disabled() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = DatabaseConfig {
        enabled: false,
        ..Default::default()
    };
    let pool = DatabasePool::new(config)?;
    assert!(!pool.is_enabled());
    assert!(pool.get_connection().is_err());
    Ok(())
}

#[test]
fn test_database_pool_disabled_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Test creating a disabled database pool (new pattern)
    let config = DatabaseConfig {
        enabled: false,
        ..Default::default()
    };
    let pool = DatabasePool::new(config)?;
    assert!(!pool.is_enabled());
    Ok(())
}

#[test]
fn test_database_stats() {
    let stats = DatabaseStats {
        connections: 10,
        idle_connections: 5,
        max_connections: 20,
        min_idle: 5,
    };
    assert_eq!(stats.connections, 10);
    assert_eq!(stats.idle_connections, 5);
    assert_eq!(stats.max_connections, 20);
    assert_eq!(stats.min_idle, 5);
}

#[test]
fn test_null_database_pool() {
    let pool = NullDatabasePool::new();
    assert!(!pool.is_enabled());
    assert!(pool.get_connection().is_err());

    let stats = pool.stats();
    assert_eq!(stats.connections, 0);
    assert_eq!(stats.idle_connections, 0);
}

#[tokio::test]
async fn test_null_database_pool_health_check() {
    let pool = NullDatabasePool::new();
    assert!(pool.health_check().await.is_err());
}
