//! Admin service layer - SOLID principles implementation
//!
//! This service provides a clean interface to access system data
//! following SOLID principles and dependency injection.
//!
//! The implementation is split into focused helper modules:
//! - `helpers::logging` - Log operations
//! - `helpers::maintenance` - Cache and cleanup operations
//! - `helpers::health` - Health checks and performance tests
//! - `helpers::backup` - Backup management

pub mod helpers;
mod implementation;
mod traits;
pub mod types;

pub use implementation::{AdminServiceDependencies, AdminServiceImpl};
pub use traits::*;
pub use types::*;
