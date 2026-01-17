//! Resource limits configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Memory limit in bytes
    pub memory_limit: usize,

    /// CPU limit (number of cores)
    pub cpu_limit: usize,

    /// Disk I/O limit in bytes per second
    pub disk_io_limit: u64,

    /// Maximum concurrent connections
    pub max_connections: u32,

    /// Maximum concurrent requests per connection
    pub max_requests_per_connection: u32,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            cpu_limit: DEFAULT_CPU_LIMIT,
            disk_io_limit: DEFAULT_DISK_IO_LIMIT,
            max_connections: 1000,
            max_requests_per_connection: 100,
        }
    }
}
