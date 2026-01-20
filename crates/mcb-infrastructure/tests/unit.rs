//! Unit test suite for mcb-infrastructure
//!
//! Run with: `cargo test -p mcb-infrastructure --test unit`
//!
//! The auth, snapshot, and sync tests require the `test-utils` feature:
//! `cargo test -p mcb-infrastructure --test unit --features test-utils`

#[path = "unit/constants_tests.rs"]
mod constants_tests;

#[path = "unit/crypto_tests.rs"]
mod crypto_tests;

#[path = "unit/error_ext_tests.rs"]
mod error_ext_tests;

#[path = "unit/health_tests.rs"]
mod health_tests;

#[path = "unit/logging_tests.rs"]
mod logging_tests;

#[path = "unit/di_tests.rs"]
mod di_tests;

#[path = "unit/router_tests.rs"]
mod router_tests;

#[path = "unit/lifecycle_tests.rs"]
mod lifecycle_tests;

#[path = "unit/config_figment_tests.rs"]
mod config_figment_tests;

// Infrastructure service tests (require test-utils feature)
#[cfg(feature = "test-utils")]
#[path = "unit/auth_tests.rs"]
mod auth_tests;

#[cfg(feature = "test-utils")]
#[path = "unit/snapshot_tests.rs"]
mod snapshot_tests;

#[cfg(feature = "test-utils")]
#[path = "unit/sync_tests.rs"]
mod sync_tests;
