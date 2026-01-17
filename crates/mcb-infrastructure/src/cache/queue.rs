//! Cache operation queuing and batching
//!
//! Provides queuing and batching capabilities for cache operations
//! to improve performance and reduce network overhead.

use crate::cache::provider::SharedCacheProvider;
use mcb_application::ports::providers::cache::CacheEntryConfig;
use mcb_domain::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache operation types
#[derive(Debug, Clone)]
pub enum CacheOperation<K, V> {
    Set(K, V, CacheEntryConfig),
    Delete(K),
}

/// Type alias for the operations queue to reduce complexity
type OperationsQueue = Arc<RwLock<Vec<CacheOperation<String, Vec<u8>>>>>;

/// Batch cache operations
#[derive(Clone)]
pub struct CacheBatchProcessor {
    provider: SharedCacheProvider,
    operations: OperationsQueue,
    max_batch_size: usize,
}

impl CacheBatchProcessor {
    /// Create a new batch processor
    pub fn new(provider: SharedCacheProvider, max_batch_size: usize) -> Self {
        Self {
            provider,
            operations: Arc::new(RwLock::new(Vec::new())),
            max_batch_size,
        }
    }

    /// Queue a set operation
    pub async fn queue_set<K, V>(&self, key: K, value: V, config: CacheEntryConfig) -> Result<()>
    where
        K: Into<String>,
        V: Serialize,
    {
        let key_str = key.into();
        let value_bytes =
            serde_json::to_vec(&value).map_err(|e| mcb_domain::error::Error::Cache {
                message: format!("Failed to serialize value for batch operation: {}", e),
            })?;

        let mut operations = self.operations.write().await;
        operations.push(CacheOperation::Set(key_str, value_bytes, config));

        // Auto-flush if batch is full
        if operations.len() >= self.max_batch_size {
            let ops = operations.drain(..).collect::<Vec<_>>();
            drop(operations); // Release lock before processing
            self.process_batch(ops).await?;
        }

        Ok(())
    }

    /// Queue a delete operation
    pub async fn queue_delete<K>(&self, key: K) -> Result<()>
    where
        K: Into<String>,
    {
        let key_str = key.into();

        let mut operations = self.operations.write().await;
        operations.push(CacheOperation::Delete(key_str));

        // Auto-flush if batch is full
        if operations.len() >= self.max_batch_size {
            let ops = operations.drain(..).collect::<Vec<_>>();
            drop(operations); // Release lock before processing
            self.process_batch(ops).await?;
        }

        Ok(())
    }

    /// Flush all queued operations
    pub async fn flush(&self) -> Result<()> {
        let operations = {
            let mut ops = self.operations.write().await;
            ops.drain(..).collect::<Vec<_>>()
        };

        if !operations.is_empty() {
            self.process_batch(operations).await?;
        }

        Ok(())
    }

    /// Get the number of queued operations
    pub async fn queued_count(&self) -> usize {
        self.operations.read().await.len()
    }

    /// Process a batch of operations
    async fn process_batch(&self, operations: Vec<CacheOperation<String, Vec<u8>>>) -> Result<()> {
        if operations.is_empty() {
            return Ok(());
        }

        // Group operations by type for optimization
        let mut sets = HashMap::new();
        let mut deletes = Vec::new();

        for op in operations {
            match op {
                CacheOperation::Set(key, value, config) => {
                    sets.insert(key, (value, config));
                }
                CacheOperation::Delete(key) => {
                    deletes.push(key);
                }
            }
        }

        // Process deletes first (to avoid conflicts)
        for key in deletes {
            self.provider.delete(&key).await?;
        }

        // Process sets
        for (key, (value, config)) in sets {
            let json_str =
                String::from_utf8(value).map_err(|e| mcb_domain::error::Error::Infrastructure {
                    message: format!("Invalid UTF-8 in cached value: {}", e),
                    source: Some(Box::new(e)),
                })?;
            self.provider.set_json(&key, &json_str, config).await?;
        }

        Ok(())
    }
}

/// Cache operation result
#[derive(Debug, Clone)]
pub struct CacheOperationResult<T> {
    /// The result value
    pub value: T,
    /// Whether the result came from cache (hit) or was computed (miss)
    pub from_cache: bool,
    /// Operation duration
    pub duration: std::time::Duration,
}

/// Cache-aside pattern helper
pub struct CacheAsideHelper {
    cache: SharedCacheProvider,
}

impl CacheAsideHelper {
    /// Create a new cache-aside helper
    pub fn new(cache: SharedCacheProvider) -> Self {
        Self { cache }
    }

    /// Get or compute a value using cache-aside pattern
    pub async fn get_or_compute<F, V, Fut>(
        &self,
        key: &str,
        compute_fn: F,
    ) -> Result<CacheOperationResult<V>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
        V: Serialize + DeserializeOwned + Clone + Send + Sync,
    {
        let start_time = std::time::Instant::now();

        // Try cache first
        if let Some(cached_value) = self.cache.get(key).await? {
            return Ok(CacheOperationResult {
                value: cached_value,
                from_cache: true,
                duration: start_time.elapsed(),
            });
        }

        // Cache miss - compute the value
        let computed_value = compute_fn().await?;

        // Cache the result (with default config)
        let _ = self
            .cache
            .set(key, &computed_value, CacheEntryConfig::default())
            .await;

        Ok(CacheOperationResult {
            value: computed_value,
            from_cache: false,
            duration: start_time.elapsed(),
        })
    }

    /// Invalidate a cached value
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.cache.delete(key).await?;
        Ok(())
    }

    /// Refresh a cached value
    pub async fn refresh<F, V, Fut>(&self, key: &str, compute_fn: F) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
        V: Serialize + Clone + Send + Sync,
    {
        let value = compute_fn().await?;
        self.cache
            .set(key, &value, CacheEntryConfig::default())
            .await?;
        Ok(value)
    }
}
