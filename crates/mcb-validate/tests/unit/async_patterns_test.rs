//! Tests for Async Patterns Validation

use crate::test_utils::create_test_crate;
use mcb_validate::async_patterns::AsyncPatternValidator;
use tempfile::TempDir;

#[test]
fn test_blocking_in_async_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
use std::thread;

pub async fn bad_async() {
    thread::sleep(std::time::Duration::from_secs(1));
}
",
    );

    let validator = AsyncPatternValidator::new(temp.path());
    let violations = validator.validate_blocking_in_async().unwrap();

    assert!(
        !violations.is_empty(),
        "Should detect blocking call in async context"
    );
}

#[test]
fn test_proper_async_patterns() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
use tokio::time;

pub async fn good_async() {
    time::sleep(std::time::Duration::from_secs(1)).await;
}
",
    );

    let validator = AsyncPatternValidator::new(temp.path());
    let violations = validator.validate_blocking_in_async().unwrap();

    assert!(violations.is_empty(), "Proper async should pass");
}
