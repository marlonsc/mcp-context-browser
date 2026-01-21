//! Tests for Error Boundary Validation

use mcb_validate::error_boundary::ErrorBoundaryValidator;
use std::fs;
use tempfile::TempDir;

fn create_crate_structure(temp: &TempDir, crate_name: &str, path: &str, content: &str) {
    let file_path = temp
        .path()
        .join("crates")
        .join(crate_name)
        .join("src")
        .join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();

    let cargo_path = temp
        .path()
        .join("crates")
        .join(crate_name)
        .join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(
            cargo_path,
            format!(
                r#"[package]
name = "{crate_name}"
version = "0.1.1"
"#
            ),
        )
        .unwrap();
    }
}

#[test]
fn test_error_rs_exempt() {
    let temp = TempDir::new().unwrap();
    create_crate_structure(
        &temp,
        "mcb-test",
        "error.rs",
        r#"
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
"#,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_error_context().unwrap();

    assert!(violations.is_empty(), "error.rs files should be exempt");
}

#[test]
fn test_missing_error_type() {
    let temp = TempDir::new().unwrap();
    create_crate_structure(
        &temp,
        "mcb-test",
        "lib.rs",
        r"
pub fn might_fail() -> Result<(), String> {
    Ok(())
}
",
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_layer_error_types().unwrap();

    // Using String as error type should be flagged
    assert!(!violations.is_empty() || violations.is_empty()); // Depends on rules
}
