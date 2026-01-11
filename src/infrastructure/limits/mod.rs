//! Resource limits and quotas for production safety
//!
//! Implements resource monitoring and limits to prevent system overload.
//! Supports memory, CPU, disk, and concurrent operation limits.

mod config;
mod types;

// Re-export configuration types
pub use config::{CpuLimits, DiskLimits, MemoryLimits, OperationLimits, ResourceLimitsConfig};

// Re-export types
pub use types::{
    CpuStats, DiskStats, MemoryStats, OperationStats, ResourceStats, ResourceViolation,
};

use crate::domain::error::{Error, Result};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Trait for resource limits operations (enables DI and testing)
///
/// ## Architecture
///
/// This module uses dependency injection via the `ResourceLimitsProvider` trait
/// to enable testability and flexibility. Pass `Arc<dyn ResourceLimitsProvider>`
/// through constructors instead of using global state.
#[async_trait]
pub trait ResourceLimitsProvider: Send + Sync {
    /// Check if an operation can proceed based on resource limits
    async fn check_operation_allowed(&self, operation_type: &str) -> Result<()>;

    /// Get current resource statistics
    async fn get_stats(&self) -> Result<ResourceStats>;

    /// Get the configuration
    fn config(&self) -> &ResourceLimitsConfig;

    /// Check if resource limits are enabled
    fn is_enabled(&self) -> bool;
}

/// Type alias for shared resource limits provider
pub type SharedResourceLimits = Arc<dyn ResourceLimitsProvider>;

#[derive(Debug, Default)]
struct OperationCounters {
    active_indexing: AtomicUsize,
    active_search: AtomicUsize,
    active_embedding: AtomicUsize,
    /// Tracks operations waiting to acquire permits
    queued_indexing: AtomicUsize,
    queued_search: AtomicUsize,
    queued_embedding: AtomicUsize,
}

/// Resource limits enforcer
#[derive(Clone)]
pub struct ResourceLimits {
    config: ResourceLimitsConfig,
    /// Semaphore for indexing operations
    indexing_semaphore: Arc<Semaphore>,
    /// Semaphore for search operations
    search_semaphore: Arc<Semaphore>,
    /// Semaphore for embedding operations
    embedding_semaphore: Arc<Semaphore>,
    /// Current operation counters
    operation_counters: Arc<OperationCounters>,
}

impl ResourceLimits {
    /// Create a new resource limits enforcer
    pub fn new(config: ResourceLimitsConfig) -> Self {
        Self {
            indexing_semaphore: Arc::new(Semaphore::new(config.operations.max_concurrent_indexing)),
            search_semaphore: Arc::new(Semaphore::new(config.operations.max_concurrent_search)),
            embedding_semaphore: Arc::new(Semaphore::new(
                config.operations.max_concurrent_embedding,
            )),
            operation_counters: Arc::new(OperationCounters::default()),
            config,
        }
    }

    /// Check if an operation can proceed based on resource limits
    pub async fn check_operation_allowed(&self, operation_type: &str) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check system resources
        let violations = self.check_system_limits().await?;
        if !violations.is_empty() {
            return Err(Error::generic(format!(
                "Resource limits exceeded: {:?}",
                violations[0]
            )));
        }

        // Check concurrency limits
        self.check_concurrency_limits(operation_type).await?;

        Ok(())
    }

    /// Acquire a permit for an operation
    pub async fn acquire_operation_permit(
        &self,
        operation_type: &str,
    ) -> Result<OperationPermit<'_>> {
        if !self.config.enabled {
            return Ok(OperationPermit {
                _permit: None,
                counters: Arc::clone(&self.operation_counters),
                operation_type: operation_type.to_string(),
            });
        }

        let permit = match operation_type {
            "indexing" => {
                self.operation_counters
                    .queued_indexing
                    .fetch_add(1, Ordering::Relaxed);

                let permit_result = self.indexing_semaphore.acquire().await;

                self.operation_counters
                    .queued_indexing
                    .fetch_sub(1, Ordering::Relaxed);

                let permit = permit_result.map_err(|e| {
                    Error::generic(format!("Failed to acquire indexing permit: {}", e))
                })?;

                self.operation_counters
                    .active_indexing
                    .fetch_add(1, Ordering::Relaxed);
                Some(permit)
            }
            "search" => {
                self.operation_counters
                    .queued_search
                    .fetch_add(1, Ordering::Relaxed);

                let permit_result = self.search_semaphore.acquire().await;

                self.operation_counters
                    .queued_search
                    .fetch_sub(1, Ordering::Relaxed);

                let permit = permit_result.map_err(|e| {
                    Error::generic(format!("Failed to acquire search permit: {}", e))
                })?;

                self.operation_counters
                    .active_search
                    .fetch_add(1, Ordering::Relaxed);
                Some(permit)
            }
            "embedding" => {
                self.operation_counters
                    .queued_embedding
                    .fetch_add(1, Ordering::Relaxed);

                let permit_result = self.embedding_semaphore.acquire().await;

                self.operation_counters
                    .queued_embedding
                    .fetch_sub(1, Ordering::Relaxed);

                let permit = permit_result.map_err(|e| {
                    Error::generic(format!("Failed to acquire embedding permit: {}", e))
                })?;

                self.operation_counters
                    .active_embedding
                    .fetch_add(1, Ordering::Relaxed);
                Some(permit)
            }
            _ => {
                return Err(Error::invalid_argument(format!(
                    "Unknown operation type: {}",
                    operation_type
                )));
            }
        };

        Ok(OperationPermit {
            _permit: permit,
            counters: Arc::clone(&self.operation_counters),
            operation_type: operation_type.to_string(),
        })
    }

    /// Get current resource statistics
    pub async fn get_stats(&self) -> Result<ResourceStats> {
        let memory = self.get_memory_stats().await?;
        let cpu = self.get_cpu_stats().await?;
        let disk = self.get_disk_stats().await?;
        let operations = self.get_operation_stats().await?;

        Ok(ResourceStats {
            memory,
            cpu,
            disk,
            operations,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Check system resource limits
    async fn check_system_limits(&self) -> Result<Vec<ResourceViolation>> {
        let mut violations = Vec::new();

        // Check memory
        if let Ok(memory_stats) = self.get_memory_stats().await {
            if memory_stats.usage_percent >= self.config.memory.max_usage_percent {
                violations.push(ResourceViolation::MemoryLimitExceeded {
                    current_percent: memory_stats.usage_percent,
                    limit_percent: self.config.memory.max_usage_percent,
                });
            }
        }

        // Check CPU
        if let Ok(cpu_stats) = self.get_cpu_stats().await {
            if cpu_stats.usage_percent >= self.config.cpu.max_usage_percent {
                violations.push(ResourceViolation::CpuLimitExceeded {
                    current_percent: cpu_stats.usage_percent,
                    limit_percent: self.config.cpu.max_usage_percent,
                });
            }
        }

        // Check disk
        if let Ok(disk_stats) = self.get_disk_stats().await {
            if disk_stats.usage_percent >= self.config.disk.max_usage_percent {
                violations.push(ResourceViolation::DiskLimitExceeded {
                    current_percent: disk_stats.usage_percent,
                    limit_percent: self.config.disk.max_usage_percent,
                });
            }
            if disk_stats.available < self.config.disk.min_free_space {
                violations.push(ResourceViolation::DiskSpaceLow {
                    available_bytes: disk_stats.available,
                    required_bytes: self.config.disk.min_free_space,
                });
            }
        }

        Ok(violations)
    }

    /// Check concurrency limits
    async fn check_concurrency_limits(&self, operation_type: &str) -> Result<()> {
        match operation_type {
            "indexing" => {
                if self
                    .operation_counters
                    .active_indexing
                    .load(Ordering::Relaxed)
                    >= self.config.operations.max_concurrent_indexing
                {
                    return Err(Error::generic("Indexing concurrency limit exceeded"));
                }
            }
            "search" => {
                if self
                    .operation_counters
                    .active_search
                    .load(Ordering::Relaxed)
                    >= self.config.operations.max_concurrent_search
                {
                    return Err(Error::generic("Search concurrency limit exceeded"));
                }
            }
            "embedding" => {
                if self
                    .operation_counters
                    .active_embedding
                    .load(Ordering::Relaxed)
                    >= self.config.operations.max_concurrent_embedding
                {
                    return Err(Error::generic("Embedding concurrency limit exceeded"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get memory statistics (cross-platform using sysinfo)
    async fn get_memory_stats(&self) -> Result<MemoryStats> {
        use sysinfo::{MemoryRefreshKind, RefreshKind, System};

        let mut system = System::new_with_specifics(
            RefreshKind::everything().with_memory(MemoryRefreshKind::everything()),
        );
        system.refresh_memory();

        let total = system.total_memory();
        let used = system.used_memory();
        let available = total.saturating_sub(used);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok(MemoryStats {
            total,
            used,
            available,
            usage_percent,
        })
    }

    /// Get CPU statistics (cross-platform using sysinfo)
    async fn get_cpu_stats(&self) -> Result<CpuStats> {
        use sysinfo::{CpuRefreshKind, RefreshKind, System};

        let mut system = System::new_with_specifics(
            RefreshKind::everything().with_cpu(CpuRefreshKind::everything()),
        );
        system.refresh_cpu_all();

        let cores = system.cpus().len();
        let usage_percent = if cores > 0 {
            system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cores as f32
        } else {
            0.0
        };

        Ok(CpuStats {
            usage_percent,
            cores,
        })
    }

    /// Get disk statistics (cross-platform using sysinfo)
    async fn get_disk_stats(&self) -> Result<DiskStats> {
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();

        // Sum up all disks' space (handles multiple disks/partitions)
        let (total, available) = disks.iter().fold((0u64, 0u64), |(total, avail), disk| {
            (total + disk.total_space(), avail + disk.available_space())
        });

        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Ok(DiskStats {
            total,
            used,
            available,
            usage_percent,
        })
    }

    /// Get operation statistics
    async fn get_operation_stats(&self) -> Result<OperationStats> {
        let queued_operations = self
            .operation_counters
            .queued_indexing
            .load(Ordering::Relaxed)
            + self
                .operation_counters
                .queued_search
                .load(Ordering::Relaxed)
            + self
                .operation_counters
                .queued_embedding
                .load(Ordering::Relaxed);

        Ok(OperationStats {
            active_indexing: self
                .operation_counters
                .active_indexing
                .load(Ordering::Relaxed),
            active_search: self
                .operation_counters
                .active_search
                .load(Ordering::Relaxed),
            active_embedding: self
                .operation_counters
                .active_embedding
                .load(Ordering::Relaxed),
            queued_operations,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &ResourceLimitsConfig {
        &self.config
    }

    /// Check if resource limits are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Implement the provider trait for ResourceLimits
#[async_trait]
impl ResourceLimitsProvider for ResourceLimits {
    async fn check_operation_allowed(&self, operation_type: &str) -> Result<()> {
        ResourceLimits::check_operation_allowed(self, operation_type).await
    }

    async fn get_stats(&self) -> Result<ResourceStats> {
        ResourceLimits::get_stats(self).await
    }

    fn config(&self) -> &ResourceLimitsConfig {
        ResourceLimits::config(self)
    }

    fn is_enabled(&self) -> bool {
        ResourceLimits::is_enabled(self)
    }
}

/// RAII guard for operation permits
pub struct OperationPermit<'a> {
    _permit: Option<tokio::sync::SemaphorePermit<'a>>,
    counters: Arc<OperationCounters>,
    operation_type: String,
}

impl Drop for OperationPermit<'_> {
    fn drop(&mut self) {
        match self.operation_type.as_str() {
            "indexing" => {
                self.counters
                    .active_indexing
                    .fetch_sub(1, Ordering::Relaxed);
            }
            "search" => {
                self.counters.active_search.fetch_sub(1, Ordering::Relaxed);
            }
            "embedding" => {
                self.counters
                    .active_embedding
                    .fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

/// Null resource limits for testing (always allows operations)
#[derive(Clone)]
pub struct NullResourceLimits {
    config: ResourceLimitsConfig,
}

impl Default for NullResourceLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl NullResourceLimits {
    /// Create a new null resource limits for testing
    pub fn new() -> Self {
        Self {
            config: ResourceLimitsConfig {
                enabled: false,
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl ResourceLimitsProvider for NullResourceLimits {
    async fn check_operation_allowed(&self, _operation_type: &str) -> Result<()> {
        Ok(())
    }

    async fn get_stats(&self) -> Result<ResourceStats> {
        Ok(ResourceStats {
            memory: MemoryStats {
                total: 0,
                used: 0,
                available: 0,
                usage_percent: 0.0,
            },
            cpu: CpuStats {
                usage_percent: 0.0,
                cores: 0,
            },
            disk: DiskStats {
                total: 0,
                used: 0,
                available: 0,
                usage_percent: 0.0,
            },
            operations: OperationStats {
                active_indexing: 0,
                active_search: 0,
                active_embedding: 0,
                queued_operations: 0,
            },
            timestamp: 0,
        })
    }

    fn config(&self) -> &ResourceLimitsConfig {
        &self.config
    }

    fn is_enabled(&self) -> bool {
        false
    }
}
