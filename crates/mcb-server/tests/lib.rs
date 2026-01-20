//! Test utilities for mcb-server
//!
//! This module provides shared test utilities.
//!
//! Test Structure:
//! - `tests/unit.rs` - Unit tests (args, builder, formatter, mcp_error_handling)
//! - `tests/integration.rs` - Integration tests (admin, handlers, tools, golden_acceptance)
//!
//! Run all tests: `cargo test -p mcb-server`
//! Run unit tests: `cargo test -p mcb-server --test unit`
//! Run integration: `cargo test -p mcb-server --test integration`

/// Shared test utilities
pub mod test_helpers {
    use std::time::Duration;

    /// Default timeout for async tests
    pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);
}
