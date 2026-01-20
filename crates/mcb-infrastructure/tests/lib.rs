//! Integration Tests for mcb-infrastructure
//!
//! This module provides shared test utilities for integration tests.
//!
//! Test Structure:
//! - `tests/unit.rs` - Unit tests (constants, crypto, error_ext, health, logging)
//! - `tests/integration.rs` - Integration tests (cache, config, di, utils)
//!
//! Run all tests: `cargo test -p mcb-infrastructure`
//! Run unit tests: `cargo test -p mcb-infrastructure --test unit`
//! Run integration: `cargo test -p mcb-infrastructure --test integration`

// Shared test utilities
pub mod test_helpers {
    /// Create a temporary test directory
    pub fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create temp directory")
    }
}
