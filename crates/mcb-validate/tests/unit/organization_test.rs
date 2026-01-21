//! Tests for Code Organization Validation

use crate::test_utils::create_test_crate;
use mcb_validate::OrganizationValidator;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_magic_number_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn process_data() {
    let timeout = 300000;  // magic number (5+ digits)
    let buffer_size = 163840;  // magic number (5+ digits)
}
",
    );

    let validator = OrganizationValidator::new(temp.path());
    let violations = validator.validate_magic_numbers().unwrap();

    assert!(!violations.is_empty(), "Should detect magic numbers");
}

#[test]
fn test_constants_file_exemption() {
    let temp = TempDir::new().unwrap();

    let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(
        crate_dir.join("constants.rs"),
        r"
pub const TIMEOUT_MS: u64 = 300000;
pub const BUFFER_SIZE: usize = 163840;
",
    )
    .unwrap();

    fs::write(
        temp.path()
            .join("crates")
            .join("mcb-test")
            .join("Cargo.toml"),
        r#"
[package]
name = "mcb-test"
version = "0.1.1"
"#,
    )
    .unwrap();

    let validator = OrganizationValidator::new(temp.path());
    let violations = validator.validate_magic_numbers().unwrap();

    assert!(violations.is_empty(), "Constants files should be exempt");
}
