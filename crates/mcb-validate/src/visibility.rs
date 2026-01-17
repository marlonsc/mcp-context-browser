//! Module Visibility Validation
//!
//! Validates proper use of pub(crate), pub, and private visibility.

use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationConfig};
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Visibility Violations
#[derive(Debug, Clone, Serialize)]
pub enum VisibilityViolation {
    InternalHelperTooPublic {
        item_name: String,
        file: PathBuf,
        line: usize,
    },
    DomainTypeTooRestricted {
        type_name: String,
        file: PathBuf,
        line: usize,
    },
    UtilityModuleTooPublic {
        module_name: String,
        file: PathBuf,
        line: usize,
    },
}

impl std::fmt::Display for VisibilityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalHelperTooPublic {
                item_name,
                file,
                line,
            } => write!(
                f,
                "Internal helper {} is pub at {}:{} - consider pub(crate)",
                item_name,
                file.display(),
                line
            ),
            Self::DomainTypeTooRestricted {
                type_name,
                file,
                line,
            } => write!(
                f,
                "Domain type {} is pub(crate) at {}:{} - should be pub",
                type_name,
                file.display(),
                line
            ),
            Self::UtilityModuleTooPublic {
                module_name,
                file,
                line,
            } => write!(
                f,
                "Utility module {} has pub items at {}:{}",
                module_name,
                file.display(),
                line
            ),
        }
    }
}

impl Violation for VisibilityViolation {
    fn id(&self) -> &str {
        match self {
            Self::InternalHelperTooPublic { .. } => "VIS001",
            Self::DomainTypeTooRestricted { .. } => "VIS002",
            Self::UtilityModuleTooPublic { .. } => "VIS003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Organization
    }

    fn severity(&self) -> Severity {
        match self {
            Self::InternalHelperTooPublic { .. } | Self::UtilityModuleTooPublic { .. } => {
                Severity::Info
            }
            Self::DomainTypeTooRestricted { .. } => Severity::Warning,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::InternalHelperTooPublic { file, .. }
            | Self::DomainTypeTooRestricted { file, .. }
            | Self::UtilityModuleTooPublic { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::InternalHelperTooPublic { line, .. }
            | Self::DomainTypeTooRestricted { line, .. }
            | Self::UtilityModuleTooPublic { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::InternalHelperTooPublic { .. } => {
                Some("Use pub(crate) for internal helpers".to_string())
            }
            Self::DomainTypeTooRestricted { .. } => {
                Some("Domain types should use pub for external use".to_string())
            }
            Self::UtilityModuleTooPublic { .. } => {
                Some("Consider pub(crate) for utility modules".to_string())
            }
        }
    }
}

/// Visibility Validator
pub struct VisibilityValidator;

impl Default for VisibilityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibilityValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.check_internal_helpers(config)?);
        violations.extend(self.check_utility_modules(config)?);
        Ok(violations)
    }

    fn check_internal_helpers(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();
        let internal_dirs = [
            "crates/mcb-infrastructure/src/utils",
            "crates/mcb-providers/src/utils",
            "crates/mcb-server/src/utils",
            "crates/mcb-application/src/utils",
        ];

        let pub_item_re = Regex::new(r"^pub\s+(fn|struct|enum|type|const|static)\s+(\w+)")
            .expect("Invalid regex");
        let pub_crate_re = Regex::new(r"^pub\(crate\)").expect("Invalid regex");

        for dir_path in &internal_dirs {
            let full_path = config.workspace_root.join(dir_path);
            if !full_path.exists() {
                continue;
            }

            for entry in WalkDir::new(&full_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if pub_crate_re.is_match(trimmed) {
                        continue;
                    }

                    if let Some(captures) = pub_item_re.captures(trimmed) {
                        let item_name = captures.get(2).map(|m| m.as_str()).unwrap_or("unknown");
                        if item_name.starts_with("Error") || item_name == "Result" {
                            continue;
                        }

                        violations.push(VisibilityViolation::InternalHelperTooPublic {
                            item_name: item_name.to_string(),
                            file: path.to_path_buf(),
                            line: line_num + 1,
                        });
                    }
                }
            }
        }
        Ok(violations)
    }

    fn check_utility_modules(&self, config: &ValidationConfig) -> Result<Vec<VisibilityViolation>> {
        let mut violations = Vec::new();
        let utility_patterns = ["common.rs", "helpers.rs", "internal.rs", "compat.rs"];

        let pub_item_re =
            Regex::new(r"^pub\s+(fn|struct|enum|type)\s+(\w+)").expect("Invalid regex");
        let pub_crate_re = Regex::new(r"^pub\(crate\)").expect("Invalid regex");

        for crate_name in [
            "mcb-infrastructure",
            "mcb-providers",
            "mcb-server",
            "mcb-application",
        ] {
            let crate_src = config
                .workspace_root
                .join("crates")
                .join(crate_name)
                .join("src");
            if !crate_src.exists() {
                continue;
            }

            for entry in WalkDir::new(&crate_src).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !utility_patterns.contains(&file_name) {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let mut pub_count = 0;

                for line in content.lines() {
                    let trimmed = line.trim();
                    if pub_crate_re.is_match(trimmed) {
                        continue;
                    }
                    if pub_item_re.is_match(trimmed) {
                        pub_count += 1;
                    }
                }

                if pub_count > 3 {
                    violations.push(VisibilityViolation::UtilityModuleTooPublic {
                        module_name: file_name.trim_end_matches(".rs").to_string(),
                        file: path.to_path_buf(),
                        line: 1,
                    });
                }
            }
        }
        Ok(violations)
    }
}

impl crate::validator_trait::Validator for VisibilityValidator {
    fn name(&self) -> &'static str {
        "visibility"
    }

    fn description(&self) -> &'static str {
        "Validates visibility modifiers for proper encapsulation"
    }

    fn validate(
        &self,
        config: &crate::ValidationConfig,
    ) -> anyhow::Result<Vec<Box<dyn crate::violation_trait::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::violation_trait::Violation>)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pub_item_pattern() {
        let re = Regex::new(r"^pub\s+(fn|struct|enum|type|const|static)\s+(\w+)").unwrap();
        assert!(re.is_match("pub fn helper() {}"));
        assert!(re.is_match("pub struct Config {}"));
        assert!(!re.is_match("pub(crate) fn internal() {}"));
    }
}
