//! Daemon types and configurations

use std::collections::HashMap;
use std::time::Instant;
use validator::Validate;

/// Background daemon configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct DaemonConfig {
    /// Lock cleanup interval in seconds (default: 30)
    #[validate(range(min = 1))]
    pub cleanup_interval_secs: u64,
    /// Monitoring interval in seconds (default: 30)
    #[validate(range(min = 1))]
    pub monitoring_interval_secs: u64,
    /// Maximum age for lock cleanup in seconds (default: 300 = 5 minutes)
    #[validate(range(min = 1))]
    pub max_lock_age_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_secs: 30,
            monitoring_interval_secs: 30,
            max_lock_age_secs: 300, // 5 minutes
        }
    }
}

impl DaemonConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            cleanup_interval_secs: std::env::var("DAEMON_CLEANUP_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            monitoring_interval_secs: std::env::var("DAEMON_MONITORING_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_lock_age_secs: std::env::var("DAEMON_MAX_LOCK_AGE")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        }
    }
}

/// Daemon statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct DaemonStats {
    /// Total cleanup cycles run
    pub cleanup_cycles: u64,
    /// Total locks cleaned up
    pub locks_cleaned: u64,
    /// Total monitoring cycles run
    pub monitoring_cycles: u64,
    /// Current number of active locks
    pub active_locks: usize,
    /// Timestamp of last cleanup
    pub last_cleanup: Option<std::time::SystemTime>,
    /// Timestamp of last monitoring
    pub last_monitoring: Option<std::time::SystemTime>,
}

// ============================================================================
// Recovery System Types
// ============================================================================

/// Strategy for handling component failures
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStrategy {
    /// Restart the failed component (default)
    #[default]
    Restart,
    /// Skip the component and continue without it
    Skip,
    /// Log error and wait for manual intervention
    Alert,
    /// Escalate to full process respawn
    Respawn,
}

impl std::fmt::Display for RecoveryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Restart => write!(f, "restart"),
            Self::Skip => write!(f, "skip"),
            Self::Alert => write!(f, "alert"),
            Self::Respawn => write!(f, "respawn"),
        }
    }
}

/// Recovery policy for a subsystem
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct RecoveryPolicy {
    /// Enable automatic recovery for this subsystem
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum retry attempts before giving up (0 = infinite)
    #[serde(default = "default_max_retries")]
    #[validate(range(max = 100))]
    pub max_retries: u32,

    /// Base delay between retries in milliseconds
    #[serde(default = "default_base_delay_ms")]
    #[validate(range(min = 100, max = 300000))]
    pub base_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    #[serde(default = "default_max_delay_ms")]
    #[validate(range(min = 1000, max = 600000))]
    pub max_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
    #[serde(default = "default_backoff_multiplier")]
    #[validate(range(min = 1.0, max = 10.0))]
    pub backoff_multiplier: f64,

    /// Recovery strategy to use
    #[serde(default)]
    pub strategy: RecoveryStrategy,
}

fn default_enabled() -> bool {
    true
}

fn default_max_retries() -> u32 {
    3
}

fn default_base_delay_ms() -> u64 {
    1000
}

fn default_max_delay_ms() -> u64 {
    30000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_retries: default_max_retries(),
            base_delay_ms: default_base_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            backoff_multiplier: default_backoff_multiplier(),
            strategy: RecoveryStrategy::default(),
        }
    }
}

impl RecoveryPolicy {
    /// Calculate backoff delay for a given retry attempt
    pub fn calculate_backoff(&self, attempt: u32) -> u64 {
        let delay = self.base_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        (delay as u64).min(self.max_delay_ms)
    }
}

/// Configuration for the recovery manager
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Validate)]
pub struct RecoveryConfig {
    /// Whether recovery is enabled globally
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    #[validate(range(min = 5, max = 300))]
    pub health_check_interval_secs: u64,

    /// Default policy for all subsystems
    #[serde(default)]
    #[validate(nested)]
    pub default_policy: RecoveryPolicy,

    /// Per-subsystem policy overrides (key = subsystem_id)
    #[serde(default)]
    pub subsystem_policies: HashMap<String, RecoveryPolicy>,
}

fn default_health_check_interval() -> u64 {
    30
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            health_check_interval_secs: default_health_check_interval(),
            default_policy: RecoveryPolicy::default(),
            subsystem_policies: HashMap::new(),
        }
    }
}

impl RecoveryConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("RECOVERY_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        let health_check_interval_secs = std::env::var("RECOVERY_HEALTH_CHECK_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let max_retries = std::env::var("RECOVERY_MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let base_delay_ms = std::env::var("RECOVERY_BASE_DELAY_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .unwrap_or(1000);

        Self {
            enabled,
            health_check_interval_secs,
            default_policy: RecoveryPolicy {
                max_retries,
                base_delay_ms,
                ..Default::default()
            },
            subsystem_policies: HashMap::new(),
        }
    }

    /// Get policy for a specific subsystem (falls back to default)
    pub fn get_policy(&self, subsystem_id: &str) -> &RecoveryPolicy {
        self.subsystem_policies
            .get(subsystem_id)
            .unwrap_or(&self.default_policy)
    }
}

/// Current status of recovery for a subsystem
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStatus {
    /// Subsystem is healthy, no recovery needed
    #[default]
    Healthy,
    /// Recovery is in progress
    Recovering,
    /// Max retries exhausted, waiting for manual intervention
    Exhausted,
    /// Manual intervention required (Alert strategy)
    Manual,
}

impl std::fmt::Display for RecoveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Recovering => write!(f, "Recovering"),
            Self::Exhausted => write!(f, "Exhausted"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

/// State tracking for recovery of a single subsystem
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryState {
    /// Subsystem identifier
    pub subsystem_id: String,

    /// Number of consecutive health check failures
    pub consecutive_failures: u32,

    /// Current retry attempt (0 = not recovering)
    pub current_retry: u32,

    /// Maximum retries configured for this subsystem
    pub max_retries: u32,

    /// Timestamp of last recovery attempt
    #[serde(skip)]
    pub last_recovery_attempt: Option<Instant>,

    /// Last error message from failed recovery
    pub last_error: Option<String>,

    /// Current recovery status
    pub status: RecoveryStatus,
}

impl RecoveryState {
    /// Create new recovery state for a subsystem
    pub fn new(subsystem_id: String, max_retries: u32) -> Self {
        Self {
            subsystem_id,
            consecutive_failures: 0,
            current_retry: 0,
            max_retries,
            last_recovery_attempt: None,
            last_error: None,
            status: RecoveryStatus::Healthy,
        }
    }

    /// Record a subsystem health check failure
    pub fn record_failure(&mut self, error: Option<String>) {
        self.consecutive_failures += 1;
        self.last_error = error;

        if self.status == RecoveryStatus::Healthy {
            self.status = RecoveryStatus::Recovering;
        }
    }

    /// Record a successful health check
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.current_retry = 0;
        self.last_error = None;
        self.status = RecoveryStatus::Healthy;
        self.last_recovery_attempt = None;
    }

    /// Record a recovery attempt
    pub fn record_recovery_attempt(&mut self) {
        self.current_retry += 1;
        self.last_recovery_attempt = Some(Instant::now());

        if self.max_retries > 0 && self.current_retry >= self.max_retries {
            self.status = RecoveryStatus::Exhausted;
        }
    }

    /// Reset the recovery state for manual retry
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.current_retry = 0;
        self.last_error = None;
        self.status = RecoveryStatus::Healthy;
        self.last_recovery_attempt = None;
    }

    /// Check if recovery should be attempted now
    pub fn should_attempt_recovery(&self, policy: &RecoveryPolicy) -> bool {
        if !policy.enabled {
            return false;
        }

        match self.status {
            RecoveryStatus::Healthy => false,
            RecoveryStatus::Exhausted | RecoveryStatus::Manual => false,
            RecoveryStatus::Recovering => {
                // Check if enough time has passed since last attempt
                if let Some(last_attempt) = self.last_recovery_attempt {
                    let delay = policy.calculate_backoff(self.current_retry);
                    last_attempt.elapsed().as_millis() >= delay as u128
                } else {
                    // No previous attempt, should try now
                    true
                }
            }
        }
    }
}
