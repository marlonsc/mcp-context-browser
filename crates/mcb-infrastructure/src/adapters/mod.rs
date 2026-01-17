//! Infrastructure Adapters
//!
//! Provides adapter interfaces for DI integration.
//! Following Clean Architecture: adapters implement domain interfaces.
//!
//! **ARCHITECTURE**:
//! - All provider implementations are in mcb-providers crate
//! - Repository interfaces are in mcb-domain crate
//! - Real implementations are injected at runtime via factory pattern
