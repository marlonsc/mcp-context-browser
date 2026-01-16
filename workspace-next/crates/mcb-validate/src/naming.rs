//! Naming Convention Validation
//!
//! Validates naming conventions:
//! - Structs/Enums/Traits: CamelCase
//! - Functions/Methods: snake_case
//! - Constants: SCREAMING_SNAKE_CASE
//! - Modules/Files: snake_case

use crate::{Result, Severity};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Naming violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamingViolation {
    /// Bad struct/enum/trait name (should be CamelCase)
    BadTypeName {
        file: PathBuf,
        line: usize,
        name: String,
        expected_case: String,
        severity: Severity,
    },
    /// Bad function/method name (should be snake_case)
    BadFunctionName {
        file: PathBuf,
        line: usize,
        name: String,
        expected_case: String,
        severity: Severity,
    },
    /// Bad constant name (should be SCREAMING_SNAKE_CASE)
    BadConstantName {
        file: PathBuf,
        line: usize,
        name: String,
        expected_case: String,
        severity: Severity,
    },
    /// Bad module/file name (should be snake_case)
    BadModuleName {
        path: PathBuf,
        expected_case: String,
        severity: Severity,
    },
}

impl NamingViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::BadTypeName { severity, .. } => *severity,
            Self::BadFunctionName { severity, .. } => *severity,
            Self::BadConstantName { severity, .. } => *severity,
            Self::BadModuleName { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for NamingViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadTypeName {
                file,
                line,
                name,
                expected_case,
                ..
            } => {
                write!(
                    f,
                    "Bad type name: {}:{} - {} (expected {})",
                    file.display(),
                    line,
                    name,
                    expected_case
                )
            }
            Self::BadFunctionName {
                file,
                line,
                name,
                expected_case,
                ..
            } => {
                write!(
                    f,
                    "Bad function name: {}:{} - {} (expected {})",
                    file.display(),
                    line,
                    name,
                    expected_case
                )
            }
            Self::BadConstantName {
                file,
                line,
                name,
                expected_case,
                ..
            } => {
                write!(
                    f,
                    "Bad constant name: {}:{} - {} (expected {})",
                    file.display(),
                    line,
                    name,
                    expected_case
                )
            }
            Self::BadModuleName {
                path,
                expected_case,
                ..
            } => {
                write!(
                    f,
                    "Bad module name: {} (expected {})",
                    path.display(),
                    expected_case
                )
            }
        }
    }
}

/// Naming validator
pub struct NamingValidator {
    workspace_root: PathBuf,
}

impl NamingValidator {
    /// Create a new naming validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Run all naming validations
    pub fn validate_all(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_type_names()?);
        violations.extend(self.validate_function_names()?);
        violations.extend(self.validate_constant_names()?);
        violations.extend(self.validate_module_names()?);
        Ok(violations)
    }

    /// Validate struct/enum/trait names are CamelCase
    pub fn validate_type_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let struct_pattern = Regex::new(r"(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");
        let enum_pattern = Regex::new(r"(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");
        let trait_pattern = Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip doc comments and regular comments
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check structs
                    if let Some(cap) = struct_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.is_camel_case(name) {
                            violations.push(NamingViolation::BadTypeName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "CamelCase".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check enums
                    if let Some(cap) = enum_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.is_camel_case(name) {
                            violations.push(NamingViolation::BadTypeName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "CamelCase".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check traits
                    if let Some(cap) = trait_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.is_camel_case(name) {
                            violations.push(NamingViolation::BadTypeName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "CamelCase".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate function/method names are snake_case
    pub fn validate_function_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]").expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = fn_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip test functions
                        if name.starts_with("test_") {
                            continue;
                        }

                        if !self.is_snake_case(name) {
                            violations.push(NamingViolation::BadFunctionName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "snake_case".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate constants are SCREAMING_SNAKE_CASE
    pub fn validate_constant_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let const_pattern = Regex::new(r"(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex");
        let static_pattern = Regex::new(r"(?:pub\s+)?static\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    // Check const
                    if let Some(cap) = const_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.is_screaming_snake_case(name) {
                            violations.push(NamingViolation::BadConstantName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "SCREAMING_SNAKE_CASE".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check static
                    if let Some(cap) = static_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.is_screaming_snake_case(name) {
                            violations.push(NamingViolation::BadConstantName {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                name: name.to_string(),
                                expected_case: "SCREAMING_SNAKE_CASE".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate module/file names are snake_case
    pub fn validate_module_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let file_name = entry
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Skip lib.rs, mod.rs, main.rs
                if file_name == "lib" || file_name == "mod" || file_name == "main" {
                    continue;
                }

                if !self.is_snake_case(file_name) {
                    violations.push(NamingViolation::BadModuleName {
                        path: entry.path().to_path_buf(),
                        expected_case: "snake_case".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Check if name is CamelCase
    fn is_camel_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must start with uppercase
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_uppercase() {
            return false;
        }

        // No underscores allowed (except at the start for private items, which we skip)
        if name.contains('_') {
            return false;
        }

        // Must have at least one lowercase letter
        name.chars().any(|c| c.is_ascii_lowercase())
    }

    /// Check if name is snake_case
    fn is_snake_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must be all lowercase or underscores or digits
        for c in name.chars() {
            if !c.is_ascii_lowercase() && c != '_' && !c.is_ascii_digit() {
                return false;
            }
        }

        // Can't start with digit
        !name.chars().next().unwrap().is_ascii_digit()
    }

    /// Check if name is SCREAMING_SNAKE_CASE
    fn is_screaming_snake_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must be all uppercase or underscores or digits
        for c in name.chars() {
            if !c.is_ascii_uppercase() && c != '_' && !c.is_ascii_digit() {
                return false;
            }
        }

        // Can't start with digit
        !name.chars().next().unwrap().is_ascii_digit()
    }

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        let crates_dir = self.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut dirs = Vec::new();
        for entry in std::fs::read_dir(&crates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str != "mcb-validate" {
                    dirs.push(entry.path());
                }
            }
        }
        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_case_detection() {
        let validator = NamingValidator::new("/tmp");

        assert!(validator.is_camel_case("MyStruct"));
        assert!(validator.is_camel_case("SomeEnumVariant"));
        assert!(!validator.is_camel_case("my_struct"));
        assert!(!validator.is_camel_case("SCREAMING"));
        assert!(!validator.is_camel_case("Snake_Case"));
    }

    #[test]
    fn test_snake_case_detection() {
        let validator = NamingValidator::new("/tmp");

        assert!(validator.is_snake_case("my_function"));
        assert!(validator.is_snake_case("another_one"));
        assert!(validator.is_snake_case("simple"));
        assert!(!validator.is_snake_case("MyFunction"));
        assert!(!validator.is_snake_case("SCREAMING"));
    }

    #[test]
    fn test_screaming_snake_case_detection() {
        let validator = NamingValidator::new("/tmp");

        assert!(validator.is_screaming_snake_case("MY_CONSTANT"));
        assert!(validator.is_screaming_snake_case("MAX_SIZE"));
        assert!(validator.is_screaming_snake_case("VALUE"));
        assert!(!validator.is_screaming_snake_case("myConstant"));
        assert!(!validator.is_screaming_snake_case("my_constant"));
    }
}
