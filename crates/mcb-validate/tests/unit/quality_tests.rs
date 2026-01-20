//! Unit tests for `mcb_validate::quality` module

use mcb_validate::{QualityValidator, QualityViolation};
use std::fmt::Write;
use std::fs;
use tempfile::TempDir;

fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join("lib.rs"), content).unwrap();

    // Create Cargo.toml
    let cargo_dir = temp.path().join("crates").join(name);
    fs::write(
        cargo_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.1"
"#
        ),
    )
    .unwrap();
}

#[test]
fn test_unwrap_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn bad_function() {
    let x: Option<i32> = Some(1);
    let _ = x.unwrap();
}
",
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_no_unwrap_expect().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        QualityViolation::UnwrapInProduction { line, .. } => {
            assert_eq!(*line, 4);
        }
        _ => panic!("Expected UnwrapInProduction"),
    }
}

#[test]
fn test_safe_unwrap_patterns() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn good_function() {
    let x: Option<i32> = Some(1);
    let _ = x.unwrap_or(0);
    let _ = x.unwrap_or_else(|| 0);
    let _ = x.unwrap_or_default();
}
",
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_no_unwrap_expect().unwrap();

    assert!(
        violations.is_empty(),
        "Safe patterns should not trigger violations: {violations:?}"
    );
}

#[test]
fn test_test_module_exemption() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn good_function() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        let x = Some(1).unwrap();
        assert_eq!(x, 1);
    }
}
",
    );

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_no_unwrap_expect().unwrap();

    assert!(
        violations.is_empty(),
        "Test modules should be exempt: {violations:?}"
    );
}

#[test]
fn test_file_size_validation() {
    let temp = TempDir::new().unwrap();
    let content = (0..600).fold(String::new(), |mut acc, i| {
        let _ = writeln!(acc, "// line {i}");
        acc
    });
    create_test_crate(&temp, "mcb-test", &content);

    let validator = QualityValidator::new(temp.path());
    let violations = validator.validate_file_sizes().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        QualityViolation::FileTooLarge { lines, .. } => {
            assert!(*lines > 500);
        }
        _ => panic!("Expected FileTooLarge"),
    }
}

#[test]
fn test_todo_detection() {
    let temp = TempDir::new().unwrap();
    // Build test content dynamically to avoid SATD detection in this source file
    // The validator should still detect these patterns in the generated test crate
    let todo_marker = ["TO", "DO"].join(""); // T-O-D-O
    let fixme_marker = ["FIX", "ME"].join(""); // F-I-X-M-E
    let msg1 = ["implement", "this"].join(" ");
    let msg2 = ["needs", "repair"].join(" ");
    let test_content = format!(
        "// {todo_marker}: {msg1}\npub fn incomplete() {{}}\n\n// {fixme_marker}: {msg2}\npub fn needs_work() {{}}"
    );
    create_test_crate(&temp, "mcb-test", &test_content);

    let validator = QualityValidator::new(temp.path());
    let violations = validator.find_todo_comments().unwrap();

    assert_eq!(violations.len(), 2);
}
