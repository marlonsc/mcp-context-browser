//! Infrastructure layer - cross-cutting concerns and adapters
//!
//! This module contains:
//! - Error handling utilities (error_ext)
//! - Authentication and authorization
//! - Caching layer
//! - Configuration management
//! - Cryptographic operations
//! - Event bus
//! - Health checking
//! - Logging
//! - Metrics collection
//! - Rate limiting
//! - Resource limits
//! - And more...

pub mod auth;
pub mod backup;
pub mod binary_watcher;
pub mod cache;
pub mod config;
pub mod connection_tracker;
pub mod constants;
pub mod crypto;
pub mod daemon;
pub mod di;
pub mod error_ext;
pub mod events;
pub mod health;
pub mod limits;
pub mod logging;
pub mod merkle;
pub mod metrics;
pub mod provider_connection_tracker;
pub mod provider_lifecycle;
pub mod rate_limit;
pub mod recovery;
pub mod respawn;
pub mod service_helpers;
pub mod shutdown;
pub mod signals;
pub mod snapshot;
pub mod sync;
pub mod utils;

// Re-export commonly used traits and types
pub use error_ext::ErrorContext;
