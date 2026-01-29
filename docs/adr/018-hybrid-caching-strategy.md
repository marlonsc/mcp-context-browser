# ADR 018: Hybrid Caching Strategy

## Status

**Accepted** (v0.2.0 - Define, v0.3.0 - Implement)
**Date**: 2026-01-14
**Version**: v0.2.0 (Define), v0.3.0 (Implement)

## Context

Code analysis is CPU-intensive (500ms-5s per file). Caching is critical.

**Two proven patterns**:

1.  **MCB**: Moka async cache with TTL (fast lookups, memory-based)
2.  **PMAT**: SHA256 file hashing for invalidation (accurate change detection)

## Decision

**Hybrid strategy** combining both:

```rust
// crates/mcb-infrastructure/src/cache/hybrid.rs

pub struct HybridAnalysisCache {
    // Fast memory cache (MCB pattern)
    memory_cache: Arc<Cache<String, CachedAnalysis>>,

    // Change detection (PMAT pattern)
    file_hashes: Arc<DashMap<PathBuf, String>>,
}

impl HybridAnalysisCache {
    pub async fn get(&self, path: &Path) -> Result<Option<CachedAnalysis>> {
        // 1. Check if file changed (PMAT pattern)
        let current_hash = Self::sha256_file(path)?;
        if let Some(cached_hash) = self.file_hashes.get(path) {
            if cached_hash.value() != &current_hash {
                // File changed - invalidate
                self.memory_cache.invalidate(&path.display().to_string()).await;
                return Ok(None);
            }
        }

        // 2. Return cached result (MCB pattern)
        Ok(self.memory_cache.get(&path.display().to_string()).await)
    }

    pub async fn set(&self, path: &Path, analysis: CachedAnalysis) {
        // Store hash for future invalidation
        if let Ok(hash) = Self::sha256_file(path) {
            self.file_hashes.insert(path.to_path_buf(), hash);
        }

        // Store in memory cache
        self.memory_cache.insert(path.display().to_string(), analysis).await;
    }

    fn sha256_file(path: &Path) -> Result<String> {
        // COPY from PMAT (proven implementation)
    }
}
```

## Cache Layers

| Layer | Purpose | Invalidation | TTL |
|-------|---------|--------------|-----|
| **Memory (Moka)** | Fast lookups | SHA256 mismatch | 1 hour |
| **SHA256 Tracker** | Change detection | File modification | Persistent |
| **Disk** (future) | Long-term storage | LRU eviction | 7 days |

## v0.1.1 Status

Current cache implementation in `crates/mcb-providers/src/cache/`:

-   `moka.rs` - Moka async cache provider
-   `null.rs` - Null cache for testing

The hybrid cache will extend this foundation in v0.3.0.

## Implementation Plan

**v0.2.0** (Define):

-   Define `HybridAnalysisCache` interface
-   Update existing MCB cache to support pluggable invalidation

**v0.3.0** (Implement):

-   Implement SHA256 tracking
-   Integrate with analysis services
-   Benchmark cache hit rates

## Consequences

**Positive**:

-   Accurate invalidation (SHA256)
-   Fast lookups (Moka)
-   Best of both worlds

**Negative**:

-   SHA256 computation overhead (~1-5ms per file)

**Mitigation**:

-   Compute SHA256 in background
-   Cache SHA256 values
-   Only recompute on cache miss

## Related ADRs

-   [ADR-001: Modular Crates Architecture](001-modular-crates-architecture.md) - Cache provider trait
-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Cache location in mcb-providers

---

*Updated 2026-01-17 - Reflects v0.1.2 crate organization*
