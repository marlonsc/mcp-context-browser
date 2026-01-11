//! Infrastructure tests
//!
//! Tests for infrastructure layer components including cache, auth, limits, etc.

mod auth_tests;
mod backup_tests;
mod binary_watcher_tests;
mod cache_tests;
mod config_providers_tests;
mod connection_tracker_tests;
mod crypto_tests;
mod events_tests;
mod limits_tests;
mod logging_tests;
mod merkle_tests;
mod performance_tests;
// rate_limit_tests removed - duplicates tests/unit/rate_limiting.rs
mod respawn_tests;
mod signals_tests;
mod utils_tests;
