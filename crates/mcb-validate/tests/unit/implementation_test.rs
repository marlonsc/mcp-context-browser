//! Tests for Implementation Quality Validation

use crate::test_utils::create_test_crate;
use mcb_validate::ImplementationQualityValidator;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_empty_method_detection() {
    let temp = TempDir::new().unwrap();
    // Use single-line format that the validator pattern matches
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub struct MyService;

impl MyService {
    pub fn do_nothing(&self) -> Result<(), Error> { Ok(()) }
}
",
    );

    let validator = ImplementationQualityValidator::new(temp.path());
    let violations = validator.validate_empty_methods().unwrap();

    assert!(!violations.is_empty(), "Should detect empty method body");
}

#[test]
fn test_stub_macro_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub fn not_implemented_yet(&self) {
    todo!("implement this")
}

pub fn also_not_done(&self) {
    unimplemented!()
}
"#,
    );

    let validator = ImplementationQualityValidator::new(temp.path());
    let violations = validator.validate_stub_macros().unwrap();

    assert_eq!(
        violations.len(),
        2,
        "Should detect both todo! and unimplemented!"
    );
}

#[test]
fn test_empty_catchall_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub fn handle_event(&self, event: Event) {
    match event {
        Event::Created => handle_created(),
        Event::Updated => handle_updated(),
        _ => {}
    }
}
",
    );

    let validator = ImplementationQualityValidator::new(temp.path());
    let violations = validator.validate_empty_catch_alls().unwrap();

    assert!(
        !violations.is_empty(),
        "Should detect empty catch-all _ => {{}}"
    );
}

#[test]
fn test_null_provider_exempt() {
    let temp = TempDir::new().unwrap();

    // Create a null provider file
    let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(
        crate_dir.join("null.rs"),
        r"
pub fn do_nothing(&self) -> Result<(), Error> {
    Ok(())
}
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

    let validator = ImplementationQualityValidator::new(temp.path());
    let violations = validator.validate_empty_methods().unwrap();

    assert!(
        violations.is_empty(),
        "Null provider files should be exempt"
    );
}
