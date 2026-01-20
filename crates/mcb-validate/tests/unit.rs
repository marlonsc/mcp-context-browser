//! Unit test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test unit`

// Shared test utilities
#[path = "unit/test_utils.rs"]
mod test_utils;

#[path = "unit/architecture_rules_test.rs"]
mod architecture_rules;

#[path = "unit/ast_executor_test.rs"]
mod ast_executor;

#[path = "unit/ast_test.rs"]
mod ast;

#[path = "unit/async_patterns_test.rs"]
mod async_patterns;

#[path = "unit/cargo_dependency_test.rs"]
mod cargo_dependency;

#[path = "unit/documentation_test.rs"]
mod documentation;

#[path = "unit/error_boundary_test.rs"]
mod error_boundary;

#[path = "unit/expression_engine_test.rs"]
mod expression_engine;

#[path = "unit/implementation_test.rs"]
mod implementation;

#[path = "unit/kiss_test.rs"]
mod kiss;

#[path = "unit/lib_tests.rs"]
mod lib_tests;

#[path = "unit/linters_test.rs"]
mod linters;

#[path = "unit/organization_test.rs"]
mod organization;

#[path = "unit/patterns_test.rs"]
mod patterns;

#[path = "unit/performance_test.rs"]
mod performance;

#[path = "unit/quality_test.rs"]
mod quality;

#[path = "unit/quality_tests.rs"]
mod quality_tests;

#[path = "unit/refactoring_test.rs"]
mod refactoring;

#[path = "unit/rete_engine_test.rs"]
mod rete_engine;

#[path = "unit/solid_test.rs"]
mod solid;

#[path = "unit/template_engine_test.rs"]
mod template_engine;

#[path = "unit/tests_org_test.rs"]
mod tests_org;

#[path = "unit/unwrap_detector_test.rs"]
mod unwrap_detector;

#[path = "unit/yaml_loader_test.rs"]
mod yaml_loader;
