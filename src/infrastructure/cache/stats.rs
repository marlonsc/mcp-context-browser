//! Cache statistics and monitoring
//!
//! Provides statistics collection and monitoring functionality for the cache system.

use super::config::{CacheNamespaceConfig, CacheStats};
use super::CacheManager;
use std::sync::atomic::Ordering;

impl CacheManager {
    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = CacheStats::default();

        // If in Redis mode, local caches are empty, stats will reflect that.
        // We could potentially fetch Redis info, but that's expensive/complex.
        // So stats will track local usage or be mostly empty for Redis mode (except hits/misses).

        for cache in [
            &self.embeddings_cache,
            &self.search_results_cache,
            &self.metadata_cache,
            &self.provider_responses_cache,
            &self.sync_batches_cache,
        ] {
            cache.run_pending_tasks().await;
            stats.total_entries += cache.entry_count() as usize;
        }

        stats.hits = self.stats_hits.load(Ordering::Relaxed);
        stats.misses = self.stats_misses.load(Ordering::Relaxed);
        stats.evictions = self.stats_evictions.load(Ordering::Relaxed);

        let total_requests = stats.hits + stats.misses;
        if total_requests > 0 {
            stats.hit_ratio = stats.hits as f64 / total_requests as f64;
        }

        stats
    }

    /// Check if cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &super::config::CacheConfig {
        &self.config
    }

    /// Get namespace configuration
    pub(crate) fn get_namespace_config(&self, namespace: &str) -> &CacheNamespaceConfig {
        match namespace {
            "embeddings" => &self.config.namespaces.embeddings,
            "search_results" => &self.config.namespaces.search_results,
            "metadata" => &self.config.namespaces.metadata,
            "provider_responses" => &self.config.namespaces.provider_responses,
            "sync_batches" => &self.config.namespaces.sync_batches,
            _ => &self.config.namespaces.metadata, // Default to metadata config
        }
    }
}
