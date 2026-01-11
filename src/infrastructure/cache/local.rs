//! Local Moka cache operations
//!
//! Provides Moka-specific cache operations for in-memory caching.
//! Used when Redis is not configured.

use super::CacheManager;
use crate::domain::error::Result;
use moka::future::Cache;

impl CacheManager {
    /// Get the appropriate cache for a namespace
    pub(crate) fn get_cache(&self, namespace: &str) -> &Cache<String, serde_json::Value> {
        match namespace {
            "embeddings" => &self.embeddings_cache,
            "search_results" => &self.search_results_cache,
            "metadata" => &self.metadata_cache,
            "provider_responses" => &self.provider_responses_cache,
            "sync_batches" => &self.sync_batches_cache,
            _ => &self.metadata_cache,
        }
    }

    /// Get a value from the local Moka cache
    pub(crate) async fn get_from_local(&self, full_key: &str) -> Result<Option<serde_json::Value>> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        Ok(cache.get(full_key).await)
    }

    /// Set a value in the local Moka cache
    pub(crate) async fn set_in_local(&self, full_key: &str, value: serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        cache.insert(full_key.to_string(), value).await;
        Ok(())
    }

    /// Delete a value from the local Moka cache
    pub(crate) async fn delete_from_local(&self, full_key: &str) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);
        cache.invalidate(full_key).await;
        Ok(())
    }

    /// Clear a namespace in the local Moka cache
    pub(crate) async fn clear_namespace_local(&self, namespace: &str) -> Result<()> {
        let cache = self.get_cache(namespace);
        let prefix = format!("{}:", namespace);
        if let Err(e) = cache.invalidate_entries_if(move |k, _v| k.starts_with(&prefix)) {
            tracing::warn!("Failed to invalidate entries: {}", e);
        }
        Ok(())
    }

    /// Enqueue an item to a list in the local cache
    pub(crate) async fn enqueue_local(&self, full_key: &str, value: serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);

        let mut current_list = if let Some(existing) = cache.get(full_key).await {
            if let Some(arr) = existing.as_array() {
                arr.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        current_list.push(value);
        cache
            .insert(full_key.to_string(), serde_json::Value::Array(current_list))
            .await;
        Ok(())
    }

    /// Remove an item from a list in the local cache
    pub(crate) async fn remove_from_local(&self, full_key: &str, value: &serde_json::Value) -> Result<()> {
        let namespace = full_key.split(':').next().unwrap_or("");
        let cache = self.get_cache(namespace);

        if let Some(existing) = cache.get(full_key).await {
            if let Some(arr) = existing.as_array() {
                // Remove all occurrences that match
                let new_list: Vec<serde_json::Value> =
                    arr.iter().filter(|v| *v != value).cloned().collect();
                cache
                    .insert(full_key.to_string(), serde_json::Value::Array(new_list))
                    .await;
            }
        }
        Ok(())
    }
}
