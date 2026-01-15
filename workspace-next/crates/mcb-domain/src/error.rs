//! Error handling types

use thiserror::Error;

/// Result type alias for operations that can fail
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the MCP Context Browser
#[derive(Error, Debug)]
pub enum Error {
    /// I/O operation error
    #[error("I/O error: {source}")]
    Io {
        /// The underlying I/O error
        #[from]
        source: std::io::Error,
    },

    /// JSON parsing or serialization error
    #[error("JSON parsing error: {source}")]
    Json {
        /// The underlying JSON error
        #[from]
        source: serde_json::Error,
    },

    /// Generic error from external sources
    #[error("Generic error: {0}")]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// UTF-8 encoding/decoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Base64 decoding error
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Generic string-based error
    #[error("String error: {0}")]
    String(String),

    /// Resource not found error
    #[error("Not found: {resource}")]
    NotFound {
        /// The resource that was not found
        resource: String,
    },

    /// Invalid argument provided to a function
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        /// Description of the invalid argument
        message: String,
    },

    /// Vector database operation error
    #[error("Vector database error: {message}")]
    VectorDb {
        /// Description of the vector database error
        message: String,
    },

    /// Embedding provider operation error
    #[error("Embedding provider error: {message}")]
    Embedding {
        /// Description of the embedding provider error
        message: String,
    },

    /// Configuration-related error
    #[error("Configuration error: {message}")]
    Config {
        /// Description of the configuration error
        message: String,
    },

    /// Internal system error
    #[error("Internal error: {message}")]
    Internal {
        /// Description of the internal error
        message: String,
    },

    /// Cache operation error
    #[error("Cache error: {message}")]
    Cache {
        /// Description of the cache error
        message: String,
    },

    /// Infrastructure operation error
    #[error("Infrastructure error: {message}")]
    Infrastructure {
        /// Description of the infrastructure error
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    /// Create a generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic(message.into().into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource: S) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create an invalid argument error
    pub fn invalid_argument<S: Into<String>>(message: S) -> Self {
        Self::InvalidArgument {
            message: message.into(),
        }
    }

    /// Create a vector database error
    pub fn vector_db<S: Into<String>>(message: S) -> Self {
        Self::VectorDb {
            message: message.into(),
        }
    }

    /// Create an embedding provider error
    pub fn embedding<S: Into<String>>(message: S) -> Self {
        Self::Embedding {
            message: message.into(),
        }
    }

    /// Create an I/O error
    pub fn io<S: Into<String>>(message: S) -> Self {
        Self::Io {
            source: std::io::Error::other(message.into()),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(message: S) -> Self {
        Self::Cache {
            message: message.into(),
        }
    }

    /// Create an infrastructure error
    pub fn infrastructure<S: Into<String>>(message: S) -> Self {
        Self::Infrastructure {
            message: message.into(),
            source: None,
        }
    }

    /// Create an infrastructure error with source
    pub fn infrastructure_with_source<S: Into<String>, E: std::error::Error + Send + Sync + 'static>(message: S, source: E) -> Self {
        Self::Infrastructure {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

// Note: nix::errno::Errno conversion removed for domain purity
// Infrastructure layer should handle OS-specific error conversions

impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

// Note: External crate error conversions removed for domain purity
// Infrastructure layer should handle these conversions
