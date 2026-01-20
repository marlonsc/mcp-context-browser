//! Unit test suite for mcb-application
//!
//! Run with: `cargo test -p mcb-application --test unit`

#[path = "unit/search_tests.rs"]
mod search_tests;

#[path = "unit/use_cases_tests.rs"]
mod use_cases_tests;

#[path = "unit/registry_tests.rs"]
mod registry_tests;
