//! Resource limits and quotas for production safety
//!
//! Implements resource monitoring and limits to prevent system overload.
//! Supports memory, CPU, disk, and concurrent operation limits.

use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsConfig {
    /// Memory limits
    pub memory: MemoryLimits,
    /// CPU limits
    pub cpu: CpuLimits,
    /// Disk limits
    pub disk: DiskLimits,
    /// Operation concurrency limits
    pub operations: OperationLimits,
    /// Whether resource limits are enabled
    pub enabled: bool,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            memory: MemoryLimits::default(),
            cpu: CpuLimits::default(),
            disk: DiskLimits::default(),
            operations: OperationLimits::default(),
            enabled: true,
        }
    }
}

/// Memory resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimits {
    /// Maximum memory usage percentage (0-100)
    pub max_usage_percent: f32,
    /// Maximum memory per operation (bytes)
    pub max_per_operation: u64,
    /// Warning threshold percentage
    pub warning_threshold: f32,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: 85.0,
            max_per_operation: 512 * 1024 * 1024, // 512MB
            warning_threshold: 75.0,
        }
    }
}

/// CPU resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLimits {
    /// Maximum CPU usage percentage (0-100)
    pub max_usage_percent: f32,
    /// Maximum CPU time per operation (seconds)
    pub max_time_per_operation: Duration,
    /// Warning threshold percentage
    pub warning_threshold: f32,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: 80.0,
            max_time_per_operation: Duration::from_secs(300), // 5 minutes
            warning_threshold: 70.0,
        }
    }
}

/// Disk resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskLimits {
    /// Maximum disk usage percentage (0-100)
    pub max_usage_percent: f32,
    /// Minimum free space required (bytes)
    pub min_free_space: u64,
    /// Warning threshold percentage
    pub warning_threshold: f32,
}

impl Default for DiskLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: 90.0,
            min_free_space: 1024 * 1024 * 1024, // 1GB
            warning_threshold: 80.0,
        }
    }
}

/// Operation concurrency limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLimits {
    /// Maximum concurrent indexing operations
    pub max_concurrent_indexing: usize,
    /// Maximum concurrent search operations
    pub max_concurrent_search: usize,
    /// Maximum concurrent embedding operations
    pub max_concurrent_embedding: usize,
    /// Maximum queue size for operations
    pub max_queue_size: usize,
}

impl Default for OperationLimits {
    fn default() -> Self {
        Self {
            max_concurrent_indexing: 3,
            max_concurrent_search: 10,
            max_concurrent_embedding: 5,
            max_queue_size: 100,
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// Memory usage
    pub memory: MemoryStats,
    /// CPU usage
    pub cpu: CpuStats,
    /// Disk usage
    pub disk: DiskStats,
    /// Operation counts
    pub operations: OperationStats,
    /// Timestamp of measurement
    pub timestamp: u64,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f32,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub usage_percent: f32,
    pub cores: usize,
}

/// Disk usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f32,
}

/// Operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub active_indexing: usize,
    pub active_search: usize,
    pub active_embedding: usize,
    pub queued_operations: usize,
}

/// Resource limit violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceViolation {
    MemoryLimitExceeded {
        current_percent: f32,
        limit_percent: f32,
    },
    CpuLimitExceeded {
        current_percent: f32,
        limit_percent: f32,
    },
    DiskLimitExceeded {
        current_percent: f32,
        limit_percent: f32,
    },
    DiskSpaceLow {
        available_bytes: u64,
        required_bytes: u64,
    },
    ConcurrencyLimitExceeded {
        operation_type: String,
        current: usize,
        limit: usize,
    },
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
    operation_counters: Arc<Mutex<OperationCounters>>,
}

#[derive(Debug, Clone, Default)]
struct OperationCounters {
    active_indexing: usize,
    active_search: usize,
    active_embedding: usize,
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
            operation_counters: Arc::new(Mutex::new(OperationCounters::default())),
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
                let permit = self.indexing_semaphore.acquire().await.map_err(|e| {
                    Error::generic(format!("Failed to acquire indexing permit: {}", e))
                })?;
                let mut counters = self.operation_counters.lock().await;
                counters.active_indexing += 1;
                Some(permit)
            }
            "search" => {
                let permit = self.search_semaphore.acquire().await.map_err(|e| {
                    Error::generic(format!("Failed to acquire search permit: {}", e))
                })?;
                let mut counters = self.operation_counters.lock().await;
                counters.active_search += 1;
                Some(permit)
            }
            "embedding" => {
                let permit = self.embedding_semaphore.acquire().await.map_err(|e| {
                    Error::generic(format!("Failed to acquire embedding permit: {}", e))
                })?;
                let mut counters = self.operation_counters.lock().await;
                counters.active_embedding += 1;
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
        if let Ok(memory_stats) = self.get_memory_stats().await
            && memory_stats.usage_percent >= self.config.memory.max_usage_percent
        {
            violations.push(ResourceViolation::MemoryLimitExceeded {
                current_percent: memory_stats.usage_percent,
                limit_percent: self.config.memory.max_usage_percent,
            });
        }

        // Check CPU
        if let Ok(cpu_stats) = self.get_cpu_stats().await
            && cpu_stats.usage_percent >= self.config.cpu.max_usage_percent
        {
            violations.push(ResourceViolation::CpuLimitExceeded {
                current_percent: cpu_stats.usage_percent,
                limit_percent: self.config.cpu.max_usage_percent,
            });
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
        let counters = self.operation_counters.lock().await;

        match operation_type {
            "indexing" => {
                if counters.active_indexing >= self.config.operations.max_concurrent_indexing {
                    return Err(Error::generic("Indexing concurrency limit exceeded"));
                }
            }
            "search" => {
                if counters.active_search >= self.config.operations.max_concurrent_search {
                    return Err(Error::generic("Search concurrency limit exceeded"));
                }
            }
            "embedding" => {
                if counters.active_embedding >= self.config.operations.max_concurrent_embedding {
                    return Err(Error::generic("Embedding concurrency limit exceeded"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get memory statistics
    async fn get_memory_stats(&self) -> Result<MemoryStats> {
        #[cfg(target_os = "linux")]
        {
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

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback for non-Linux systems
            Ok(MemoryStats {
                total: 8 * 1024 * 1024 * 1024, // 8GB assumed
                used: 4 * 1024 * 1024 * 1024,  // 4GB assumed
                available: 4 * 1024 * 1024 * 1024,
                usage_percent: 50.0,
            })
        }
    }

    /// Get CPU statistics
    async fn get_cpu_stats(&self) -> Result<CpuStats> {
        #[cfg(target_os = "linux")]
        {
            use sysinfo::{CpuRefreshKind, RefreshKind, System};

            let mut system = System::new_with_specifics(
                RefreshKind::everything().with_cpu(CpuRefreshKind::everything()),
            );
            system.refresh_cpu_all();

            let cores = system.cpus().len();
            let usage_percent =
                system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cores as f32;

            Ok(CpuStats {
                usage_percent,
                cores,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback for non-Linux systems
            Ok(CpuStats {
                usage_percent: 25.0,
                cores: 4,
            })
        }
    }

    /// Get disk statistics
    async fn get_disk_stats(&self) -> Result<DiskStats> {
        #[cfg(target_os = "linux")]
        {
            use std::path::Path;

            let path = Path::new("/");
            let statvfs = nix::sys::statvfs::statvfs(path)?;

            let total = statvfs.blocks() * statvfs.fragment_size();
            let available = statvfs.blocks_available() * statvfs.fragment_size();
            let used = total.saturating_sub(available);
            let usage_percent = if total > 0 {
                ((total - available) as f32 / total as f32) * 100.0
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

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback for non-Linux systems
            Ok(DiskStats {
                total: 256 * 1024 * 1024 * 1024, // 256GB assumed
                used: 128 * 1024 * 1024 * 1024,  // 128GB assumed
                available: 128 * 1024 * 1024 * 1024,
                usage_percent: 50.0,
            })
        }
    }

    /// Get operation statistics
    async fn get_operation_stats(&self) -> Result<OperationStats> {
        let counters = self.operation_counters.lock().await;

        Ok(OperationStats {
            active_indexing: counters.active_indexing,
            active_search: counters.active_search,
            active_embedding: counters.active_embedding,
            queued_operations: 0, // TODO: Implement queue tracking
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

/// RAII guard for operation permits
pub struct OperationPermit<'a> {
    _permit: Option<tokio::sync::SemaphorePermit<'a>>,
    counters: Arc<Mutex<OperationCounters>>,
    operation_type: String,
}

impl Drop for OperationPermit<'_> {
    fn drop(&mut self) {
        // Decrement counter when permit is dropped
        let counters = Arc::clone(&self.counters);
        let operation_type = self.operation_type.clone();

        tokio::spawn(async move {
            let mut counters = counters.lock().await;
            match operation_type.as_str() {
                "indexing" => counters.active_indexing = counters.active_indexing.saturating_sub(1),
                "search" => counters.active_search = counters.active_search.saturating_sub(1),
                "embedding" => {
                    counters.active_embedding = counters.active_embedding.saturating_sub(1)
                }
                _ => {}
            }
        });
    }
}

/// Global resource limits instance
static RESOURCE_LIMITS: std::sync::OnceLock<ResourceLimits> = std::sync::OnceLock::new();

/// Initialize global resource limits
pub fn init_global_resource_limits(config: ResourceLimitsConfig) -> Result<()> {
    // Check if we already have resource limits
    if get_global_resource_limits().is_some() {
        return Ok(());
    }

    // Try to create and set new limits
    let limits = ResourceLimits::new(config);

    // Try to set it, but if it fails (already set), just return success
    match RESOURCE_LIMITS.set(limits) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Already set by another thread/test, just return success
            Ok(())
        }
    }
}

/// Get global resource limits
pub fn get_global_resource_limits() -> Option<&'static ResourceLimits> {
    RESOURCE_LIMITS.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_config_default() {
        let config = ResourceLimitsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.memory.max_usage_percent, 85.0);
        assert_eq!(config.cpu.max_usage_percent, 80.0);
        assert_eq!(config.disk.max_usage_percent, 90.0);
    }

    #[test]
    fn test_operation_limits_config() {
        let limits = OperationLimits::default();
        assert_eq!(limits.max_concurrent_indexing, 3);
        assert_eq!(limits.max_concurrent_search, 10);
        assert_eq!(limits.max_concurrent_embedding, 5);
    }

    #[tokio::test]
    async fn test_resource_limits_creation() {
        let config = ResourceLimitsConfig::default();
        let limits = ResourceLimits::new(config);

        assert!(limits.is_enabled());

        // Test stats collection
        let stats = limits.get_stats().await.unwrap();
        assert!(stats.timestamp > 0);
        assert!(stats.memory.total > 0);
        assert!(stats.cpu.cores > 0);
    }

    #[tokio::test]
    async fn test_operation_permits() {
        let config = ResourceLimitsConfig::default();
        let limits = ResourceLimits::new(config);

        // Acquire permits
        let _permit1 = limits.acquire_operation_permit("indexing").await.unwrap();
        let _permit2 = limits.acquire_operation_permit("search").await.unwrap();

        // Check that counters are updated
        let stats = limits.get_stats().await.unwrap();
        assert_eq!(stats.operations.active_indexing, 1);
        assert_eq!(stats.operations.active_search, 1);

        // Permits should be released when dropped
        drop(_permit1);
        drop(_permit2);

        // Give a moment for async cleanup
        tokio::time::sleep(Duration::from_millis(10)).await;

        let stats = limits.get_stats().await.unwrap();
        assert_eq!(stats.operations.active_indexing, 0);
        assert_eq!(stats.operations.active_search, 0);
    }

    #[tokio::test]
    async fn test_disabled_limits() {
        let config = ResourceLimitsConfig {
            enabled: false,
            ..Default::default()
        };
        let limits = ResourceLimits::new(config);

        assert!(!limits.is_enabled());

        // Should always allow operations when disabled
        limits.check_operation_allowed("indexing").await.unwrap();
        let _permit = limits.acquire_operation_permit("search").await.unwrap();
    }
}
