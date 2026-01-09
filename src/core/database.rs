//! Database connection pooling for PostgreSQL
//!
//! Provides connection pooling with r2d2 for efficient database access.
//! Supports health checks, metrics, and graceful shutdown.
//!
//! ## Architecture
//!
//! This module uses dependency injection via the `DatabasePoolProvider` trait
//! to enable testability and flexibility. Pass `Arc<dyn DatabasePoolProvider>`
//! through constructors instead of using global state.

use crate::core::error::{Error, Result};
use async_trait::async_trait;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, postgres::NoTls};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use validator::Validate;

/// Trait for database pool operations (enables DI and testing)
#[async_trait]
pub trait DatabasePoolProvider: Send + Sync {
    /// Get a connection from the pool
    fn get_connection(&self) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>>;

    /// Execute a health check
    async fn health_check(&self) -> Result<()>;

    /// Get pool statistics
    fn stats(&self) -> DatabaseStats;

    /// Check if database is enabled
    fn is_enabled(&self) -> bool;

    /// Get configuration
    fn config(&self) -> &DatabaseConfig;
}

/// Type alias for shared database pool provider
pub type SharedDatabasePool = Arc<dyn DatabasePoolProvider>;

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    #[validate(length(min = 1))]
    pub url: String,
    /// Maximum number of connections in the pool
    #[validate(range(min = 1))]
    pub max_connections: u32,
    /// Minimum number of idle connections
    pub min_idle: u32,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// Maximum idle time for a connection
    pub idle_timeout: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Whether database is enabled
    pub enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://user:password@localhost:5432/mcp_context".to_string(),
            max_connections: 20,
            min_idle: 5,
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            idle_timeout: Duration::from_secs(600),  // 10 minutes
            connection_timeout: Duration::from_secs(30),
            enabled: false, // Disabled by default
        }
    }
}

/// Database connection pool
#[derive(Clone)]
pub struct DatabasePool {
    pool: Option<Pool<PostgresConnectionManager<NoTls>>>,
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        if !config.enabled {
            return Ok(Self { pool: None, config });
        }

        let manager = PostgresConnectionManager::new(
            config
                .url
                .parse()
                .map_err(|e| Error::generic(format!("Invalid database URL: {}", e)))?,
            NoTls,
        );

        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_idle))
            .max_lifetime(Some(config.max_lifetime))
            .idle_timeout(Some(config.idle_timeout))
            .connection_timeout(config.connection_timeout)
            .build(manager)
            .map_err(|e| Error::generic(format!("Failed to create connection pool: {}", e)))?;

        Ok(Self {
            pool: Some(pool),
            config,
        })
    }

    /// Get a connection from the pool
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| Error::generic("Database is disabled"))?;

        pool.get()
            .map_err(|e| Error::generic(format!("Failed to get database connection: {}", e)))
    }

    /// Execute a health check
    pub async fn health_check(&self) -> Result<()> {
        if !self.config.enabled {
            return Err(Error::generic("Database is disabled"));
        }

        let mut conn = self.get_connection()?;
        conn.execute("SELECT 1", &[])
            .map_err(|e| Error::generic(format!("Database health check failed: {}", e)))?;
        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> DatabaseStats {
        match &self.pool {
            Some(pool) => {
                let state = pool.state();
                DatabaseStats {
                    connections: state.connections,
                    idle_connections: state.idle_connections,
                    max_connections: self.config.max_connections,
                    min_idle: self.config.min_idle,
                }
            }
            None => DatabaseStats {
                connections: 0,
                idle_connections: 0,
                max_connections: self.config.max_connections,
                min_idle: self.config.min_idle,
            },
        }
    }

    /// Check if database is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }
}

/// Implement the provider trait for DatabasePool
#[async_trait]
impl DatabasePoolProvider for DatabasePool {
    fn get_connection(&self) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        DatabasePool::get_connection(self)
    }

    async fn health_check(&self) -> Result<()> {
        DatabasePool::health_check(self).await
    }

    fn stats(&self) -> DatabaseStats {
        DatabasePool::stats(self)
    }

    fn is_enabled(&self) -> bool {
        DatabasePool::is_enabled(self)
    }

    fn config(&self) -> &DatabaseConfig {
        DatabasePool::config(self)
    }
}

/// Null database pool for testing (always disabled)
#[derive(Clone, Default)]
pub struct NullDatabasePool {
    config: DatabaseConfig,
}

impl NullDatabasePool {
    /// Create a new null database pool for testing
    pub fn new() -> Self {
        Self {
            config: DatabaseConfig {
                enabled: false,
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl DatabasePoolProvider for NullDatabasePool {
    fn get_connection(&self) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        Err(Error::generic("NullDatabasePool: database is disabled"))
    }

    async fn health_check(&self) -> Result<()> {
        Err(Error::generic("NullDatabasePool: database is disabled"))
    }

    fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            connections: 0,
            idle_connections: 0,
            max_connections: 0,
            min_idle: 0,
        }
    }

    fn is_enabled(&self) -> bool {
        false
    }

    fn config(&self) -> &DatabaseConfig {
        &self.config
    }
}

/// Database pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total connections in the pool
    pub connections: u32,
    /// Idle connections in the pool
    pub idle_connections: u32,
    /// Maximum connections allowed
    pub max_connections: u32,
    /// Minimum idle connections
    pub min_idle: u32,
}

/// Global database pool instance
///
/// **DEPRECATED**: Use dependency injection with `Arc<dyn DatabasePoolProvider>` instead.
/// This global static will be removed in a future release.
static DB_POOL: std::sync::OnceLock<DatabasePool> = std::sync::OnceLock::new();

/// Initialize the global database pool
///
/// **DEPRECATED**: Use `DatabasePool::new()` and pass via DI instead.
#[deprecated(
    since = "0.0.5",
    note = "Use DatabasePool::new() and pass Arc<dyn DatabasePoolProvider> via dependency injection"
)]
pub fn init_global_database_pool(config: DatabaseConfig) -> Result<()> {
    let pool = DatabasePool::new(config)?;
    DB_POOL
        .set(pool)
        .map_err(|_| "Database pool already initialized".into())
}

/// Get the global database pool
///
/// **DEPRECATED**: Use dependency injection with `Arc<dyn DatabasePoolProvider>` instead.
#[deprecated(
    since = "0.0.5",
    note = "Use Arc<dyn DatabasePoolProvider> via dependency injection"
)]
pub fn get_global_database_pool() -> Option<&'static DatabasePool> {
    DB_POOL.get()
}

/// Get the global database pool or create a default one
///
/// **DEPRECATED**: Use `DatabasePool::new()` or `NullDatabasePool::new()` and pass via DI instead.
#[deprecated(
    since = "0.0.5",
    note = "Use DatabasePool::new() or NullDatabasePool::new() and pass via DI"
)]
pub fn get_or_create_global_database_pool() -> Result<&'static DatabasePool> {
    // Use DB_POOL.get() directly instead of calling deprecated get_global_database_pool()
    if let Some(pool) = DB_POOL.get() {
        Ok(pool)
    } else {
        // Create disabled pool if not configured
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };
        let _pool = DatabasePool::new(config)?;
        DB_POOL
            .set(_pool)
            .map_err(|_| Error::internal("Database pool already initialized"))?;
        DB_POOL
            .get()
            .ok_or_else(|| Error::internal("Failed to retrieve database pool after initialization"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_database_pool_disabled() {
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };
        let pool = DatabasePool::new(config).expect("Should be able to create disabled pool");
        assert!(!pool.is_enabled());
        assert!(pool.get_connection().is_err());
    }

    #[test]
    fn test_database_pool_disabled_creation() {
        // Test creating a disabled database pool (new pattern)
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };
        let pool_result = DatabasePool::new(config);
        assert!(
            pool_result.is_ok(),
            "Disabled pool creation failed: {:?}",
            pool_result.err()
        );
        let pool = pool_result.unwrap();
        assert!(!pool.is_enabled());
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
}
