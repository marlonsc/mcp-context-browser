//! # Infrastructure Layer
//!
//! Cross-cutting technical concerns that support the application and domain layers.
//!
//! This layer provides shared technical capabilities used across the entire system.
//! All adapters/providers are in mcb-providers crate, accessed via Shaku DI.
//!
//! ## Module Categories
//!
//! ### Security & Authentication
//! | Module | Description |
//! |--------|-------------|
//! | [`crypto`] | AES-GCM encryption, secure key generation |
//!
//! ### Data & Storage
//! | Module | Description |
//! |--------|-------------|
//! | [`cache`] | Moka/Redis caching with TTL and namespaces |
//!
//! ### Configuration & DI
//! | Module | Description |
//! |--------|-------------|
//! | [`config`] | TOML configuration with hot-reload |
//! | [`di`] | Shaku dependency injection modules |
//! | [`constants`] | Centralized configuration constants |
//!
//! ### Observability
//! | Module | Description |
//! |--------|-------------|
//! | [`health`] | Health check endpoints |
//! | [`logging`] | Structured logging with tracing |

// Core infrastructure modules
pub mod cache;
pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error_ext;
pub mod health;
pub mod logging;
pub mod utils;

// DI bridge modules (re-exports for module composition)
pub mod adapters;
pub mod application;
pub mod infrastructure;

// Re-export commonly used types
pub use error_ext::ErrorContext;
pub use utils::TimedOperation;
