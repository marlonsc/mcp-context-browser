//! DI Modules for infrastructure components
//!
//! Contains modules that define how infrastructure components
//! are wired together and injected into the application.

pub mod domain_services;

/// Re-export commonly used module types
pub use domain_services::*;