//! Naming Convention Validation
//!
//! Validates naming conventions:
//! - Structs/Enums/Traits: CamelCase
//! - Functions/Methods: snake_case
//! - Constants: SCREAMING_SNAKE_CASE
//! - Modules/Files: snake_case

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
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

    /// File suffix doesn't match component type
    BadFileSuffix {
        path: PathBuf,
        component_type: String,
        current_suffix: String,
        expected_suffix: String,
        severity: Severity,
    },

    /// File name doesn't follow CA naming convention
    BadCaNaming {
        path: PathBuf,
        detected_type: String,
        issue: String,
        suggestion: String,
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
            Self::BadFileSuffix { severity, .. } => *severity,
            Self::BadCaNaming { severity, .. } => *severity,
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
            Self::BadFileSuffix {
                path,
                component_type,
                current_suffix,
                expected_suffix,
                ..
            } => {
                write!(
                    f,
                    "Bad file suffix: {} ({}) has suffix '{}' but expected '{}'",
                    path.display(),
                    component_type,
                    current_suffix,
                    expected_suffix
                )
            }
            Self::BadCaNaming {
                path,
                detected_type,
                issue,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "CA naming: {} ({}): {} - {}",
                    path.display(),
                    detected_type,
                    issue,
                    suggestion
                )
            }
        }
    }
}

impl Violation for NamingViolation {
    fn id(&self) -> &str {
        match self {
            Self::BadTypeName { .. } => "NAME001",
            Self::BadFunctionName { .. } => "NAME002",
            Self::BadConstantName { .. } => "NAME003",
            Self::BadModuleName { .. } => "NAME004",
            Self::BadFileSuffix { .. } => "NAME005",
            Self::BadCaNaming { .. } => "NAME006",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Naming
    }

    fn severity(&self) -> Severity {
        match self {
            Self::BadTypeName { severity, .. } => *severity,
            Self::BadFunctionName { severity, .. } => *severity,
            Self::BadConstantName { severity, .. } => *severity,
            Self::BadModuleName { severity, .. } => *severity,
            Self::BadFileSuffix { severity, .. } => *severity,
            Self::BadCaNaming { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::BadTypeName { file, .. } => Some(file),
            Self::BadFunctionName { file, .. } => Some(file),
            Self::BadConstantName { file, .. } => Some(file),
            Self::BadModuleName { path, .. } => Some(path),
            Self::BadFileSuffix { path, .. } => Some(path),
            Self::BadCaNaming { path, .. } => Some(path),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::BadTypeName { line, .. } => Some(*line),
            Self::BadFunctionName { line, .. } => Some(*line),
            Self::BadConstantName { line, .. } => Some(*line),
            Self::BadModuleName { .. } => None,
            Self::BadFileSuffix { .. } => None,
            Self::BadCaNaming { .. } => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::BadTypeName {
                name,
                expected_case,
                ..
            } => Some(format!("Rename '{}' to {} format", name, expected_case)),
            Self::BadFunctionName {
                name,
                expected_case,
                ..
            } => Some(format!("Rename '{}' to {} format", name, expected_case)),
            Self::BadConstantName {
                name,
                expected_case,
                ..
            } => Some(format!("Rename '{}' to {} format", name, expected_case)),
            Self::BadModuleName {
                path,
                expected_case,
                ..
            } => {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                Some(format!(
                    "Rename '{}' to {} format",
                    file_name, expected_case
                ))
            }
            Self::BadFileSuffix {
                expected_suffix, ..
            } => Some(format!("Add '{}' suffix to file name", expected_suffix)),
            Self::BadCaNaming { suggestion, .. } => Some(suggestion.clone()),
        }
    }
}

/// Naming validator
pub struct NamingValidator {
    config: ValidationConfig,
}

impl NamingValidator {
    /// Create a new naming validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all naming validations
    pub fn validate_all(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_type_names()?);
        violations.extend(self.validate_function_names()?);
        violations.extend(self.validate_constant_names()?);
        violations.extend(self.validate_module_names()?);
        violations.extend(self.validate_file_suffixes()?);
        violations.extend(self.validate_ca_naming()?);
        Ok(violations)
    }

    /// Validate struct/enum/trait names are CamelCase
    pub fn validate_type_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");
        let enum_pattern =
            Regex::new(r"(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)").expect("Invalid regex");

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

        let fn_pattern =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
                .expect("Invalid regex");

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

        let const_pattern =
            Regex::new(r"(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex");
        let static_pattern =
            Regex::new(r"(?:pub\s+)?static\s+([A-Za-z_][A-Za-z0-9_]*)\s*:").expect("Invalid regex");

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

    /// Validate file suffixes match component types per CA naming conventions
    pub fn validate_file_suffixes(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip standard files
                if file_name == "lib" || file_name == "mod" || file_name == "main" {
                    continue;
                }

                let path_str = path.to_string_lossy();

                // Check repository files should have _repository suffix
                if (path_str.contains("/repositories/")
                    || path_str.contains("/adapters/repository/"))
                    && !file_name.ends_with("_repository")
                    && file_name != "mod"
                {
                    violations.push(NamingViolation::BadFileSuffix {
                        path: path.to_path_buf(),
                        component_type: "Repository".to_string(),
                        current_suffix: self.get_suffix(file_name).to_string(),
                        expected_suffix: "_repository".to_string(),
                        severity: Severity::Warning,
                    });
                }

                // Check handler files in server crate
                if crate_name == "mcb-server" && path_str.contains("/handlers/") {
                    // Handlers should have descriptive names (snake_case tool names)
                    // but NOT have _handler suffix (that's redundant with directory)
                    if file_name.ends_with("_handler") {
                        violations.push(NamingViolation::BadFileSuffix {
                            path: path.to_path_buf(),
                            component_type: "Handler".to_string(),
                            current_suffix: "_handler".to_string(),
                            expected_suffix: "<tool_name> (no _handler suffix in handlers/ dir)"
                                .to_string(),
                            severity: Severity::Info,
                        });
                    }
                }

                // Check service files should have _service suffix if in services directory
                // Note: mcb-domain/domain_services contains interfaces, not implementations
                // so we skip suffix validation for that directory
                if path_str.contains("/services/")
                    && !path_str.contains("/domain_services/")
                    && crate_name != "mcb-domain"
                    && !file_name.ends_with("_service")
                    && file_name != "mod"
                {
                    violations.push(NamingViolation::BadFileSuffix {
                        path: path.to_path_buf(),
                        component_type: "Service".to_string(),
                        current_suffix: self.get_suffix(file_name).to_string(),
                        expected_suffix: "_service".to_string(),
                        severity: Severity::Info,
                    });
                }

                // Check factory files - allow both 'factory.rs' and '*_factory.rs'
                // A file named exactly "factory.rs" is valid (e.g., provider_factory module)
                if file_name.contains("factory")
                    && !file_name.ends_with("_factory")
                    && file_name != "factory"
                {
                    violations.push(NamingViolation::BadFileSuffix {
                        path: path.to_path_buf(),
                        component_type: "Factory".to_string(),
                        current_suffix: self.get_suffix(file_name).to_string(),
                        expected_suffix: "_factory".to_string(),
                        severity: Severity::Info,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Validate Clean Architecture naming conventions
    pub fn validate_ca_naming(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let path_str = path.to_string_lossy();

                // Skip standard files
                if file_name == "lib" || file_name == "mod" || file_name == "main" {
                    continue;
                }

                // Domain crate: port traits should be in ports/
                if crate_name == "mcb-domain" {
                    // Files with "provider" in name should be in ports/providers/
                    if file_name.contains("provider")
                        && !path_str.contains("/ports/providers/")
                        && !path_str.contains("/ports/")
                    {
                        violations.push(NamingViolation::BadCaNaming {
                            path: path.to_path_buf(),
                            detected_type: "Provider Port".to_string(),
                            issue: "Provider file outside ports/ directory".to_string(),
                            suggestion: "Move to ports/providers/".to_string(),
                            severity: Severity::Warning,
                        });
                    }

                    // Files with "repository" in name should be in repositories/
                    if file_name.contains("repository") && !path_str.contains("/repositories/") {
                        violations.push(NamingViolation::BadCaNaming {
                            path: path.to_path_buf(),
                            detected_type: "Repository Port".to_string(),
                            issue: "Repository file outside repositories/ directory".to_string(),
                            suggestion: "Move to repositories/".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Infrastructure crate: adapters should be in adapters/
                if crate_name == "mcb-infrastructure" {
                    // Implementation files should be in adapters/
                    if (file_name.ends_with("_impl") || file_name.contains("adapter"))
                        && !path_str.contains("/adapters/")
                    {
                        violations.push(NamingViolation::BadCaNaming {
                            path: path.to_path_buf(),
                            detected_type: "Adapter".to_string(),
                            issue: "Adapter/implementation file outside adapters/ directory"
                                .to_string(),
                            suggestion: "Move to adapters/".to_string(),
                            severity: Severity::Warning,
                        });
                    }

                    // DI modules should be in di/
                    if file_name.contains("module") && !path_str.contains("/di/") {
                        violations.push(NamingViolation::BadCaNaming {
                            path: path.to_path_buf(),
                            detected_type: "DI Module".to_string(),
                            issue: "Module file outside di/ directory".to_string(),
                            suggestion: "Move to di/modules/".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }

                // Server crate: handlers should be in handlers/ or admin/
                if crate_name == "mcb-server" {
                    // Allow handlers in handlers/, admin/, or tools/ directories
                    let in_allowed_handler_dir = path_str.contains("/handlers/")
                        || path_str.contains("/admin/")
                        || path_str.contains("/tools/");
                    if file_name.contains("handler") && !in_allowed_handler_dir {
                        violations.push(NamingViolation::BadCaNaming {
                            path: path.to_path_buf(),
                            detected_type: "Handler".to_string(),
                            issue: "Handler file outside handlers/ directory".to_string(),
                            suggestion: "Move to handlers/, admin/, or tools/".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Extract suffix from file name (part after last underscore)
    fn get_suffix<'a>(&self, name: &'a str) -> &'a str {
        if let Some(pos) = name.rfind('_') {
            &name[pos..]
        } else {
            ""
        }
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
        self.config.get_source_dirs()
    }

    /// Check if a path is from legacy/additional source directories
    #[allow(dead_code)]
    fn is_legacy_path(&self, path: &std::path::Path) -> bool {
        self.config.is_legacy_path(path)
    }
}

impl crate::validator_trait::Validator for NamingValidator {
    fn name(&self) -> &'static str {
        "naming"
    }

    fn description(&self) -> &'static str {
        "Validates naming conventions (CamelCase, snake_case, SCREAMING_SNAKE_CASE)"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
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
