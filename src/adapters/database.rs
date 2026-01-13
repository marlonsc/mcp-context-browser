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

use crate::domain::error::{Error, Result};
use crate::infrastructure::constants::{
    DB_CONNECTION_IDLE_TIMEOUT, DB_CONNECTION_MAX_LIFETIME, DB_CONNECTION_TIMEOUT,
    DB_MAX_CONNECTIONS, DB_MIN_IDLE,
};
use async_trait::async_trait;
use r2d2::Pool;
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use validator::Validate;

/// Helper module for Duration serialization/deserialization
/// Converts between Duration and integer seconds in TOML
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    /// Serialize Duration as integer seconds
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    /// Deserialize Duration from integer seconds
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

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
    /// PostgreSQL connection URL (empty when database disabled)
    #[serde(default)]
    pub url: String,
    /// Maximum number of connections in the pool
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_connections: u32,
    /// Minimum number of idle connections
    #[serde(default)]
    pub min_idle: u32,
    /// Maximum lifetime of a connection (in seconds)
    #[serde(default, with = "duration_secs")]
    pub max_lifetime: Duration,
    /// Maximum idle time for a connection (in seconds)
    #[serde(default, with = "duration_secs")]
    pub idle_timeout: Duration,
    /// Connection timeout (in seconds)
    #[serde(default, with = "duration_secs")]
    pub connection_timeout: Duration,
    /// Whether database is enabled
    #[serde(default)]
    pub enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(), // Empty - must load from environment
            max_connections: DB_MAX_CONNECTIONS,
            min_idle: DB_MIN_IDLE,
            max_lifetime: DB_CONNECTION_MAX_LIFETIME,
            idle_timeout: DB_CONNECTION_IDLE_TIMEOUT,
            connection_timeout: DB_CONNECTION_TIMEOUT,
            enabled: false, // Disabled by default - enable via DATABASE_URL
        }
    }
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    ///
    /// # Environment Variables
    /// - `DATABASE_URL` - PostgreSQL connection string (required if database enabled)
    /// - `DATABASE_MAX_CONNECTIONS` - Max pool size (default: 20)
    /// - `DATABASE_MIN_IDLE` - Min idle connections (default: 5)
    /// - `DATABASE_MAX_LIFETIME_SECS` - Max connection lifetime in seconds (default: 1800)
    /// - `DATABASE_IDLE_TIMEOUT_SECS` - Idle timeout in seconds (default: 600)
    /// - `DATABASE_CONNECTION_TIMEOUT_SECS` - Connection timeout in seconds (default: 30)
    ///
    /// # Returns
    /// - If DATABASE_URL is set: Enabled config with connection parameters
    /// - If DATABASE_URL is not set: Disabled config (graceful degradation)
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").unwrap_or_default();
        let enabled = !url.is_empty();

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DB_MAX_CONNECTIONS)
            .max(1); // Ensure at least 1 connection

        let min_idle = std::env::var("DATABASE_MIN_IDLE")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DB_MIN_IDLE);

        let max_lifetime_secs = std::env::var("DATABASE_MAX_LIFETIME_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DB_CONNECTION_MAX_LIFETIME.as_secs());

        let idle_timeout_secs = std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DB_CONNECTION_IDLE_TIMEOUT.as_secs());

        let connection_timeout_secs = std::env::var("DATABASE_CONNECTION_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DB_CONNECTION_TIMEOUT.as_secs());

        Self {
            url,
            max_connections,
            min_idle,
            max_lifetime: Duration::from_secs(max_lifetime_secs),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
            connection_timeout: Duration::from_secs(connection_timeout_secs),
            enabled,
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
