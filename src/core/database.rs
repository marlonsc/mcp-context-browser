//! Database connection pooling for PostgreSQL
//!
//! Provides connection pooling with r2d2 for efficient database access.
//! Supports health checks, metrics, and graceful shutdown.

use crate::core::error::{Error, Result};
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, postgres::NoTls};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
    /// Maximum number of connections in the pool
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
    pool: Pool<PostgresConnectionManager<NoTls>>,
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Create a new database connection pool
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        if !config.enabled {
            return Err(Error::generic("Database is disabled"));
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

        Ok(Self { pool, config })
    }

    /// Get a connection from the pool
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<PostgresConnectionManager<NoTls>>> {
        self.pool
            .get()
            .map_err(|e| Error::generic(format!("Failed to get database connection: {}", e)))
    }

    /// Execute a health check
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.get_connection()?;
        conn.execute("SELECT 1", &[])
            .map_err(|e| Error::generic(format!("Database health check failed: {}", e)))?;
        Ok(())
    }

    /// Get pool statistics
    pub fn stats(&self) -> DatabaseStats {
        let state = self.pool.state();
        DatabaseStats {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_connections: self.config.max_connections,
            min_idle: self.config.min_idle,
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
static DB_POOL: std::sync::OnceLock<DatabasePool> = std::sync::OnceLock::new();

/// Initialize the global database pool
pub fn init_global_database_pool(config: DatabaseConfig) -> Result<()> {
    let pool = DatabasePool::new(config)?;
    DB_POOL
        .set(pool)
        .map_err(|_| "Database pool already initialized".into())
}

/// Get the global database pool
pub fn get_global_database_pool() -> Option<&'static DatabasePool> {
    DB_POOL.get()
}

/// Get the global database pool or create a default one
pub fn get_or_create_global_database_pool() -> Result<&'static DatabasePool> {
    if let Some(pool) = get_global_database_pool() {
        Ok(pool)
    } else {
        // Create disabled pool if not configured
        let config = DatabaseConfig {
            enabled: false,
            ..Default::default()
        };
        let pool = DatabasePool::new(config)?;
        DB_POOL
            .set(pool)
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
        assert!(DatabasePool::new(config).is_err());
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
}
