//! Unit test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test unit`

// Shared test utilities
#[path = "unit/test_utils.rs"]
mod test_utils;

#[path = "unit/architecture_rules_test_tests.rs"]
mod architecture_rules;

#[path = "unit/ast_executor_test_tests.rs"]
mod ast_executor;

#[path = "unit/ast_test_tests.rs"]
mod ast;

#[path = "unit/async_patterns_test_tests.rs"]
mod async_patterns;

#[path = "unit/cargo_dependency_test_tests.rs"]
mod cargo_dependency;

#[path = "unit/documentation_test_tests.rs"]
mod documentation;

#[path = "unit/error_boundary_test_tests.rs"]
mod error_boundary;

#[path = "unit/expression_engine_test_tests.rs"]
mod expression_engine;

#[path = "unit/implementation_test_tests.rs"]
mod implementation;

#[path = "unit/kiss_test_tests.rs"]
mod kiss;

#[path = "unit/lib_tests.rs"]
mod lib_tests;

#[path = "unit/linters_test_tests.rs"]
mod linters;

#[path = "unit/organization_test_tests.rs"]
mod organization;

#[path = "unit/patterns_test_tests.rs"]
mod patterns;

#[path = "unit/performance_test_tests.rs"]
mod performance;

#[path = "unit/quality_test_tests.rs"]
mod quality;

#[path = "unit/refactoring_test_tests.rs"]
mod refactoring;

#[path = "unit/rete_engine_test_tests.rs"]
mod rete_engine;

#[path = "unit/solid_test_tests.rs"]
mod solid;

#[path = "unit/template_engine_test_tests.rs"]
mod template_engine;

#[path = "unit/tests_org_test_tests.rs"]
mod tests_org;

#[path = "unit/unwrap_detector_test_tests.rs"]
mod unwrap_detector;

#[path = "unit/yaml_loader_test_tests.rs"]
mod yaml_loader;
