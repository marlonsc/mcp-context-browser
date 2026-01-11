//! Redis cache operations
//!
//! Provides Redis-specific cache operations including connection testing,
//! get/set/delete, and queue operations.

use super::config::CacheEntry;
use super::CacheManager;
use crate::domain::error::{Error, Result};
use std::time::Duration;

impl CacheManager {
    /// Test Redis connection
    pub(crate) async fn test_redis_connection(client: &::redis::Client) -> Result<()> {
        let timeout_duration = Duration::from_secs(2);

        let mut conn =
            match tokio::time::timeout(timeout_duration, client.get_multiplexed_async_connection())
                .await
            {
                Ok(Ok(conn)) => conn,
                Ok(Err(e)) => {
                    return Err(Error::generic(format!("Redis connection failed: {}", e)));
                }
                Err(_) => return Err(Error::generic("Redis connection timed out")),
            };

        match tokio::time::timeout(
            timeout_duration,
            ::redis::cmd("PING").query_async::<()>(&mut conn),
        )
        .await
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(Error::generic(format!("Redis PING failed: {}", e))),
            Err(_) => Err(Error::generic("Redis PING timed out")),
        }
    }

    /// Get a value from Redis
    pub(crate) async fn get_from_redis(&self, key: &str) -> Result<Option<serde_json::Value>> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let data: Option<String> = ::redis::cmd("GET").arg(key).query_async(&mut conn).await?;

            if let Some(json_str) = data {
                let entry: CacheEntry<serde_json::Value> = serde_json::from_str(&json_str)?;
                return Ok(Some(entry.data));
            }
        }
        Ok(None)
    }

    /// Set a value in Redis with TTL
    pub(crate) async fn set_in_redis(
        &self,
        key: &str,
        entry: &CacheEntry<serde_json::Value>,
        ttl: u64,
    ) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(entry)?;
            ::redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    /// Delete a value from Redis
    pub(crate) async fn delete_from_redis(&self, key: &str) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            ::redis::cmd("DEL")
                .arg(key)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    /// Clear namespace in Redis using pattern matching
    pub(crate) async fn clear_namespace_redis(&self, pattern: &str) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let keys: Vec<String> = ::redis::cmd("KEYS")
                .arg(pattern)
                .query_async::<Vec<String>>(&mut conn)
                .await?;
            if !keys.is_empty() {
                ::redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<()>(&mut conn)
                    .await?;
            }
        }
        Ok(())
    }

    /// Enqueue an item to a Redis list
    pub(crate) async fn enqueue_redis(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(value)?;
            ::redis::cmd("RPUSH")
                .arg(key)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    /// Get all items from a Redis list
    pub(crate) async fn get_queue_redis(&self, key: &str) -> Result<Vec<serde_json::Value>> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let items: Vec<String> = ::redis::cmd("LRANGE")
                .arg(key)
                .arg(0)
                .arg(-1)
                .query_async(&mut conn)
                .await?;

            let mut result = Vec::new();
            for item in items {
                let val: serde_json::Value = serde_json::from_str(&item)?;
                result.push(val);
            }
            return Ok(result);
        }
        Ok(Vec::new())
    }

    /// Remove an item from a Redis list
    pub(crate) async fn remove_from_redis(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        if let Some(ref client) = self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let json_str = serde_json::to_string(value)?;
            // LREM key count value (count 0 means remove all occurrences)
            ::redis::cmd("LREM")
                .arg(key)
                .arg(0)
                .arg(json_str)
                .query_async::<()>(&mut conn)
                .await?;
        }
        Ok(())
    }
}
