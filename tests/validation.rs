//! Tests for input validation utilities
//!
//! This module demonstrates TDD approach - RED phase: write failing tests first

use mcp_context_browser::core::validation::{NumberValidator, StringValidator};
use mcp_context_browser::core::validation::{
    NumberValidatorTrait, StringValidatorTrait, ValidationError,
};

#[cfg(test)]
mod tests {
    use super::*;

    // ===== STRING VALIDATION TESTS =====

    #[test]
    fn test_string_validator_not_empty_should_pass_for_non_empty_string() {
        let validator = StringValidator::not_empty();
        let result = validator.validate("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_string_validator_not_empty_should_fail_for_empty_string() {
        let validator = StringValidator::not_empty();
        let result = validator.validate("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::Required { .. }
        ));
    }

    #[test]
    fn test_string_validator_not_empty_should_fail_for_whitespace_only() {
        let validator = StringValidator::not_empty();
        let result = validator.validate("   ");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::Required { .. }
        ));
    }

    #[test]
    fn test_string_validator_min_length_should_pass_when_above_minimum() {
        let validator = StringValidator::min_length(3);
        let result = validator.validate("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_string_validator_min_length_should_fail_when_below_minimum() {
        let validator = StringValidator::min_length(3);
        let result = validator.validate("hi");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::TooShort { .. }
        ));
    }

    #[test]
    fn test_string_validator_max_length_should_pass_when_below_maximum() {
        let validator = StringValidator::max_length(5);
        let result = validator.validate("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_string_validator_max_length_should_fail_when_above_maximum() {
        let validator = StringValidator::max_length(3);
        let result = validator.validate("hello");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::TooLong { .. }
        ));
    }

    #[test]
    fn test_string_validator_contains_should_pass_for_matching_substring() {
        let validator = StringValidator::contains("test");
        let result = validator.validate("this is a test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test");
    }

    #[test]
    fn test_string_validator_contains_should_fail_for_missing_substring() {
        let validator = StringValidator::contains("test");
        let result = validator.validate("hello world");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::InvalidFormat { .. }
        ));
    }

    #[test]
    fn test_string_validator_chained_rules_should_apply_all() {
        let validator = StringValidator::not_empty()
            .combine_with(StringValidator::min_length(3))
            .combine_with(StringValidator::max_length(10));

        // Should pass all rules
        let result = validator.validate("hello");
        assert!(result.is_ok());

        // Should fail first rule (empty)
        let result = validator.validate("");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::Required { .. }
        ));

        // Should fail second rule (too short)
        let result = validator.validate("hi");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::TooShort { .. }
        ));

        // Should fail third rule (too long)
        let result = validator.validate("this_is_too_long");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::TooLong { .. }
        ));
    }

    // ===== NUMBER VALIDATION TESTS =====

    #[test]
    fn test_number_validator_range_should_pass_within_bounds() {
        let validator = NumberValidator::range(10, 100);
        let result = validator.validate(&50);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_number_validator_range_should_fail_below_minimum() {
        let validator = NumberValidator::range(10, 100);
        let result = validator.validate(&5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::OutOfRange { .. }
        ));
    }

    #[test]
    fn test_number_validator_range_should_fail_above_maximum() {
        let validator = NumberValidator::range(10, 100);
        let result = validator.validate(&150);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::OutOfRange { .. }
        ));
    }

    #[test]
    fn test_number_validator_positive_should_pass_for_positive_number() {
        let validator = NumberValidator::positive();
        let result = validator.validate(&42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_number_validator_positive_should_fail_for_negative_number() {
        let validator = NumberValidator::positive();
        let result = validator.validate(&-5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::OutOfRange { .. }
        ));
    }

    #[test]
    fn test_number_validator_positive_should_fail_for_zero() {
        let validator = NumberValidator::positive();
        let result = validator.validate(&0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::OutOfRange { .. }
        ));
    }

    #[test]
    fn test_number_validator_non_negative_should_pass_for_positive_and_zero() {
        let validator = NumberValidator::non_negative();
        assert!(validator.validate(&42).is_ok());
        assert!(validator.validate(&0).is_ok());
    }

    #[test]
    fn test_number_validator_non_negative_should_fail_for_negative() {
        let validator = NumberValidator::non_negative();
        let result = validator.validate(&-5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::OutOfRange { .. }
        ));
    }

    // ===== COMPOSITE VALIDATION TESTS =====

    #[test]
    fn test_composite_validation_should_combine_validators() {
        // Create a composite validator for user registration
        let username_validator = StringValidator::not_empty()
            .combine_with(StringValidator::min_length(3))
            .combine_with(StringValidator::max_length(20))
            .combine_with(StringValidator::contains("_"));

        let age_validator = NumberValidator::range(13, 120);

        // Test valid input
        assert!(username_validator.validate("john_doe_123").is_ok());
        assert!(age_validator.validate(&25).is_ok());

        // Test invalid username (special chars)
        assert!(username_validator.validate("john@doe").is_err());

        // Test invalid age (too young)
        assert!(age_validator.validate(&10).is_err());
    }

    #[test]
    fn test_validation_error_messages_should_be_descriptive() {
        let validator = StringValidator::not_empty();
        let result = validator.validate("");
        match result {
            Err(ValidationError::Required { field }) => {
                assert_eq!(field, "input");
            }
            _ => panic!("Expected Required error"),
        }

        let validator = StringValidator::min_length(5);
        let result = validator.validate("hi");
        match result {
            Err(ValidationError::TooShort {
                field,
                min_length,
                actual_length,
            }) => {
                assert_eq!(field, "input");
                assert_eq!(min_length, 5);
                assert_eq!(actual_length, 2);
            }
            _ => panic!("Expected TooShort error"),
        }
    }
}
