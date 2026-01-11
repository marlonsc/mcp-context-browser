//! Utility functions and helpers
//!
//! This module demonstrates clean code principles and TDD refactoring.
//! Contains well-tested utility functions that follow SOLID principles.

use std::collections::HashMap;

/// Validation result for input data
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult<T> {
    Valid(T),
    Invalid(String),
}

/// Generic validation trait for clean code
pub trait Validatable {
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

/// Collection utilities - demonstrates DRY principle
pub struct CollectionUtils;

impl CollectionUtils {
    /// Safe get with default - eliminates repetitive unwrap_or patterns
    pub fn get_or_default<T: Clone>(map: &HashMap<String, T>, key: &str, default: T) -> T {
        map.get(key).cloned().unwrap_or(default)
    }

    /// Safe get with custom default function - more flexible
    pub fn get_or_else<T: Clone, F>(map: &HashMap<String, T>, key: &str, default_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        map.get(key).cloned().unwrap_or_else(default_fn)
    }

    /// Check if collection is empty - extracted common check
    pub fn is_empty<T>(collection: &[T]) -> bool {
        collection.is_empty()
    }

    /// Safe indexing with bounds check - prevents panics
    pub fn get_safe<T: Clone>(slice: &[T], index: usize) -> Option<T> {
        slice.get(index).cloned()
    }
}

/// Error handling utilities - demonstrates consistent error patterns
pub struct ErrorUtils;

impl ErrorUtils {
    /// Create standardized error message - eliminates duplication
    pub fn format_error(context: &str, details: &str) -> String {
        format!("{}: {}", context, details)
    }

    /// Create validation error - consistent validation errors
    pub fn validation_error(field: &str, reason: &str) -> String {
        Self::format_error(&format!("Validation failed for {}", field), reason)
    }

    /// Create not found error - consistent not found errors
    pub fn not_found_error(resource: &str, identifier: &str) -> String {
        Self::format_error(
            &format!("{} not found", resource),
            &format!(
                "No {} found with identifier '{}'",
                resource.to_lowercase(),
                identifier
            ),
        )
    }
}

/// Configuration utilities - demonstrates parameter object pattern
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub min_length: usize,
    pub max_length: usize,
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
    /// Builder pattern for configuration - clean API
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_length(mut self, min: usize) -> Self {
        self.min_length = min;
        self
    }

    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

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
