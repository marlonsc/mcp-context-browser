//! Structured logging with tracing
//!
//! Provides centralized logging configuration and utilities using the tracing ecosystem.
//! This module configures structured logging with JSON output, log levels, and file rotation.

use crate::constants::*;
use mcb_domain::error::{Error, Result};
use std::path::PathBuf;
use tracing::{info, warn, error, debug, trace, Level};
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry,
};

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Enable JSON output format
    pub json_format: bool,
    /// Log to file in addition to stdout
    pub file_output: Option<PathBuf>,
    /// Maximum file size before rotation (bytes)
    pub max_file_size: u64,
    /// Maximum number of rotated files to keep
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: DEFAULT_LOG_LEVEL.to_string(),
            json_format: false,
            file_output: None,
            max_file_size: LOG_ROTATION_SIZE,
            max_files: LOG_MAX_FILES,
        }
    }
}

/// Initialize logging with the provided configuration
pub fn init_logging(config: LoggingConfig) -> Result<()> {
    let level = parse_log_level(&config.level)?;

    // Create environment filter
    let filter = EnvFilter::try_from_env("MCB_LOG")
        .unwrap_or_else(|_| EnvFilter::new(config.level));

    // Create stdout layer
    let stdout_layer = if config.json_format {
        fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
    } else {
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
    };

    let registry = Registry::default()
        .with(filter);

    // Add file layer if configured
    if let Some(file_path) = config.file_output {
        let file_appender = tracing_appender::rolling::daily(
            file_path.parent().unwrap_or_else(|| std::path::Path::new(".")),
            file_path.file_stem().unwrap_or_else(|| std::ffi::OsStr::new("mcb")),
        );

        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true);

        registry
            .with(stdout_layer)
            .with(file_layer)
            .init();
    } else {
        registry
            .with(stdout_layer)
            .init();
    }

    info!("Logging initialized with level: {}", level);

    Ok(())
}

/// Parse log level string to tracing Level
fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(Error::Configuration {
            message: format!("Invalid log level: {}. Use trace, debug, info, warn, or error", level),
            source: None,
        }),
    }
}

/// Log an operation with timing information
pub fn log_operation<F, T>(operation_name: &str, operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let start = std::time::Instant::now();
    debug!("Starting operation: {}", operation_name);

    match operation() {
        Ok(result) => {
            let duration = start.elapsed();
            info!("Operation '{}' completed successfully in {:?}", operation_name, duration);
            Ok(result)
        }
        Err(err) => {
            let duration = start.elapsed();
            error!(
                "Operation '{}' failed after {:?}: {}",
                operation_name, duration, err
            );
            Err(err)
        }
    }
}

/// Create a structured log entry for performance metrics
pub fn log_performance(operation: &str, duration: std::time::Duration, success: bool) {
    if success {
        info!(
            operation = operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Performance metric"
        );
    } else {
        warn!(
            operation = operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Performance metric - operation failed"
        );
    }
}

/// Log system health status
pub fn log_health_check(component: &str, healthy: bool, details: Option<&str>) {
    if healthy {
        debug!(
            component = component,
            healthy = healthy,
            details = details.unwrap_or(""),
            "Health check passed"
        );
    } else {
        error!(
            component = component,
            healthy = healthy,
            details = details.unwrap_or("Unknown failure"),
            "Health check failed"
        );
    }
}

/// Log configuration loading
pub fn log_config_loaded(config_path: &std::path::Path, success: bool) {
    if success {
        info!("Configuration loaded successfully from {}", config_path.display());
    } else {
        warn!("Failed to load configuration from {}", config_path.display());
    }
}

/// Log service startup/shutdown
pub fn log_service_lifecycle(service: &str, event: &str, success: bool) {
    match event {
        "starting" => {
            info!("Starting service: {}", service);
        }
        "started" => {
            if success {
                info!("Service started successfully: {}", service);
            } else {
                error!("Failed to start service: {}", service);
            }
        }
        "stopping" => {
            info!("Stopping service: {}", service);
        }
        "stopped" => {
            if success {
                info!("Service stopped successfully: {}", service);
            } else {
                warn!("Service stopped with issues: {}", service);
            }
        }
        _ => {
            debug!("Service {} event: {}", service, event);
        }
    }
}

/// Utility functions for conditional logging
pub mod conditional {
    use super::*;

    /// Log only if debug level is enabled
    pub fn debug_enabled<F>(f: F)
    where
        F: FnOnce(),
    {
        if tracing::level_enabled!(Level::DEBUG) {
            f();
        }
    }

    /// Log only if trace level is enabled
    pub fn trace_enabled<F>(f: F)
    where
        F: FnOnce(),
    {
        if tracing::level_enabled!(Level::TRACE) {
            f();
        }
    }
}

/// Macros for structured logging
#[macro_export]
macro_rules! log_error {
    ($err:expr, $msg:expr) => {
        error!("{}: {}", $msg, $err);
    };
    ($err:expr, $msg:expr, $($field:tt = $value:expr),* $(,)?) => {
        error!($($field = $value,)* error = %$err, "{}", $msg);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        warn!("{}", $msg);
    };
    ($msg:expr, $($field:tt = $value:expr),* $(,)?) => {
        warn!($($field = $value,)* "{}", $msg);
    };
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        info!("{}", $msg);
    };
    ($msg:expr, $($field:tt = $value:expr),* $(,)?) => {
        info!($($field = $value,)* "{}", $msg);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        debug!("{}", $msg);
    };
    ($msg:expr, $($field:tt = $value:expr),* $(,)?) => {
        debug!($($field = $value,)* "{}", $msg);
    };
}

#[macro_export]
macro_rules! log_trace {
    ($msg:expr) => {
        trace!("{}", $msg);
    };
    ($msg:expr, $($field:tt = $value:expr),* $(,)?) => {
        trace!($($field = $value,)* "{}", $msg);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logging() {
        INIT.call_once(|| {
            let _ = init_logging(LoggingConfig::default());
        });
    }

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("trace").unwrap(), Level::TRACE);
        assert_eq!(parse_log_level("debug").unwrap(), Level::DEBUG);
        assert_eq!(parse_log_level("info").unwrap(), Level::INFO);
        assert_eq!(parse_log_level("warn").unwrap(), Level::WARN);
        assert_eq!(parse_log_level("warning").unwrap(), Level::WARN);
        assert_eq!(parse_log_level("error").unwrap(), Level::ERROR);

        assert!(parse_log_level("invalid").is_err());
    }

    #[test]
    fn test_log_operation_success() {
        init_test_logging();

        let result = log_operation("test_operation", || Ok::<_, Error>(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_log_operation_failure() {
        init_test_logging();

        let result = log_operation("test_operation", || {
            Err(Error::Infrastructure {
                message: "test error".to_string(),
                source: None,
            })
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, DEFAULT_LOG_LEVEL);
        assert!(!config.json_format);
        assert!(config.file_output.is_none());
        assert_eq!(config.max_file_size, LOG_ROTATION_SIZE);
        assert_eq!(config.max_files, LOG_MAX_FILES);
    }
}