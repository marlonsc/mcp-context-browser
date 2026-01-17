//! JSON Value Extension
//!
//! Provides convenient accessor methods for JSON values with default fallbacks.

use std::collections::HashMap;

/// Extension trait for serde_json::Value with convenient accessor methods
///
/// Replaces the verbose pattern:
/// ```rust
/// use serde_json::json;
///
/// let meta = json!({"key": "value"});
/// let value = meta.get("key").and_then(|v| v.as_str()).unwrap_or("default");
/// assert_eq!(value, "value");
/// ```
///
/// # Example
///
/// ```rust
/// use serde_json::json;
/// use mcb_providers::utils::JsonExt;
///
/// let meta = json!({"key": "value", "count": 42, "enabled": true});
/// let value = meta.str_or("key", "default");
/// assert_eq!(value, "value");
///
/// let count = meta.i64_or("count", 0);
/// assert_eq!(count, 42);
///
/// let enabled = meta.bool_or("enabled", false);
/// assert!(enabled);
/// ```
pub trait JsonExt {
    /// Get string value or default (replaces .get().and_then(as_str).unwrap_or)
    fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str;

    /// Get owned string value or default
    fn string_or(&self, key: &str, default: &str) -> String;

    /// Get i64 value or default
    fn i64_or(&self, key: &str, default: i64) -> i64;

    /// Get u64 value or default
    fn u64_or(&self, key: &str, default: u64) -> u64;

    /// Get f64 value or default
    fn f64_or(&self, key: &str, default: f64) -> f64;

    /// Get bool value or default
    fn bool_or(&self, key: &str, default: bool) -> bool;

    /// Get optional string (replaces .get().and_then(as_str))
    fn opt_str(&self, key: &str) -> Option<&str>;

    /// Get optional i64
    fn opt_i64(&self, key: &str) -> Option<i64>;

    /// Get optional u64
    fn opt_u64(&self, key: &str) -> Option<u64>;
}

/// Internal trait for types that can be used as JSON-like containers
trait JsonContainer {
    fn get_value(&self, key: &str) -> Option<&serde_json::Value>;
}

impl JsonContainer for serde_json::Value {
    #[inline]
    fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }
}

impl JsonContainer for HashMap<String, serde_json::Value> {
    #[inline]
    fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }
}

impl JsonContainer for serde_json::Map<String, serde_json::Value> {
    #[inline]
    fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }
}

/// Macro to implement JsonExt for types that implement JsonContainer
macro_rules! impl_json_ext {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl JsonExt for $ty {
                #[inline]
                fn str_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
                    self.get_value(key).and_then(|v| v.as_str()).unwrap_or(default)
                }

                #[inline]
                fn string_or(&self, key: &str, default: &str) -> String {
                    self.get_value(key)
                        .and_then(|v| v.as_str())
                        .unwrap_or(default)
                        .to_string()
                }

                #[inline]
                fn i64_or(&self, key: &str, default: i64) -> i64 {
                    self.get_value(key).and_then(|v| v.as_i64()).unwrap_or(default)
                }

                #[inline]
                fn u64_or(&self, key: &str, default: u64) -> u64 {
                    self.get_value(key).and_then(|v| v.as_u64()).unwrap_or(default)
                }

                #[inline]
                fn f64_or(&self, key: &str, default: f64) -> f64 {
                    self.get_value(key).and_then(|v| v.as_f64()).unwrap_or(default)
                }

                #[inline]
                fn bool_or(&self, key: &str, default: bool) -> bool {
                    self.get_value(key).and_then(|v| v.as_bool()).unwrap_or(default)
                }

                #[inline]
                fn opt_str(&self, key: &str) -> Option<&str> {
                    self.get_value(key).and_then(|v| v.as_str())
                }

                #[inline]
                fn opt_i64(&self, key: &str) -> Option<i64> {
                    self.get_value(key).and_then(|v| v.as_i64())
                }

                #[inline]
                fn opt_u64(&self, key: &str) -> Option<u64> {
                    self.get_value(key).and_then(|v| v.as_u64())
                }
            }
        )+
    };
}

impl_json_ext!(
    serde_json::Value,
    HashMap<String, serde_json::Value>,
    serde_json::Map<String, serde_json::Value>,
);
