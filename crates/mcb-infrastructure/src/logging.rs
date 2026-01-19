//! Structured logging with tracing
//!
//! Provides centralized logging configuration and utilities using the tracing ecosystem.
//! This module configures structured logging with JSON output, log levels, and file rotation.

use mcb_domain::error::{Error, Result};

// Re-export LoggingConfig for convenience
pub use crate::config::LoggingConfig;
use tracing::{Level, debug, error, info, warn};
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize logging with the provided configuration
pub fn init_logging(config: LoggingConfig) -> Result<()> {
    let level = parse_log_level(&config.level)?;
    let filter = create_log_filter(&config.level);
    let file_appender = create_file_appender(&config.file_output);

    if config.json_format {
        init_json_logging(filter, file_appender)?;
    } else {
        init_text_logging(filter, file_appender)?;
    }

    info!("Logging initialized with level: {}", level);
    Ok(())
}

/// Create log filter from configuration
fn create_log_filter(level: &str) -> EnvFilter {
    EnvFilter::try_from_env("MCB_LOG").unwrap_or_else(|_| EnvFilter::new(level))
}

/// Create file appender if file output is configured
fn create_file_appender(
    file_output: &Option<std::path::PathBuf>,
) -> Option<tracing_appender::rolling::RollingFileAppender> {
    file_output.as_ref().map(|path| {
        tracing_appender::rolling::daily(
            path.parent().unwrap_or_else(|| std::path::Path::new(".")),
            path.file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("mcb")),
        )
    })
}

/// Initialize logging with JSON format
fn init_json_logging(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
) -> Result<()> {
    let stdout = fmt::layer()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    let registry = Registry::default().with(filter);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .json()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(stdout).with(file).init();
    } else {
        registry.with(stdout).init();
    }
    Ok(())
}

/// Initialize logging with text format
fn init_text_logging(
    filter: EnvFilter,
    file_appender: Option<tracing_appender::rolling::RollingFileAppender>,
) -> Result<()> {
    let stdout = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true);

    let registry = Registry::default().with(filter);
    if let Some(appender) = file_appender {
        let file = fmt::layer()
            .with_writer(appender)
            .with_ansi(false)
            .with_target(true);
        registry.with(stdout).with(file).init();
    } else {
        registry.with(stdout).init();
    }
    Ok(())
}

/// Parse log level string to tracing Level
pub fn parse_log_level(level: &str) -> Result<Level> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(Error::Configuration {
            message: format!(
                "Invalid log level: {}. Use trace, debug, info, warn, or error",
                level
            ),
            source: None,
        }),
    }
}

/// Log configuration loading status
pub fn log_config_loaded(config_path: &std::path::Path, success: bool) {
    if success {
        info!("Configuration loaded from {}", config_path.display());
    } else {
        warn!("Configuration file not found: {}", config_path.display());
    }
}

/// Log health check result
pub fn log_health_check(component: &str, healthy: bool, details: Option<&str>) {
    if healthy {
        debug!(component = component, "Health check passed");
    } else {
        error!(
            component = component,
            details = details.unwrap_or("Unknown failure"),
            "Health check failed"
        );
    }
}
