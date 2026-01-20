//! Unit test suite for mcb-infrastructure
//!
//! Run with: `cargo test -p mcb-infrastructure --test unit`

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
