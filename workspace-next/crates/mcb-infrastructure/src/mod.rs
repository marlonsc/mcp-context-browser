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
//! | [`auth`] | JWT authentication, RBAC, API keys, rate limiting |
//! | [`crypto`] | AES-GCM encryption, secure key generation |
//!
//! ### Data & Storage
//! | Module | Description |
//! |--------|-------------|
//! | [`cache`] | Moka/Redis caching with TTL and namespaces |
//! | [`backup`] | Encrypted backup/restore operations |
//! | [`snapshot`] | File system snapshots for change detection |
//! | [`sync`] | File synchronization coordination |
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
//! | [`metrics`] | Prometheus metrics collection |
//! | [`health`] | Health check endpoints |
//! | [`logging`] | Structured logging with tracing |
//!
//! ### Resilience
//! | Module | Description |
//! |--------|-------------|
//! | [`resilience`] | Circuit breakers, rate limiters |
//! | [`recovery`] | Error recovery strategies |
//! | [`limits`] | Resource limits (memory, CPU, disk) |
//!
//! ### Lifecycle
//! | Module | Description |
//! |--------|-------------|
//! | [`daemon`] | Background service management |
//! | [`shutdown`] | Graceful shutdown coordination |
//! | [`signals`] | Unix signal handling |
//! | [`events`] | Event bus (Tokio broadcast, NATS) |
//!
//! ## Architecture
//!
//! ```text
//! Infrastructure Layer
//! ├── Security
//! │   ├── auth/          # JWT, RBAC, API keys
//! │   └── crypto         # Encryption utilities
//! ├── Data
//! │   ├── cache/         # Distributed caching
//! │   ├── backup         # Backup/restore
//! │   └── snapshot       # Change detection
//! ├── Config
//! │   ├── config/        # Configuration management
//! │   ├── di/            # Dependency injection
//! │   └── constants      # Magic number elimination
//! ├── Observability
//! │   ├── metrics/       # Prometheus metrics
//! │   ├── health         # Health checks
//! │   └── logging        # Structured logging
//! └── Resilience
//!     ├── resilience/    # Circuit breakers
//!     ├── recovery       # Error recovery
//!     └── limits         # Resource limits
//! ```
//!
//! ## Usage Pattern
//!
//! Infrastructure modules are typically injected via Shaku DI:
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use mcb_infrastructure::cache::SharedCacheProvider;
//! use mcb_infrastructure::auth::AuthService;
//!
//! struct MyService {
//!     cache: SharedCacheProvider,
//!     auth: Arc<AuthService>,
//! }
//! ```

// Core infrastructure modules (implemented)
pub mod cache;
/// TOML configuration management with hot-reload capabilities
pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error_ext;
pub mod health;
pub mod logging;

// Placeholder modules (to be implemented - commented out to avoid compilation errors)
// pub mod auth;
// pub mod backup;
// pub mod binary_watcher;
// pub mod connection_tracker;
// pub mod daemon;
// pub mod di;
// pub mod events;
// pub mod limits;
// pub mod merkle;
// pub mod metrics;
// pub mod operations;
// pub mod provider_connection_tracker;
// pub mod provider_lifecycle;
// pub mod recovery;
// pub mod resilience;
// pub mod respawn;
// pub mod service_helpers;
// pub mod shutdown;
// pub mod signals;
// pub mod snapshot;
// pub mod sync;
// pub mod utils;

// Re-export commonly used traits and types
pub use error_ext::ErrorContext;