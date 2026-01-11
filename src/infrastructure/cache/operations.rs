//! Cache operations
//!
//! Provides the public API for cache operations including get, set, delete,
//! and queue operations.

use super::config::{CacheEntry, CacheResult};
use super::CacheManager;
use crate::domain::error::{Error, Result};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

impl CacheManager {
    /// Get a value from cache
    pub async fn get<T>(&self, namespace: &str, key: &str) -> CacheResult<T>
    where
        T: for<'de> serde::Deserialize<'de> + Clone,
    {
        if !self.config.enabled {
            return CacheResult::Miss;
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            match self.get_from_redis(&full_key).await {
                Ok(Some(data)) => match serde_json::from_value(data) {
                    Ok(deserialized) => {
                        self.stats_hits.fetch_add(1, Ordering::Relaxed);
                        return CacheResult::Hit(deserialized);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to deserialize cached data from Redis: {}", e);
                    }
                },
                Ok(None) => {} // Miss
                Err(e) => {
                    tracing::warn!("Redis get failed: {}", e);
                    // In Remote mode, we don't fallback to local to avoid inconsistency
                }
            }
            self.stats_misses.fetch_add(1, Ordering::Relaxed);
            return CacheResult::Miss;
        }

        // Local Mode (Moka)
        match self.get_from_local(&full_key).await {
            Ok(Some(data)) => match serde_json::from_value(data) {
                Ok(deserialized) => {
                    self.stats_hits.fetch_add(1, Ordering::Relaxed);
                    CacheResult::Hit(deserialized)
                }
                Err(e) => {
                    self.stats_misses.fetch_add(1, Ordering::Relaxed);
                    CacheResult::Error(Error::generic(format!(
                        "Failed to deserialize local cached data: {}",
                        e
                    )))
                }
            },
            Ok(None) => {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
                CacheResult::Miss
            }
            Err(e) => {
                self.stats_misses.fetch_add(1, Ordering::Relaxed);
                CacheResult::Error(e)
            }
        }
    }

    /// Set a value in cache
    pub async fn set<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);
        let namespace_config = self.get_namespace_config(namespace);
        let ttl = namespace_config.ttl_seconds;

        // Serialize data
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            let size_bytes = serde_json::to_string(&data).map(|s| s.len()).unwrap_or(0);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Error::generic(format!("System time error: {}", e)))?
                .as_secs();

            let entry = CacheEntry {
                data,
                created_at: now,
                accessed_at: now,
                access_count: 1,
                size_bytes,
            };

            if let Err(e) = self.set_in_redis(&full_key, &entry, ttl).await {
                tracing::warn!("Redis set failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.set_in_local(&full_key, data).await?;

        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.delete_from_redis(&full_key).await {
                tracing::warn!("Redis delete failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.delete_from_local(&full_key).await?;

        Ok(())
    }

    /// Enqueue an item to a list in cache
    pub async fn enqueue_item<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Serialize data
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.enqueue_redis(&full_key, &data).await {
                tracing::warn!("Redis enqueue failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.enqueue_local(&full_key, data).await?;

        Ok(())
    }

    /// Get all items from a queue in cache
    pub async fn get_queue<T>(&self, namespace: &str, key: &str) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de> + Clone + Send + Sync,
    {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let full_key = format!("{}:{}", namespace, key);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            match self.get_queue_redis(&full_key).await {
                Ok(items) => {
                    let mut result = Vec::new();
                    for item in items {
                        match serde_json::from_value(item) {
                            Ok(deserialized) => result.push(deserialized),
                            Err(e) => tracing::warn!("Failed to deserialize queue item: {}", e),
                        }
                    }
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Redis get_queue failed: {}", e);
                    return Ok(Vec::new()); // Return empty on error to avoid breaking flow
                }
            }
        }

        // Local Mode (Moka)
        if let Some(data_array) = self.get_from_local(&full_key).await? {
            if let Some(array) = data_array.as_array() {
                let mut result = Vec::new();
                for item in array {
                    match serde_json::from_value(item.clone()) {
                        Ok(deserialized) => result.push(deserialized),
                        Err(e) => tracing::warn!("Failed to deserialize local queue item: {}", e),
                    }
                }
                return Ok(result);
            }
        }

        Ok(Vec::new())
    }

    /// Remove an item from the queue
    pub async fn remove_item<T>(&self, namespace: &str, key: &str, value: T) -> Result<()>
    where
        T: serde::Serialize + Clone + Send + Sync + 'static,
    {
        if !self.config.enabled {
            return Ok(());
        }

        let full_key = format!("{}:{}", namespace, key);
        let data = serde_json::to_value(&value)
            .map_err(|e| Error::generic(format!("Serialization failed: {}", e)))?;

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.remove_from_redis(&full_key, &data).await {
                tracing::warn!("Redis remove_item failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.remove_from_local(&full_key, &data).await?;

        Ok(())
    }

    /// Clear entire namespace
    pub async fn clear_namespace(&self, namespace: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let pattern = format!("{}:*", namespace);

        // Remote Mode (Redis)
        if self.redis_client.is_some() {
            if let Err(e) = self.clear_namespace_redis(&pattern).await {
                tracing::warn!("Redis namespace clear failed: {}", e);
            }
            return Ok(());
        }

        // Local Mode (Moka)
        self.clear_namespace_local(namespace).await?;

        Ok(())
    }
}
