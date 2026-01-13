//! Helpers for building view models - reduces duplication in ViewModelBuilder
//!
//! This module provides efficient builders for complex view model patterns that
//! repeat throughout the builder service. Each helper eliminates 10+ lines of code.

use super::view_models::*;
use crate::admin::models::AdminState;
use crate::admin::service::helpers::activity::ActivityLevel;
use crate::infrastructure::utils::{activity_level, FormattingUtils};
use anyhow::{Context, Result};

/// Helper for building configuration settings efficiently
pub struct ConfigSettingBuilder;

impl ConfigSettingBuilder {
    /// Create a number configuration setting
    pub fn number(
        key: &str,
        label: &str,
        value: impl std::fmt::Display,
        description: &str,
    ) -> ConfigSettingViewModel {
        let value_str = value.to_string();
        ConfigSettingViewModel {
            key: key.to_string(),
            label: label.to_string(),
            value: serde_json::json!(value_str.parse::<i64>().unwrap_or(0)),
            value_display: value_str,
            setting_type: "number",
            description: description.to_string(),
            editable: true,
        }
    }

    /// Create a boolean configuration setting
    pub fn boolean(
        key: &str,
        label: &str,
        value: bool,
        description: &str,
    ) -> ConfigSettingViewModel {
        let value_display = Self::format_bool(value);
        ConfigSettingViewModel {
            key: key.to_string(),
            label: label.to_string(),
            value: serde_json::json!(value),
            value_display,
            setting_type: "boolean",
            description: description.to_string(),
            editable: true,
        }
    }

    /// Create a bytes configuration setting (with formatting)
    pub fn bytes(key: &str, label: &str, value: u64, description: &str) -> ConfigSettingViewModel {
        ConfigSettingViewModel {
            key: key.to_string(),
            label: label.to_string(),
            value: serde_json::json!(value),
            value_display: FormattingUtils::format_bytes(value),
            setting_type: "number",
            description: description.to_string(),
            editable: true,
        }
    }

    /// Create a string configuration setting
    pub fn string(
        key: &str,
        label: &str,
        value: &str,
        description: &str,
    ) -> ConfigSettingViewModel {
        ConfigSettingViewModel {
            key: key.to_string(),
            label: label.to_string(),
            value: serde_json::json!(value),
            value_display: value.to_string(),
            setting_type: "text",
            description: description.to_string(),
            editable: true,
        }
    }

    /// Format boolean as "Enabled" or "Disabled"
    fn format_bool(value: bool) -> String {
        if value { "Enabled" } else { "Disabled" }.to_string()
    }
}

/// Helper for creating configuration categories with settings
pub struct ConfigCategoryBuilder;

impl ConfigCategoryBuilder {
    /// Create a category with settings
    ///
    /// This builder returns ConfigCategoryViewModel (not Self) because it's a factory
    /// method that constructs view models from configuration data, not the builder itself.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: &str,
        description: &str,
        settings: Vec<ConfigSettingViewModel>,
    ) -> ConfigCategoryViewModel {
        ConfigCategoryViewModel {
            name: name.to_string(),
            description: description.to_string(),
            settings,
        }
    }
}

/// Helper for system metrics collection
pub struct MetricsCollector<'a> {
    state: &'a AdminState,
}

impl<'a> MetricsCollector<'a> {
    /// Create a new metrics collector
    pub fn new(state: &'a AdminState) -> Self {
        Self { state }
    }

    /// Collect both CPU and memory metrics in one call
    pub async fn collect_system(&self) -> Result<(f64, f64)> {
        let cpu = self
            .state
            .mcp_server
            .system_collector
            .collect_cpu_metrics()
            .await
            .context("Failed to collect CPU metrics")?;

        let memory = self
            .state
            .mcp_server
            .system_collector
            .collect_memory_metrics()
            .await
            .context("Failed to collect memory metrics")?;

        Ok((cpu.usage as f64, memory.usage_percent as f64))
    }
}

/// Helper for activity level formatting
pub struct ActivityLevelFormatter;

impl ActivityLevelFormatter {
    /// Convert ActivityLevel to CSS class string
    pub fn to_css_class(level: ActivityLevel) -> &'static str {
        match level {
            ActivityLevel::Success => activity_level::SUCCESS,
            ActivityLevel::Warning => activity_level::WARNING,
            ActivityLevel::Error => activity_level::ERROR,
            ActivityLevel::Info => activity_level::INFO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_setting_builder_number() {
        let setting =
            ConfigSettingBuilder::number("chunk_size", "Chunk Size", 512, "Size of chunks");
        assert_eq!(setting.key, "chunk_size");
        assert_eq!(setting.label, "Chunk Size");
        assert_eq!(setting.value_display, "512");
        assert_eq!(setting.setting_type, "number");
    }

    #[test]
    fn test_config_setting_builder_boolean() {
        let setting = ConfigSettingBuilder::boolean(
            "enable_auth",
            "Enable Auth",
            true,
            "Enable authentication",
        );
        assert_eq!(setting.key, "enable_auth");
        assert_eq!(setting.value_display, "Enabled");
        assert_eq!(setting.setting_type, "boolean");

        let disabled = ConfigSettingBuilder::boolean(
            "enable_auth",
            "Enable Auth",
            false,
            "Enable authentication",
        );
        assert_eq!(disabled.value_display, "Disabled");
    }

    #[test]
    fn test_config_setting_builder_bytes() {
        let setting =
            ConfigSettingBuilder::bytes("max_file_size", "Max File Size", 1_048_576, "Max size");
        assert_eq!(setting.value_display, "1.0 MB");
    }

    #[test]
    fn test_config_category_builder() {
        let settings = vec![ConfigSettingBuilder::number(
            "key1",
            "Label 1",
            100,
            "Description 1",
        )];
        let category = ConfigCategoryBuilder::new("Test Category", "Test description", settings);
        assert_eq!(category.name, "Test Category");
        assert_eq!(category.description, "Test description");
        assert_eq!(category.settings.len(), 1);
    }

    #[test]
    fn test_activity_level_formatter() {
        assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Success).contains("success"));
        assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Error).contains("error"));
        assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Warning).contains("warning"));
        assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Info).contains("info"));
    }
}
