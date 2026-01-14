//! Validation utilities - demonstrates Extract Method refactoring (DRY)
//!
//! Eliminates duplication in input validation across the codebase

/// Validation result for input data
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult<T> {
    /// Input data passed validation
    Valid(T),
    /// Input data failed validation with error message
    Invalid(String),
}

/// Generic validation trait for clean code
pub trait Validatable {
    /// Validate this instance and return the result
    fn validate(&self) -> ValidationResult<Self>
    where
        Self: Clone;
}

/// String validation utilities - demonstrates Extract Method refactoring
pub struct StringValidator;

impl StringValidator {
    /// Validate string is not empty - extracted from multiple locations
    pub fn validate_not_empty(s: &str) -> ValidationResult<String> {
        if s.trim().is_empty() {
            ValidationResult::Invalid("String cannot be empty".to_string())
        } else {
            ValidationResult::Valid(s.to_string())
        }
    }

    /// Validate string length - extracted common validation logic
    pub fn validate_length(s: &str, min: usize, max: usize) -> ValidationResult<String> {
        let len = s.len();
        if len < min {
            ValidationResult::Invalid(format!("String too short: {} < {}", len, min))
        } else if len > max {
            ValidationResult::Invalid(format!("String too long: {} > {}", len, max))
        } else {
            ValidationResult::Valid(s.to_string())
        }
    }

    /// Validate string contains only alphanumeric characters - clean validation
    pub fn validate_alphanumeric(s: &str) -> ValidationResult<String> {
        if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            ValidationResult::Valid(s.to_string())
        } else {
            ValidationResult::Invalid("String contains invalid characters".to_string())
        }
    }
}

/// Configuration utilities - demonstrates parameter object pattern
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum allowed length for validated strings
    pub min_length: usize,
    /// Maximum allowed length for validated strings
    pub max_length: usize,
    /// Whether special characters are allowed in validated strings
    pub allow_special_chars: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            min_length: 1,
            max_length: 100,
            allow_special_chars: false,
        }
    }
}

impl ValidationConfig {
    /// Create a new validation configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum allowed length for validated strings
    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    /// Set the maximum allowed length for validated strings
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Configure whether special characters are allowed in validated strings
    pub fn allow_special_chars(mut self, allow: bool) -> Self {
        self.allow_special_chars = allow;
        self
    }
}

/// Advanced validation with configuration - demonstrates strategy pattern
pub struct ConfigurableValidator {
    config: ValidationConfig,
}

impl ConfigurableValidator {
    /// Create a new configurable validator with specified settings
    ///
    /// # Arguments
    /// * `config` - Validation configuration specifying length limits and character restrictions
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate string with configuration - flexible validation
    pub fn validate_string(&self, s: &str) -> ValidationResult<String> {
        // Check length
        match StringValidator::validate_length(s, self.config.min_length, self.config.max_length) {
            ValidationResult::Valid(_) => {}
            ValidationResult::Invalid(e) => return ValidationResult::Invalid(e),
        }

        // Check characters if special chars not allowed
        if !self.config.allow_special_chars {
            match StringValidator::validate_alphanumeric(s) {
                ValidationResult::Valid(_) => {}
                ValidationResult::Invalid(e) => return ValidationResult::Invalid(e),
            }
        }

        ValidationResult::Valid(s.to_string())
    }
}
