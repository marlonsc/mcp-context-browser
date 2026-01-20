//! Integration test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test integration`

#[path = "integration/ca001_integration_test.rs"]
mod ca001;

#[path = "integration/integration_architecture.rs"]
mod architecture;

#[path = "integration/integration_ast.rs"]
mod ast;

#[path = "integration/integration_duplication.rs"]
mod duplication;

#[path = "integration/integration_engines.rs"]
mod engines;

#[path = "integration/integration_full.rs"]
mod full;

#[path = "integration/integration_linters.rs"]
mod linters;

#[path = "integration/integration_metrics.rs"]
mod metrics;

#[path = "integration/integration_rca_metrics.rs"]
mod rca_metrics;

#[path = "integration/integration_test.rs"]
mod integration_test;

#[path = "integration/integration_yaml_metrics.rs"]
mod yaml_metrics;
