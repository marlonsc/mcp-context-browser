//! # Infrastructure Layer
//!
//! Cross-cutting technical concerns that support the application and domain layers.
//!
//! This layer provides shared technical capabilities used across the entire system.
//! Unlike adapters (which integrate with external services), infrastructure modules
//! provide internal technical services.
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
//!
//! ## Architecture
//!
//! ```text
//! Infrastructure Layer
//! ├── Security
//! │   └── crypto         # Encryption utilities
//! ├── Data
//! │   └── cache/         # Distributed caching
//! ├── Config
//! │   ├── config/        # Configuration management
//! │   ├── di/            # Dependency injection
//! │   └── constants      # Magic number elimination
//! └── Observability
//!     ├── health         # Health checks
//!     └── logging        # Structured logging
//! ```
//!
//! ## Usage Pattern
//!
//! Infrastructure modules are typically injected via Shaku DI:
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use mcb_infrastructure::cache::SharedCacheProvider;
//! use mcb_infrastructure::crypto::CryptoService;
//!
//! struct MyService {
//!     cache: SharedCacheProvider,
//!     crypto: Arc<CryptoService>,
//! }
//! ```

// Core infrastructure modules (implemented)
/// Adapter implementations for domain ports
pub mod adapters;
pub mod cache;
/// TOML configuration management with hot-reload capabilities
pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error_ext;
pub mod health;
pub mod logging;
/// Utility helpers (timing, etc.)
pub mod utils;


// Re-export commonly used traits and types
pub use adapters::repository::{VectorStoreChunkRepository, VectorStoreSearchRepository};
pub use error_ext::ErrorContext;
pub use utils::TimedOperation;
