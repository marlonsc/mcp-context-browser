//! Unit test suite for mcb-providers
//!
//! Run with: `cargo test -p mcb-providers --test unit --features hybrid-search`

#[cfg(feature = "hybrid-search")]
#[path = "unit/hybrid_search_tests.rs"]
mod hybrid_search_tests;
