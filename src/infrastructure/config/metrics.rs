use crate::infrastructure::rate_limit::RateLimitConfig;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Metrics API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(default)]
pub struct MetricsConfig {
    /// Port for metrics HTTP API
    #[serde(default = "default_metrics_port")]
    #[validate(range(min = 1))]
    pub port: u16,
    /// Enable metrics collection
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
    /// Rate limiting configuration
    #[serde(default)]
    #[validate(nested)]
    pub rate_limiting: RateLimitConfig,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            port: default_metrics_port(),
            enabled: default_metrics_enabled(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

fn default_metrics_port() -> u16 {
    // MCP_PORT is the unified port for all services (Admin + Metrics + MCP HTTP)
    std::env::var("MCP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3001)
}

fn default_metrics_enabled() -> bool {
    // Allow disabling metrics via environment variable for testing
    std::env::var("MCP_METRICS_ENABLED")
        .map(|v| !v.eq_ignore_ascii_case("false") && v != "0")
        .unwrap_or(true)
}
