//! Standalone tests for validation module to verify TDD implementation
//!
//! This test file is separate to avoid compilation issues with the main codebase.

#[cfg(test)]
mod validation_tests {
    use mcp_context_browser::core::validation::{ValidationError, ValidationResult, StringValidatorTrait, NumberValidatorTrait};
    use mcp_context_browser::core::validation::{StringValidator, NumberValidator};
    use mcp_context_browser::core::validation::common;

    #[test]
    fn test_string_validator_not_empty() {
        let validator = StringValidator::not_empty();
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
    }

    #[test]
    fn test_string_validator_min_length() {
        let validator = StringValidator::min_length(3);
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("hi").is_err());
    }

    #[test]
    fn test_string_validator_max_length() {
        let validator = StringValidator::max_length(5);
        assert!(validator.validate("hello").is_ok());
        assert!(validator.validate("hello world").is_err());
    }

    #[test]
    fn test_string_validator_contains() {
        let validator = StringValidator::contains("test");
        assert!(validator.validate("this is a test").is_ok());
        assert!(validator.validate("hello world").is_err());
    }

    #[test]
    fn test_string_validator_starts_with() {
        let validator = StringValidator::starts_with("http");
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("ftp://example.com").is_err());
    }

    #[test]
    fn test_number_validator_positive() {
        let validator = NumberValidator::positive();
        assert!(validator.validate(&5).is_ok());
        assert!(validator.validate(&0).is_err());
        assert!(validator.validate(&-1).is_err());
    }

    #[test]
    fn test_number_validator_range() {
        let validator = NumberValidator::range(10, 20);
        assert!(validator.validate(&15).is_ok());
        assert!(validator.validate(&5).is_err());
        assert!(validator.validate(&25).is_err());
    }

    #[test]
    fn test_number_validator_non_negative() {
        let validator = NumberValidator::non_negative();
        assert!(validator.validate(&0).is_ok());
        assert!(validator.validate(&5).is_ok());
        assert!(validator.validate(&-1).is_err());
    }

    #[test]
    fn test_number_validator_negative() {
        let validator = NumberValidator::negative();
        assert!(validator.validate(&-5).is_ok());
        assert!(validator.validate(&0).is_err());
        assert!(validator.validate(&5).is_err());
    }

    #[test]
    fn test_common_validators() {
        // Test username validator
        let username_validator = common::username();
        assert!(username_validator.validate("john_doe").is_ok());
        assert!(username_validator.validate("a").is_err()); // too short
        assert!(username_validator.validate("user@domain").is_err()); // invalid char

        // Test email-like validator
        let email_validator = common::email_like();
        assert!(email_validator.validate("user@domain.com").is_ok());
        assert!(email_validator.validate("invalid-email").is_err());

        // Test port validator
        let port_validator = common::port_number();
        assert!(port_validator.validate(&8080).is_ok());
        assert!(port_validator.validate(&0).is_err());
        assert!(port_validator.validate(&70000).is_err());

        // Test age validator
        let age_validator = common::age();
        assert!(age_validator.validate(&25).is_ok());
        assert!(age_validator.validate(&-5).is_err());
        assert!(age_validator.validate(&200).is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::Required { field: "name".to_string() };
        assert_eq!(error.to_string(), "Field 'name' is required");

        let error = ValidationError::TooShort {
            field: "password".to_string(),
            min_length: 8,
            actual_length: 5,
        };
        assert_eq!(error.to_string(), "Field 'password' is too short: 5 < 8");
    }
}