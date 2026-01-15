//! Error extension utilities
//!
//! Provides context extension methods for domain errors and infrastructure-specific
//! error handling utilities.

use mcb_domain::error::{Error, Result};
use std::fmt;

/// Extension trait for adding context to errors
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_context_extension() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");

        let result: Result<()> = Err(io_error).io_context("failed to read file");
        assert!(result.is_err());

        if let Err(Error::Io { source }) = result {
            let error_message = format!("{}", source);
            assert!(error_message.contains("file not found"));
        } else {
            panic!("Expected Io error");
        }
    }

    #[test]
    fn test_infra_error_creation() {
        let error = infra::infrastructure_error_msg("test error message");

        match error {
            Error::Infrastructure { message, source } => {
                assert_eq!(message, "test error message");
                assert!(source.is_none());
            }
            _ => panic!("Expected Infrastructure error"),
        }
    }
}