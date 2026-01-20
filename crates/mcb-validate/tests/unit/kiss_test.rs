//! Tests for KISS (Keep It Simple, Stupid) Validation

use crate::test_utils::create_test_crate;
use mcb_validate::{KissValidator, KissViolation};
use tempfile::TempDir;

#[test]
fn test_struct_too_many_fields() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub struct TooManyFields {
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
}
",
    );

    let validator = KissValidator::new(temp.path());
    let violations = validator.validate_struct_fields().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        KissViolation::StructTooManyFields { field_count, .. } => {
            assert!(*field_count > 7);
        }
        _ => panic!("Expected StructTooManyFields"),
    }
}

#[test]
fn test_function_too_many_params() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn too_many_params(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    a + b + c + d + e + f
}
",
    );

    let validator = KissValidator::new(temp.path());
    let violations = validator.validate_function_params().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        KissViolation::FunctionTooManyParams { param_count, .. } => {
            assert!(*param_count > 5);
        }
        _ => panic!("Expected FunctionTooManyParams"),
    }
}

#[test]
fn test_function_too_long() {
    let temp = TempDir::new().unwrap();
    let long_function = format!(
        r"
pub fn long_function() {{
{}
}}
",
        (0..60)
            .map(|i| format!("    let x{i} = {i};"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    create_test_crate(&temp, "mcb-test", &long_function);

    let validator = KissValidator::new(temp.path());
    let violations = validator.validate_function_length().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        KissViolation::FunctionTooLong { line_count, .. } => {
            assert!(*line_count > 50);
        }
        _ => panic!("Expected FunctionTooLong"),
    }
}

#[test]
fn test_acceptable_struct() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub struct AcceptableFields {
    field1: String,
    field2: String,
    field3: String,
}
",
    );

    let validator = KissValidator::new(temp.path());
    let violations = validator.validate_struct_fields().unwrap();

    assert!(violations.is_empty());
}
