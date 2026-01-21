//! Tests for Documentation Validation

use crate::test_utils::create_test_crate;
use mcb_validate::{DocumentationValidator, DocumentationViolation};
use tempfile::TempDir;

#[test]
fn test_missing_struct_doc() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
pub struct UndocumentedStruct {
    pub field: String,
}
",
    );

    let validator = DocumentationValidator::new(temp.path());
    let violations = validator.validate_pub_item_docs().unwrap();

    assert!(!violations.is_empty(), "Should detect missing struct doc");
    match &violations[0] {
        DocumentationViolation::MissingPubItemDoc { item_name, .. } => {
            assert_eq!(item_name, "UndocumentedStruct");
        }
        _ => panic!("Expected MissingPubItemDoc"),
    }
}

#[test]
fn test_documented_struct() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
/// A well-documented struct
pub struct DocumentedStruct {
    pub field: String,
}
",
    );

    let validator = DocumentationValidator::new(temp.path());
    let violations = validator.validate_pub_item_docs().unwrap();

    assert!(violations.is_empty(), "Documented structs should pass");
}
