//! Resource limits configuration types
//!
//! Defines configuration structures for resource limits including
//! memory, CPU, disk, and operation concurrency limits.

use crate::infrastructure::constants::{
    CPU_LIMIT_UNHEALTHY_PERCENT, CPU_TIMEOUT_PER_OPERATION, CPU_WARNING_THRESHOLD,
    DISK_LIMIT_UNHEALTHY_PERCENT, DISK_MIN_FREE_SPACE, DISK_WARNING_THRESHOLD,
    MAX_CONCURRENT_EMBEDDING, MAX_CONCURRENT_INDEXING, MAX_CONCURRENT_SEARCH,
    MAX_OPERATION_QUEUE_SIZE, MEMORY_LIMIT_PER_OPERATION, MEMORY_LIMIT_UNHEALTHY_PERCENT,
    MEMORY_WARNING_THRESHOLD,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use validator::Validate;

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResourceLimitsConfig {
    /// Memory limits
    #[serde(default)]
    #[validate(nested)]
    pub memory: MemoryLimits,
    /// CPU limits
    #[serde(default)]
    #[validate(nested)]
    pub cpu: CpuLimits,
    /// Disk limits
    #[serde(default)]
    #[validate(nested)]
    pub disk: DiskLimits,
    /// Operation concurrency limits
    #[serde(default)]
    #[validate(nested)]
    pub operations: OperationLimits,
    /// Whether resource limits are enabled
    #[serde(default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MemoryLimits {
    /// Maximum memory usage percentage (0-100)
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_usage_percent: f32,
    /// Maximum memory per operation (bytes)
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_per_operation: u64,
    /// Warning threshold percentage
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub warning_threshold: f32,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: MEMORY_LIMIT_UNHEALTHY_PERCENT as f32,
            max_per_operation: MEMORY_LIMIT_PER_OPERATION as u64,
            warning_threshold: MEMORY_WARNING_THRESHOLD as f32,
        }
    }
}

/// CPU resource limits
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CpuLimits {
    /// Maximum CPU usage percentage (0-100)
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_usage_percent: f32,
    /// Maximum CPU time per operation (seconds)
    #[serde(default)]
    pub max_time_per_operation: Duration,
    /// Warning threshold percentage
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub warning_threshold: f32,
}

impl Default for CpuLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: CPU_LIMIT_UNHEALTHY_PERCENT as f32,
            max_time_per_operation: CPU_TIMEOUT_PER_OPERATION,
            warning_threshold: CPU_WARNING_THRESHOLD as f32,
        }
    }
}

/// Disk resource limits
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DiskLimits {
    /// Maximum disk usage percentage (0-100)
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub max_usage_percent: f32,
    /// Minimum free space required (bytes)
    #[serde(default)]
    #[validate(range(min = 1))]
    pub min_free_space: u64,
    /// Warning threshold percentage
    #[serde(default)]
    #[validate(range(min = 0.0, max = 100.0))]
    pub warning_threshold: f32,
}

impl Default for DiskLimits {
    fn default() -> Self {
        Self {
            max_usage_percent: DISK_LIMIT_UNHEALTHY_PERCENT as f32,
            min_free_space: DISK_MIN_FREE_SPACE,
            warning_threshold: DISK_WARNING_THRESHOLD as f32,
        }
    }
}

/// Operation concurrency limits
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OperationLimits {
    /// Maximum concurrent indexing operations
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_concurrent_indexing: usize,
    /// Maximum concurrent search operations
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_concurrent_search: usize,
    /// Maximum concurrent embedding operations
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_concurrent_embedding: usize,
    /// Maximum queue size for operations
    #[serde(default)]
    #[validate(range(min = 1))]
    pub max_queue_size: usize,
}

impl Default for OperationLimits {
    fn default() -> Self {
        Self {
            max_concurrent_indexing: MAX_CONCURRENT_INDEXING,
            max_concurrent_search: MAX_CONCURRENT_SEARCH,
            max_concurrent_embedding: MAX_CONCURRENT_EMBEDDING,
            max_queue_size: MAX_OPERATION_QUEUE_SIZE,
        }
    }
}
