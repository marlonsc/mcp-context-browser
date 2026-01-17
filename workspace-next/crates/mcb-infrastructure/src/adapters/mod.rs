//! Infrastructure Adapters
//!
//! Provides adapter interfaces and null implementations for DI integration.
//! Following Clean Architecture: adapters implement domain interfaces.
//!
//! **ARCHITECTURE**:
//! - Provider implementations are in mcb-providers crate
//! - Repository interfaces and null implementations here
//! - Real implementations are injected at runtime via factory pattern

/// Provider adapters - interfaces and null implementations
pub mod providers;

/// Repository adapters - interfaces and null implementations
pub mod repository;
