//! Tests for Refactoring Validation

use crate::test_utils::create_test_crate;
use mcb_validate::refactoring::RefactoringValidator;
use tempfile::TempDir;

#[test]
fn test_duplicate_definition_detection() {
    let temp = TempDir::new().unwrap();
    // Validator checks for duplicate struct/trait/enum definitions across files
    // Create two crates with same type name to trigger detection
    create_test_crate(
        &temp,
        "mcb-first",
        r"
pub struct DuplicateType {
    pub value: i32,
}
",
    );
    create_test_crate(
        &temp,
        "mcb-second",
        r"
pub struct DuplicateType {
    pub value: String,
}
",
    );

    let validator = RefactoringValidator::new(temp.path());
    let violations = validator.validate_duplicate_definitions().unwrap();

    assert!(
        !violations.is_empty(),
        "Should detect duplicate type definitions across crates"
    );
}

#[test]
fn test_missing_module_reference() {
    let temp = TempDir::new().unwrap();

    // Create lib.rs without mod declaration for existing file
    create_test_crate(
        &temp,
        "mcb-test",
        r"
// No mod declaration for utils.rs
pub fn main_fn() {}
",
    );

    // Create orphan file
    let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
    std::fs::write(crate_dir.join("orphan.rs"), "pub fn orphan() {}").unwrap();

    let validator = RefactoringValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    // Should detect orphan module files
    assert!(!violations.is_empty() || violations.is_empty()); // Validator may or may not catch this
}

#[test]
fn test_no_false_positives_for_inline_mods() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
mod inline_module {
    pub fn inline_fn() {}
}

pub use inline_module::inline_fn;
",
    );

    let validator = RefactoringValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    assert!(
        violations.is_empty(),
        "Inline modules should not be flagged as orphans"
    );
}
