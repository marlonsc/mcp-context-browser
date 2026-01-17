//! Metrics configuration types

use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metrics enabled
    pub enabled: bool,

    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,

    /// Prometheus metrics prefix
    pub prefix: String,

    /// Metrics endpoint enabled
    pub endpoint_enabled: bool,

    /// Metrics endpoint path
    pub endpoint_path: String,

    /// External metrics exporter URL
    pub exporter_url: Option<String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_secs: METRICS_COLLECTION_INTERVAL_SECS,
            prefix: METRICS_PREFIX.to_string(),
            endpoint_enabled: true,
            endpoint_path: METRICS_PATH.to_string(),
            exporter_url: None,
        }
    }
}
