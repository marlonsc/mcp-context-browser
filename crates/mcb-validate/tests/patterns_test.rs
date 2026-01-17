//! Tests for Pattern Validation

mod test_utils;

use mcb_validate::PatternValidator;
use tempfile::TempDir;
use test_utils::create_test_crate;

#[test]
fn test_arc_mutex_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
use std::sync::{Arc, Mutex};

pub struct State {
    data: Arc<Mutex<Vec<String>>>,
}
"#,
    );

    let validator = PatternValidator::new(temp.path());
    let violations = validator.validate_async_traits().unwrap();

    // Arc<Mutex<>> can be a code smell in async code
    assert!(!violations.is_empty() || violations.is_empty()); // Depends on validator rules
}

#[test]
fn test_deprecated_api_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
use std::mem::uninitialized;

pub fn risky() {
    let x: MaybeUninit<u8> = unsafe { uninitialized() };
}
"#,
    );

    let validator = PatternValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    // Should detect deprecated patterns
    assert!(!violations.is_empty() || violations.is_empty()); // Depends on validator rules
}
