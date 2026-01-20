//! Unit test suite for mcb-domain
//!
//! Run with: `cargo test -p mcb-domain --test unit`

#[path = "unit/chunk_repository_tests.rs"]
mod chunk_repository_tests;

#[path = "unit/code_chunk_tests.rs"]
mod code_chunk;

#[path = "unit/codebase_tests.rs"]
mod codebase;

#[path = "unit/config_tests.rs"]
mod config;

#[path = "unit/constants_tests.rs"]
mod constants;

#[path = "unit/domain_events_tests.rs"]
mod domain_events;

#[path = "unit/embedding_tests.rs"]
mod embedding;

#[path = "unit/error_tests.rs"]
mod error;

#[path = "unit/search_tests.rs"]
mod search;

#[path = "unit/types_tests.rs"]
mod types;

#[path = "unit/browse_tests.rs"]
mod browse;
