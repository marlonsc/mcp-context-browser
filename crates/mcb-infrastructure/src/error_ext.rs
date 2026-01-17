//! Error extension utilities
//!
//! Provides context extension methods for domain errors and infrastructure-specific
//! error handling utilities.

use mcb_domain::error::{Error, Result};
use std::fmt;

/// Extension trait for adding context to errors
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::error_ext::ErrorContext;
///
/// // Add context to file operations
/// let content = std::fs::read_to_string(&path)
///     .io_context(format!("Failed to read config file: {}", path.display()))?;
///
/// // Add context with lazy evaluation
/// let result = operation()
///     .with_context(|| format!("Operation failed for item {}", expensive_id()))?;
///
/// // Type-specific context
/// auth_service.validate(token).auth_context("Invalid authentication token")?;
/// ```
pub trait ErrorContext<T> {
    /// Add context to a Result, converting the error to our domain Error type
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;

    /// Add context with lazy evaluation for expensive context creation
    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add context for I/O operations
    fn io_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized;

    /// Add context for configuration operations
    fn config_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized;

    /// Add context for authentication operations
    fn auth_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized;

    /// Add context for network operations
    fn network_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized;

    /// Add context for database operations
    fn db_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error::Infrastructure {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|err| Error::Infrastructure {
            message: format!("{}: {}", f(), err),
            source: Some(Box::new(err)),
        })
    }

    fn io_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized,
    {
        self.map_err(|err| Error::Io {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }

    fn config_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized,
    {
        self.map_err(|err| Error::Configuration {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }

    fn auth_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized,
    {
        self.map_err(|err| Error::Authentication {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }

    fn network_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized,
    {
        self.map_err(|err| Error::Network {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }

    fn db_context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        Self: Sized,
    {
        self.map_err(|err| Error::Database {
            message: format!("{}: {}", context, err),
            source: Some(Box::new(err)),
        })
    }
}

/// Convert standard library errors to domain errors with context
pub fn to_domain_error<E>(error: E, context: &str) -> Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    Error::Infrastructure {
        message: format!("{}: {}", context, error),
        source: Some(Box::new(error)),
    }
}

/// Convert standard library errors to domain results with context
pub fn to_domain_result<T, E>(result: std::result::Result<T, E>, context: &str) -> Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.map_err(|err| to_domain_error(err, context))
}

/// Infrastructure-specific error utilities
pub mod infra {
    use super::*;

    /// Convert any error to infrastructure error
    pub fn infrastructure_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Infrastructure {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }

    /// Create infrastructure error from message only
    pub fn infrastructure_error_msg(message: &str) -> Error {
        Error::Infrastructure {
            message: message.to_string(),
            source: None,
        }
    }

    /// Convert I/O error with context
    pub fn io_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Io {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }

    /// Convert configuration error with context
    pub fn config_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Configuration {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }

    /// Convert authentication error with context
    pub fn auth_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Authentication {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }

    /// Convert network error with context
    pub fn network_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Network {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }

    /// Convert database error with context
    pub fn db_error<E>(error: E, message: &str) -> Error
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Error::Database {
            message: message.to_string(),
            source: Some(Box::new(error)),
        }
    }
}
