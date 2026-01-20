//! # Infrastructure Layer
//!
//! Cross-cutting technical concerns that support the application and domain layers.
//!
//! This layer provides shared technical capabilities used across the entire system.
//! All adapters/providers are in mcb-providers crate, accessed via linkme registry.
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
//! | [`di`] | Handle-based dependency injection |
//! | [`constants`] | Centralized configuration constants |
//!
//! ### Observability
//! | Module | Description |
//! |--------|-------------|
//! | [`health`] | Health check endpoints |
//! | [`logging`] | Structured logging with tracing |
//!
//! ### Routing & Selection
//! | Module | Description |
//! |--------|-------------|
//! | [`routing`] | Provider routing and selection |

// Clippy allows for complex patterns in infrastructure code
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_range_contains)]

// Core infrastructure modules
pub mod cache;
pub mod config;
pub mod constants;
pub mod crypto;
pub mod di;
pub mod error_ext;
pub mod health;
pub mod logging;
pub mod routing;
pub mod utils;

// DI bridge modules (re-exports for module composition)
pub mod adapters;
pub mod infrastructure;

// Re-export commonly used types
pub use error_ext::ErrorContext;
pub use utils::TimedOperation;

// Provider registration happens automatically via linkme distributed slices
// when mcb-providers is compiled as a dependency
