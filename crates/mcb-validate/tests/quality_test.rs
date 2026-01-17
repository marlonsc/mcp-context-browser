//! Tests for Quality Validation

mod test_utils;

use mcb_validate::QualityValidator;
use tempfile::TempDir;
use test_utils::create_test_crate;

#[test]
fn test_unwrap_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub fn risky_code() {
    let value = some_option().unwrap();
    let result = some_result().expect("should work");
}
"#,
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_no_unwrap_expect().unwrap();

    assert!(
        violations.len() >= 2,
        "Should detect unwrap() and expect() calls"
    );
}

#[test]
fn test_todo_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub fn incomplete() {
    // TODO: implement this properly
    // FIXME: this is broken
}
"#,
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    assert!(
        violations.len() >= 2,
        "Should detect TODO and FIXME comments"
    );
}

#[test]
fn test_test_code_exempt() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        let value = some_option().unwrap();
    }
}
"#,
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_no_unwrap_expect().unwrap();

    assert!(
        violations.is_empty(),
        "Test code should be exempt from unwrap checks"
    );
}
