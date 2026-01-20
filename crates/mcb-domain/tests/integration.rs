//! Integration test suite for mcb-domain
//!
//! Run with: `cargo test -p mcb-domain --test integration`

#[path = "integration/entity_value_object_integration.rs"]
mod entity_value_object;

#[path = "integration/semantic_search_workflow.rs"]
mod semantic_search;
