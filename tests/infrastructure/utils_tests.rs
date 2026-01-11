//! Utility function tests
//!
//! Tests migrated from src/infrastructure/utils.rs

use mcp_context_browser::infrastructure::utils::{
    CollectionUtils, ConfigurableValidator, ErrorUtils, StringValidator, ValidationConfig,
    ValidationResult,
};

#[test]
fn test_string_validator_not_empty() {
    assert!(matches!(
        StringValidator::validate_not_empty("hello"),
        ValidationResult::Valid(_)
    ));
    assert!(matches!(
        StringValidator::validate_not_empty(""),
        ValidationResult::Invalid(_)
    ));
    assert!(matches!(
        StringValidator::validate_not_empty("   "),
        ValidationResult::Invalid(_)
    ));
}

#[test]
fn test_string_validator_length() {
    assert!(matches!(
        StringValidator::validate_length("hi", 1, 10),
        ValidationResult::Valid(_)
    ));
    assert!(matches!(
        StringValidator::validate_length("", 1, 10),
        ValidationResult::Invalid(_)
    ));
    assert!(matches!(
        StringValidator::validate_length("this_is_a_very_long_string", 1, 10),
        ValidationResult::Invalid(_)
    ));
}

#[test]
fn test_collection_utils_safe_get() {
    let slice = vec![1, 2, 3];
    assert_eq!(CollectionUtils::get_safe(&slice, 0), Some(1));
    assert_eq!(CollectionUtils::get_safe(&slice, 5), None);
}

#[test]
fn test_configurable_validator() {
    let config = ValidationConfig::new()
        .min_length(3)
        .max_length(10)
        .allow_special_chars(false);

    let validator = ConfigurableValidator::new(config);

    assert!(matches!(
        validator.validate_string("hello"),
        ValidationResult::Valid(_)
    ));
    assert!(matches!(
        validator.validate_string("hi"),
        ValidationResult::Invalid(_)
    )); // too short
    assert!(matches!(
        validator.validate_string("hello@world"),
        ValidationResult::Invalid(_)
    )); // special char
}

#[test]
fn test_error_utils_formatting() {
    let error = ErrorUtils::validation_error("username", "cannot be empty");
    assert!(error.contains("username"));
    assert!(error.contains("cannot be empty"));
}
