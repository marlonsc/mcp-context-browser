//! Tests for SOLID Principles Validation

use mcb_validate::{SolidValidator, SolidViolation};
use tempfile::TempDir;

use crate::test_utils::create_test_crate;

#[test]
fn test_large_trait_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub trait TooManyMethods {
    fn method1(&self);
    fn method2(&self);
    fn method3(&self);
    fn method4(&self);
    fn method5(&self);
    fn method6(&self);
    fn method7(&self);
    fn method8(&self);
    fn method9(&self);
    fn method10(&self);
    fn method11(&self);
    fn method12(&self);
}
",
    );

    let validator = SolidValidator::new(temp.path());
    let violations = validator.validate_isp().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        SolidViolation::TraitTooLarge { method_count, .. } => {
            assert!(*method_count > 10);
        }
        _ => panic!("Expected TraitTooLarge"),
    }
}

#[test]
fn test_partial_impl_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub trait MyTrait {
    fn do_something(&self);
}

pub struct MyStruct;

impl MyTrait for MyStruct {
    fn do_something(&self) {
        todo!("not implemented")
    }
}
"#,
    );

    let validator = SolidValidator::new(temp.path());
    let violations = validator.validate_lsp().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        SolidViolation::PartialTraitImplementation { method_name, .. } => {
            assert_eq!(method_name, "do_something");
        }
        _ => panic!("Expected PartialTraitImplementation"),
    }
}
