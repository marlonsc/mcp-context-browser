//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Documentation violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationViolation {
    /// Missing module-level documentation
    MissingModuleDoc { file: PathBuf, severity: Severity },
    /// Missing documentation on public item
    MissingPubItemDoc {
        file: PathBuf,
        line: usize,
        item_name: String,
        item_kind: String,
        severity: Severity,
    },
    /// Missing example code in documentation
    MissingExampleCode {
        file: PathBuf,
        line: usize,
        item_name: String,
        severity: Severity,
    },
}

impl DocumentationViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::MissingModuleDoc { severity, .. } => *severity,
            Self::MissingPubItemDoc { severity, .. } => *severity,
            Self::MissingExampleCode { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for DocumentationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingModuleDoc { file, .. } => {
                write!(f, "Missing module doc: {}", file.display())
            }
            Self::MissingPubItemDoc {
                file,
                line,
                item_name,
                item_kind,
                ..
            } => {
                write!(
                    f,
                    "Missing {} doc: {}:{} - {}",
                    item_kind,
                    file.display(),
                    line,
                    item_name
                )
            }
            Self::MissingExampleCode {
                file,
                line,
                item_name,
                ..
            } => {
                write!(
                    f,
                    "Missing example: {}:{} - {}",
                    file.display(),
                    line,
                    item_name
                )
            }
        }
    }
}

impl Violation for DocumentationViolation {
    fn id(&self) -> &str {
        match self {
            Self::MissingModuleDoc { .. } => "DOC001",
            Self::MissingPubItemDoc { .. } => "DOC002",
            Self::MissingExampleCode { .. } => "DOC003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Documentation
    }

    fn severity(&self) -> Severity {
        match self {
            Self::MissingModuleDoc { severity, .. } => *severity,
            Self::MissingPubItemDoc { severity, .. } => *severity,
            Self::MissingExampleCode { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::MissingModuleDoc { file, .. } => Some(file),
            Self::MissingPubItemDoc { file, .. } => Some(file),
            Self::MissingExampleCode { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::MissingModuleDoc { .. } => None,
            Self::MissingPubItemDoc { line, .. } => Some(*line),
            Self::MissingExampleCode { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::MissingModuleDoc { .. } => {
                Some("Add //! module-level documentation at the top of the file".to_string())
            }
            Self::MissingPubItemDoc {
                item_kind,
                item_name,
                ..
            } => Some(format!(
                "Add /// documentation for {} {}",
                item_kind, item_name
            )),
            Self::MissingExampleCode { item_name, .. } => Some(format!(
                "Add # Example section to {} documentation",
                item_name
            )),
        }
    }
}

/// Documentation validator
pub struct DocumentationValidator {
    config: ValidationConfig,
}

impl DocumentationValidator {
    /// Create a new documentation validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all documentation validations
    pub fn validate_all(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_module_docs()?);
        violations.extend(self.validate_pub_item_docs()?);
        Ok(violations)
    }

    /// Verify module-level documentation exists
    pub fn validate_module_docs(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        let module_doc_pattern = Regex::new(r"^//!").expect("Invalid regex");

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
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                // Only check lib.rs, mod.rs, and main module files
                if file_name != "lib.rs" && file_name != "mod.rs" {
                    continue;
                }

                // Check if first non-empty line is module doc
                let has_module_doc = content
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .take(1)
                    .any(|line| module_doc_pattern.is_match(line));

                if !has_module_doc {
                    violations.push(DocumentationViolation::MissingModuleDoc {
                        file: entry.path().to_path_buf(),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Verify all pub items have rustdoc
    pub fn validate_pub_item_docs(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();

        // Patterns for public items
        let pub_struct_pattern =
            Regex::new(r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let pub_enum_pattern =
            Regex::new(r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let pub_trait_pattern =
            Regex::new(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let pub_fn_pattern =
            Regex::new(r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)").expect("Invalid regex");
        let _doc_comment_pattern = Regex::new(r"^\s*///").expect("Invalid regex");
        let example_pattern = Regex::new(r"#\s*Example").expect("Invalid regex");

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
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    // Check for public structs
                    if let Some(cap) = pub_struct_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.has_doc_comment(&lines, line_num) {
                            violations.push(DocumentationViolation::MissingPubItemDoc {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                item_name: name.to_string(),
                                item_kind: "struct".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check for public enums
                    if let Some(cap) = pub_enum_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !self.has_doc_comment(&lines, line_num) {
                            violations.push(DocumentationViolation::MissingPubItemDoc {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                item_name: name.to_string(),
                                item_kind: "enum".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    // Check for public traits
                    if let Some(cap) = pub_trait_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let path_str = entry.path().to_string_lossy();

                        if !self.has_doc_comment(&lines, line_num) {
                            violations.push(DocumentationViolation::MissingPubItemDoc {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                item_name: name.to_string(),
                                item_kind: "trait".to_string(),
                                severity: Severity::Error,
                            });
                        } else {
                            // Check for example code in trait documentation
                            // Skip DI module traits and port traits - they are interface definitions
                            // that don't need examples (they define contracts for DI injection)
                            let is_di_or_port_trait = path_str.contains("/di/modules/")
                                || path_str.contains("/ports/");

                            if !is_di_or_port_trait {
                                let doc_section = self.get_doc_comment_section(&lines, line_num);
                                if !example_pattern.is_match(&doc_section) {
                                    violations.push(DocumentationViolation::MissingExampleCode {
                                        file: entry.path().to_path_buf(),
                                        line: line_num + 1,
                                        item_name: name.to_string(),
                                        severity: Severity::Info,
                                    });
                                }
                            }
                        }
                    }

                    // Check for public functions (only top-level, not in impl blocks)
                    if let Some(cap) = pub_fn_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip methods in impl blocks (approximation: indentation > 0)
                        if line.starts_with("    ") || line.starts_with("\t") {
                            continue;
                        }

                        if !self.has_doc_comment(&lines, line_num) {
                            violations.push(DocumentationViolation::MissingPubItemDoc {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                item_name: name.to_string(),
                                item_kind: "function".to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    fn has_doc_comment(&self, lines: &[&str], item_line: usize) -> bool {
        let doc_pattern = Regex::new(r"^\s*///").expect("Invalid regex");
        let attr_pattern = Regex::new(r"^\s*#\[").expect("Invalid regex");

        if item_line == 0 {
            return false;
        }

        // Look backwards for doc comments, skipping attributes
        let mut i = item_line - 1;
        loop {
            let line = lines[i].trim();

            // Skip empty lines between attributes and doc comments
            if line.is_empty() {
                if i == 0 {
                    return false;
                }
                i -= 1;
                continue;
            }

            // Skip attributes
            if attr_pattern.is_match(lines[i]) {
                if i == 0 {
                    return false;
                }
                i -= 1;
                continue;
            }

            // Check for doc comment
            return doc_pattern.is_match(lines[i]);
        }
    }

    fn get_doc_comment_section(&self, lines: &[&str], item_line: usize) -> String {
        let doc_pattern = Regex::new(r"^\s*///(.*)").expect("Invalid regex");
        let attr_pattern = Regex::new(r"^\s*#\[").expect("Invalid regex");

        if item_line == 0 {
            return String::new();
        }

        let mut doc_lines = Vec::new();
        let mut i = item_line - 1;

        loop {
            let line = lines[i];

            // Skip attributes
            if attr_pattern.is_match(line) {
                if i == 0 {
                    break;
                }
                i -= 1;
                continue;
            }

            // Collect doc comment
            if let Some(cap) = doc_pattern.captures(line) {
                let content = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                doc_lines.push(content);
            } else if !line.trim().is_empty() {
                break;
            }

            if i == 0 {
                break;
            }
            i -= 1;
        }

        doc_lines.reverse();
        doc_lines.join("\n")
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

impl crate::validator_trait::Validator for DocumentationValidator {
    fn name(&self) -> &'static str {
        "documentation"
    }

    fn description(&self) -> &'static str {
        "Validates documentation standards"
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
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
        let crate_dir = temp.path().join("crates").join(name).join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(crate_dir.join("lib.rs"), content).unwrap();

        let cargo_dir = temp.path().join("crates").join(name);
        fs::write(
            cargo_dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.1"
"#,
                name
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_missing_module_doc() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn something() {}
"#,
        );

        let validator = DocumentationValidator::new(temp.path());
        let violations = validator.validate_module_docs().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            DocumentationViolation::MissingModuleDoc { .. } => {}
            _ => panic!("Expected MissingModuleDoc"),
        }
    }

    #[test]
    fn test_module_doc_present() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"//! This is the module documentation.

pub fn something() {}
"#,
        );

        let validator = DocumentationValidator::new(temp.path());
        let violations = validator.validate_module_docs().unwrap();

        assert!(
            violations.is_empty(),
            "Should have no violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_missing_struct_doc() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"//! Module doc.

pub struct UndocumentedStruct {
    field: i32,
}
"#,
        );

        let validator = DocumentationValidator::new(temp.path());
        let violations = validator.validate_pub_item_docs().unwrap();

        let struct_violations: Vec<_> = violations
            .iter()
            .filter(|v| matches!(v, DocumentationViolation::MissingPubItemDoc { item_kind, .. } if item_kind == "struct"))
            .collect();

        assert_eq!(struct_violations.len(), 1);
    }

    #[test]
    fn test_documented_struct() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"//! Module doc.

/// This struct is documented.
pub struct DocumentedStruct {
    field: i32,
}
"#,
        );

        let validator = DocumentationValidator::new(temp.path());
        let violations = validator.validate_pub_item_docs().unwrap();

        let struct_violations: Vec<_> = violations
            .iter()
            .filter(|v| matches!(v, DocumentationViolation::MissingPubItemDoc { item_kind, .. } if item_kind == "struct"))
            .collect();

        assert!(
            struct_violations.is_empty(),
            "Should have no struct violations: {:?}",
            struct_violations
        );
    }
}
