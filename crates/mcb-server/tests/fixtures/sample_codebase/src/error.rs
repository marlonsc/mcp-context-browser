//! Error handling and domain errors
//!
//! This module contains error types and error handling patterns
//! using thiserror for ergonomic error definitions.

use std::fmt;

/// Domain error types for the application
#[derive(Debug)]
pub enum DomainError {
    /// Resource not found error
    NotFound(String),
    /// Invalid input error
    InvalidInput(String),
    /// Configuration error
    ConfigurationError(String),
    /// Provider error
    ProviderError { provider: String, message: String },
    /// Internal error
    InternalError(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            DomainError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            DomainError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            DomainError::ProviderError { provider, message } => {
                write!(f, "Provider {} error: {}", provider, message)
            }
            DomainError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Error context trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F, S>(self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T, E: std::error::Error> ErrorContext<T> for Result<T, E> {
    fn with_context<F, S>(self, f: F) -> Result<T, DomainError>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| DomainError::InternalError(format!("{}: {}", f().into(), e)))
    }
}

/// Application error for HTTP responses
#[derive(Debug)]
pub struct AppError {
    pub status_code: u16,
    pub message: String,
    pub error_code: String,
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match &err {
            DomainError::NotFound(_) => AppError {
                status_code: 404,
                message: err.to_string(),
                error_code: "NOT_FOUND".to_string(),
            },
            DomainError::InvalidInput(_) => AppError {
                status_code: 400,
                message: err.to_string(),
                error_code: "INVALID_INPUT".to_string(),
            },
            DomainError::ConfigurationError(_) => AppError {
                status_code: 500,
                message: err.to_string(),
                error_code: "CONFIG_ERROR".to_string(),
            },
            DomainError::ProviderError { .. } => AppError {
                status_code: 503,
                message: err.to_string(),
                error_code: "PROVIDER_ERROR".to_string(),
            },
            DomainError::InternalError(_) => AppError {
                status_code: 500,
                message: err.to_string(),
                error_code: "INTERNAL_ERROR".to_string(),
            },
        }
    }
}
