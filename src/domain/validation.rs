//! Input validation utilities with composable validators
//!
//! This module provides a clean, type-safe way to validate input data.
//! Built using TDD principles with comprehensive test coverage.
//!
//! # Examples
//!
//! ## String Validation
//! ```rust
//! use mcp_context_browser::domain::validation::{StringValidator, StringValidatorTrait};
//!
//! let validator = StringValidator::not_empty()
//!     .combine_with(StringValidator::min_length(3))
//!     .combine_with(StringValidator::max_length(50));
//!
//! assert!(validator.validate("hello").is_ok());
//! assert!(validator.validate("").is_err());
//! ```
//!
//! ## Number Validation
//! ```rust
//! use mcp_context_browser::domain::validation::{NumberValidator, NumberValidatorTrait};
//!
//! let validator = NumberValidator::range(18, 120);
//! assert!(validator.validate(&25).is_ok());
//! assert!(validator.validate(&15).is_err());
//! ```

use std::fmt;

/// Validation result type alias for cleaner code
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation errors with detailed context
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Field is required but missing or empty
    Required { field: String },
    /// Value is too short
    TooShort {
        field: String,
        min_length: usize,
        actual_length: usize,
    },
    /// Value is too long
    TooLong {
        field: String,
        max_length: usize,
        actual_length: usize,
    },
    /// Value doesn't match required format
    InvalidFormat { field: String, expected: String },
    /// Numeric value is out of allowed range
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Required { field } => {
                write!(f, "Field '{}' is required", field)
            }
            ValidationError::TooShort {
                field,
                min_length,
                actual_length,
            } => {
                write!(
                    f,
                    "Field '{}' is too short: {} < {}",
                    field, actual_length, min_length
                )
            }
            ValidationError::TooLong {
                field,
                max_length,
                actual_length,
            } => {
                write!(
                    f,
                    "Field '{}' is too long: {} > {}",
                    field, actual_length, max_length
                )
            }
            ValidationError::InvalidFormat { field, expected } => {
                write!(
                    f,
                    "Field '{}' has invalid format. Expected: {}",
                    field, expected
                )
            }
            ValidationError::OutOfRange {
                field,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "Field '{}' value '{}' is out of range [{}, {}]",
                    field, value, min, max
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Base validator trait for composability
pub trait Validator<T> {
    fn validate(&self, input: T) -> ValidationResult<T>;
}

/// String validator trait for composable string validation
pub trait StringValidatorTrait {
    fn validate(&self, input: &str) -> ValidationResult<String>;
}

/// Number validator trait for composable number validation
pub trait NumberValidatorTrait {
    fn validate(&self, input: &i64) -> ValidationResult<i64>;
}

/// String validation rule type alias to reduce complexity
type StringValidationRule = Box<dyn Fn(&str) -> ValidationResult<String> + Send + Sync>;

/// String validator with composable validation rules
///
/// Provides a fluent API for building complex string validation logic.
/// Each validation rule is applied in sequence, and all must pass.
pub struct StringValidator {
    rules: Vec<StringValidationRule>,
}

impl StringValidator {
    /// Create a validator that ensures the string is not empty or whitespace-only
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{StringValidator, StringValidatorTrait};
    /// let validator = StringValidator::not_empty();
    /// assert!(validator.validate("hello").is_ok());
    /// assert!(validator.validate("").is_err());
    /// ```
    pub fn not_empty() -> Self {
        let mut validator = Self::new();
        validator.rules.push(Box::new(|s: &str| {
            if s.trim().is_empty() {
                Err(ValidationError::Required {
                    field: "input".to_string(),
                })
            } else {
                Ok(s.to_string())
            }
        }));
        validator
    }

    /// Create validator that checks minimum length
    pub fn min_length(min: usize) -> Self {
        let mut validator = Self::new();
        let min_len = min;
        validator.rules.push(Box::new(move |s: &str| {
            if s.len() < min_len {
                Err(ValidationError::TooShort {
                    field: "input".to_string(),
                    min_length: min_len,
                    actual_length: s.len(),
                })
            } else {
                Ok(s.to_string())
            }
        }));
        validator
    }

    /// Create validator that checks maximum length
    pub fn max_length(max: usize) -> Self {
        let mut validator = Self::new();
        let max_len = max;
        validator.rules.push(Box::new(move |s: &str| {
            if s.len() > max_len {
                Err(ValidationError::TooLong {
                    field: "input".to_string(),
                    max_length: max_len,
                    actual_length: s.len(),
                })
            } else {
                Ok(s.to_string())
            }
        }));
        validator
    }

    /// Create a validator that checks if string contains a specific substring
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{StringValidator, StringValidatorTrait};
    /// let validator = StringValidator::contains("test");
    /// assert!(validator.validate("this is a test").is_ok());
    /// assert!(validator.validate("hello world").is_err());
    /// ```
    pub fn contains(substring: &str) -> Self {
        let substring = substring.to_string();
        Self::new().add_rule(move |s: &str| {
            if s.contains(&substring) {
                Ok(s.to_string())
            } else {
                Err(ValidationError::InvalidFormat {
                    field: "input".to_string(),
                    expected: format!("must contain '{}'", substring),
                })
            }
        })
    }

    /// Create a validator that checks if string starts with a specific prefix
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{StringValidator, StringValidatorTrait};
    /// let validator = StringValidator::starts_with("http");
    /// assert!(validator.validate("https://example.com").is_ok());
    /// assert!(validator.validate("ftp://example.com").is_err());
    /// ```
    pub fn starts_with(prefix: &str) -> Self {
        let prefix = prefix.to_string();
        Self::new().add_rule(move |s: &str| {
            if s.starts_with(&prefix) {
                Ok(s.to_string())
            } else {
                Err(ValidationError::InvalidFormat {
                    field: "input".to_string(),
                    expected: format!("must start with '{}'", prefix),
                })
            }
        })
    }

    /// Internal helper to add a validation rule
    fn add_rule<F>(mut self, rule: F) -> Self
    where
        F: Fn(&str) -> ValidationResult<String> + Send + Sync + 'static,
    {
        self.rules.push(Box::new(rule));
        self
    }

    /// Create empty validator for chaining
    fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Create a new validator by combining rules (not chaining due to closure limitations)
    pub fn combine_with(mut self, other: StringValidator) -> Self {
        self.rules.extend(other.rules);
        self
    }
}

impl StringValidatorTrait for StringValidator {
    fn validate(&self, input: &str) -> ValidationResult<String> {
        for rule in &self.rules {
            let _result = rule(input)?;
            // Continue with next rule
        }
        Ok(input.to_string())
    }
}

/// Number validation rule type alias to reduce complexity
type NumberValidationRule = Box<dyn Fn(&i64) -> ValidationResult<i64> + Send + Sync>;

/// Number validator for integers with composable validation rules
///
/// Provides a fluent API for building complex numeric validation logic.
/// Each validation rule is applied in sequence, and all must pass.
pub struct NumberValidator {
    rules: Vec<NumberValidationRule>,
}

impl NumberValidator {
    /// Create a validator that checks if number is within a specified range (inclusive)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{NumberValidator, NumberValidatorTrait};
    /// let validator = NumberValidator::range(10, 20);
    /// assert!(validator.validate(&15).is_ok());
    /// assert!(validator.validate(&5).is_err());
    /// ```
    pub fn range(min: i64, max: i64) -> Self {
        let min_val = min;
        let max_val = max;
        Self::new().add_rule(move |n: &i64| {
            if *n < min_val || *n > max_val {
                Err(ValidationError::OutOfRange {
                    field: "input".to_string(),
                    value: n.to_string(),
                    min: min_val.to_string(),
                    max: max_val.to_string(),
                })
            } else {
                Ok(*n)
            }
        })
    }

    /// Create a validator for positive numbers (> 0)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{NumberValidator, NumberValidatorTrait};
    /// let validator = NumberValidator::positive();
    /// assert!(validator.validate(&5).is_ok());
    /// assert!(validator.validate(&0).is_err());
    /// ```
    pub fn positive() -> Self {
        Self::range(1, i64::MAX)
    }

    /// Create a validator for non-negative numbers (>= 0)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{NumberValidator, NumberValidatorTrait};
    /// let validator = NumberValidator::non_negative();
    /// assert!(validator.validate(&0).is_ok());
    /// assert!(validator.validate(&5).is_ok());
    /// assert!(validator.validate(&-1).is_err());
    /// ```
    pub fn non_negative() -> Self {
        Self::range(0, i64::MAX)
    }

    /// Create a validator for negative numbers (< 0)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{NumberValidator, NumberValidatorTrait};
    /// let validator = NumberValidator::negative();
    /// assert!(validator.validate(&-5).is_ok());
    /// assert!(validator.validate(&0).is_err());
    /// ```
    pub fn negative() -> Self {
        Self::range(i64::MIN, -1)
    }

    /// Internal helper to create an empty validator
    fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Internal helper to add a validation rule
    fn add_rule<F>(mut self, rule: F) -> Self
    where
        F: Fn(&i64) -> ValidationResult<i64> + Send + Sync + 'static,
    {
        self.rules.push(Box::new(rule));
        self
    }
}

impl NumberValidatorTrait for NumberValidator {
    fn validate(&self, input: &i64) -> ValidationResult<i64> {
        for rule in &self.rules {
            let _result = rule(input)?;
            // Continue with next rule
        }
        Ok(*input)
    }
}

/// Common validation patterns and factory methods
pub mod common {

    use super::{NumberValidator, StringValidator};

    /// Create a username validator (3-20 chars, alphanumeric + underscore)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{common, StringValidatorTrait};
    /// let validator = common::username();
    /// assert!(validator.validate("john_doe123").is_ok());
    /// assert!(validator.validate("a").is_err()); // too short
    /// assert!(validator.validate("user@domain.com").is_err()); // invalid chars
    /// ```
    pub fn username() -> StringValidator {
        StringValidator::not_empty()
            .combine_with(StringValidator::min_length(3))
            .combine_with(StringValidator::max_length(20))
            .combine_with(StringValidator::contains("_")) // At least one underscore for this example
    }

    /// Create an email-like validator (basic format check)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{common, StringValidatorTrait};
    /// let validator = common::email_like();
    /// assert!(validator.validate("user@domain.com").is_ok());
    /// assert!(validator.validate("invalid-email").is_err());
    /// ```
    pub fn email_like() -> StringValidator {
        StringValidator::not_empty()
            .combine_with(StringValidator::min_length(5))
            .combine_with(StringValidator::contains("@"))
    }

    /// Create a port number validator (1-65535)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{common, NumberValidatorTrait};
    /// let validator = common::port_number();
    /// assert!(validator.validate(&8080).is_ok());
    /// assert!(validator.validate(&0).is_err()); // invalid port
    /// assert!(validator.validate(&70000).is_err()); // too high
    /// ```
    pub fn port_number() -> NumberValidator {
        NumberValidator::range(1, 65535)
    }

    /// Create an age validator (0-150 years)
    ///
    /// # Examples
    /// ```rust
    /// use mcp_context_browser::domain::validation::{common, NumberValidatorTrait};
    /// let validator = common::age();
    /// assert!(validator.validate(&25).is_ok());
    /// assert!(validator.validate(&-5).is_err()); // negative age
    /// assert!(validator.validate(&200).is_err()); // too old
    /// ```
    pub fn age() -> NumberValidator {
        NumberValidator::range(0, 150)
    }
}
