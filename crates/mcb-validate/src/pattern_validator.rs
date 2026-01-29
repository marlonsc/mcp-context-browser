//! Pattern Compliance Validation
//!
//! Validates code patterns:
//! - DI uses `Arc<dyn Trait>` not `Arc<ConcreteType>`
//! - Async traits have `#[async_trait]` and Send + Sync bounds
//! - Error types use `crate::error::Result<T>`
//! - Provider pattern compliance

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Pattern violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternViolation {
    /// Concrete type used in DI instead of trait object
    ConcreteTypeInDi {
        file: PathBuf,
        line: usize,
        concrete_type: String,
        suggestion: String,
        severity: Severity,
    },
    /// Async trait missing Send + Sync bounds
    MissingSendSync {
        file: PathBuf,
        line: usize,
        trait_name: String,
        missing_bound: String,
        severity: Severity,
    },
    /// Async trait missing #[async_trait] attribute
    MissingAsyncTrait {
        file: PathBuf,
        line: usize,
        trait_name: String,
        severity: Severity,
    },
    /// Using std::result::Result instead of crate::error::Result
    RawResultType {
        file: PathBuf,
        line: usize,
        context: String,
        suggestion: String,
        severity: Severity,
    },
    /// Missing Interface trait bound for Shaku DI
    MissingInterfaceBound {
        file: PathBuf,
        line: usize,
        trait_name: String,
        severity: Severity,
    },
}

impl PatternViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::ConcreteTypeInDi { severity, .. } => *severity,
            Self::MissingSendSync { severity, .. } => *severity,
            Self::MissingAsyncTrait { severity, .. } => *severity,
            Self::RawResultType { severity, .. } => *severity,
            Self::MissingInterfaceBound { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for PatternViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConcreteTypeInDi {
                file,
                line,
                concrete_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Concrete type in DI: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    concrete_type,
                    suggestion
                )
            }
            Self::MissingSendSync {
                file,
                line,
                trait_name,
                missing_bound,
                ..
            } => {
                write!(
                    f,
                    "Missing bound: {}:{} - trait {} needs {}",
                    file.display(),
                    line,
                    trait_name,
                    missing_bound
                )
            }
            Self::MissingAsyncTrait {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing #[async_trait]: {}:{} - trait {}",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::RawResultType {
                file,
                line,
                context,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Raw Result type: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    context,
                    suggestion
                )
            }
            Self::MissingInterfaceBound {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing Interface bound: {}:{} - trait {} needs : Interface",
                    file.display(),
                    line,
                    trait_name
                )
            }
        }
    }
}

impl Violation for PatternViolation {
    fn id(&self) -> &str {
        match self {
            Self::ConcreteTypeInDi { .. } => "PAT001",
            Self::MissingSendSync { .. } => "PAT002",
            Self::MissingAsyncTrait { .. } => "PAT003",
            Self::RawResultType { .. } => "PAT004",
            Self::MissingInterfaceBound { .. } => "PAT005",
        }
    }

    fn category(&self) -> ViolationCategory {
        match self {
            Self::ConcreteTypeInDi { .. } => ViolationCategory::DependencyInjection,
            Self::MissingSendSync { .. } => ViolationCategory::Async,
            Self::MissingAsyncTrait { .. } => ViolationCategory::Async,
            Self::RawResultType { .. } => ViolationCategory::Quality,
            Self::MissingInterfaceBound { .. } => ViolationCategory::DependencyInjection,
        }
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ConcreteTypeInDi { severity, .. } => *severity,
            Self::MissingSendSync { severity, .. } => *severity,
            Self::MissingAsyncTrait { severity, .. } => *severity,
            Self::RawResultType { severity, .. } => *severity,
            Self::MissingInterfaceBound { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::ConcreteTypeInDi { file, .. } => Some(file),
            Self::MissingSendSync { file, .. } => Some(file),
            Self::MissingAsyncTrait { file, .. } => Some(file),
            Self::RawResultType { file, .. } => Some(file),
            Self::MissingInterfaceBound { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ConcreteTypeInDi { line, .. } => Some(*line),
            Self::MissingSendSync { line, .. } => Some(*line),
            Self::MissingAsyncTrait { line, .. } => Some(*line),
            Self::RawResultType { line, .. } => Some(*line),
            Self::MissingInterfaceBound { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::ConcreteTypeInDi { suggestion, .. } => Some(format!("Use {}", suggestion)),
            Self::MissingSendSync { missing_bound, .. } => {
                Some(format!("Add {} bounds to trait", missing_bound))
            }
            Self::MissingAsyncTrait { .. } => Some("Add #[async_trait] attribute".to_string()),
            Self::RawResultType { suggestion, .. } => Some(format!("Use {}", suggestion)),
            Self::MissingInterfaceBound { .. } => {
                Some("Add : Interface bound for Shaku DI".to_string())
            }
        }
    }
}

/// Pattern validator
pub struct PatternValidator {
    config: ValidationConfig,
}

impl PatternValidator {
    /// Create a new pattern validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all pattern validations
    pub fn validate_all(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_trait_based_di()?);
        violations.extend(self.validate_async_traits()?);
        violations.extend(self.validate_result_types()?);
        Ok(violations)
    }

    /// Verify `Arc<dyn Trait>` pattern instead of `Arc<ConcreteType>`
    pub fn validate_trait_based_di(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        // Pattern to find Arc<SomeConcreteType> where SomeConcreteType doesn't start with "dyn"
        let arc_pattern = Regex::new(r"Arc<([A-Z][a-zA-Z0-9_]*)>").unwrap();

        // Known concrete types that are OK to use directly
        let allowed_concrete = [
            // Standard library sync primitives
            "String",
            "Mutex",
            "RwLock",
            "AtomicBool",
            "AtomicUsize",
            "AtomicU32",
            "AtomicU64",
            "AtomicI32",
            "AtomicI64",
            "Notify",
            "Barrier",
            "Semaphore",
            "Once",
            // Infrastructure services that are intentionally concrete
            "CryptoService", // Encryption service - no need for trait abstraction
            // Handler types that are final implementations
            "ToolHandler",
            "ResourceHandler",
            "PromptHandler",
            "AdminHandler",
            "ToolRouter",
        ];

        // Provider trait names that should use Arc<dyn ...>
        // Note: "Handler" is excluded - handlers are typically final implementations
        let provider_traits = ["Provider", "Service", "Repository", "Interface"];

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate (contains test examples of bad patterns)
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check for ignore hints
                    let has_ignore_hint =
                        line.contains("mcb-validate-ignore: admin_service_concrete_type");

                    for cap in arc_pattern.captures_iter(line) {
                        let type_name = cap.get(1).map_or("", |m| m.as_str());

                        // Skip allowed concrete types
                        if allowed_concrete.contains(&type_name) {
                            continue;
                        }

                        // Skip if already using dyn (handled by different pattern)
                        if line.contains(&format!("Arc<dyn {}", type_name)) {
                            continue;
                        }

                        // Skip decorator pattern: Arc<Type<T>> (generic wrapper types)
                        // e.g., Arc<EncryptedProvider<P>> where P is a generic
                        if line.contains(&format!("Arc<{}<", type_name)) {
                            continue;
                        }

                        // Check if type name ends with a provider trait suffix
                        let is_likely_provider = provider_traits
                            .iter()
                            .any(|suffix| type_name.ends_with(suffix));

                        // Also check for common service implementation patterns
                        let is_impl_suffix = type_name.ends_with("Impl")
                            || type_name.ends_with("Implementation")
                            || type_name.ends_with("Adapter");

                        if is_likely_provider || is_impl_suffix {
                            // Skip if ignore hint is present
                            if has_ignore_hint {
                                continue;
                            }

                            let trait_name = if is_impl_suffix {
                                type_name
                                    .trim_end_matches("Impl")
                                    .trim_end_matches("Implementation")
                                    .trim_end_matches("Adapter")
                            } else {
                                type_name
                            };

                            violations.push(PatternViolation::ConcreteTypeInDi {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                concrete_type: format!("Arc<{}>", type_name),
                                suggestion: format!("Arc<dyn {}>", trait_name),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check async traits have #[async_trait] and Send + Sync bounds
    pub fn validate_async_traits(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        let trait_pattern = Regex::new(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)").unwrap();
        let async_fn_pattern = Regex::new(r"async\s+fn\s+").unwrap();
        let send_sync_pattern = Regex::new(r":\s*.*Send\s*\+\s*Sync").unwrap();
        // Match both #[async_trait] and #[async_trait::async_trait]
        let async_trait_attr = Regex::new(r"#\[(async_trait::)?async_trait\]").unwrap();
        // Rust 1.75+ native async trait support
        let allow_async_fn_trait = Regex::new(r"#\[allow\(async_fn_in_trait\)\]").unwrap();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    // Find trait definitions
                    if let Some(cap) = trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map_or("", |m| m.as_str());

                        // Look ahead to see if trait has async methods
                        let mut has_async_methods = false;
                        let mut brace_depth = 0;
                        let mut in_trait = false;

                        for subsequent_line in &lines[line_num..] {
                            if subsequent_line.contains('{') {
                                in_trait = true;
                            }
                            if in_trait {
                                brace_depth +=
                                    subsequent_line.chars().filter(|c| *c == '{').count();
                                brace_depth -=
                                    subsequent_line.chars().filter(|c| *c == '}').count();

                                if async_fn_pattern.is_match(subsequent_line) {
                                    has_async_methods = true;
                                    break;
                                }

                                if brace_depth == 0 {
                                    break;
                                }
                            }
                        }

                        if has_async_methods {
                            // Check for #[async_trait] attribute or #[allow(async_fn_in_trait)]
                            let has_async_trait_attr = if line_num > 0 {
                                lines[..line_num].iter().rev().take(5).any(|l| {
                                    async_trait_attr.is_match(l) || allow_async_fn_trait.is_match(l)
                                })
                            } else {
                                false
                            };

                            // Check if using native async trait support
                            let uses_native_async = if line_num > 0 {
                                lines[..line_num]
                                    .iter()
                                    .rev()
                                    .take(5)
                                    .any(|l| allow_async_fn_trait.is_match(l))
                            } else {
                                false
                            };

                            if !has_async_trait_attr {
                                violations.push(PatternViolation::MissingAsyncTrait {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    trait_name: trait_name.to_string(),
                                    severity: Severity::Error,
                                });
                            }

                            // Check for Send + Sync bounds (skip for native async traits)
                            if !send_sync_pattern.is_match(line) && !uses_native_async {
                                violations.push(PatternViolation::MissingSendSync {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    trait_name: trait_name.to_string(),
                                    missing_bound: "Send + Sync".to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Verify consistent error type usage
    pub fn validate_result_types(&self) -> Result<Vec<PatternViolation>> {
        let mut violations = Vec::new();

        // Pattern to find std::result::Result usage
        let std_result_pattern = Regex::new(r"std::result::Result<").expect("Invalid regex");

        // Pattern to find Result<T, E> with explicit error type (not crate::Result)
        let explicit_result_pattern =
            Regex::new(r"Result<[^,]+,\s*([A-Za-z][A-Za-z0-9_:]+)>").expect("Invalid regex");

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            // Skip mcb-providers - factory functions use std::result::Result<..., String>
            // by design for linkme registry compatibility (ADR-029)
            if src_dir.to_string_lossy().contains("mcb-providers") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                // Skip error-related files (they define/extend error types)
                let file_name = entry.path().file_name().and_then(|n| n.to_str());
                if file_name.is_some_and(|n| {
                    n == "error.rs" || n == "error_ext.rs" || n.starts_with("error")
                }) {
                    continue;
                }

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments and use statements
                    if trimmed.starts_with("//") || trimmed.starts_with("use ") {
                        continue;
                    }

                    // Check for std::result::Result
                    if std_result_pattern.is_match(line) {
                        violations.push(PatternViolation::RawResultType {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            context: trimmed.to_string(),
                            suggestion: "crate::Result<T>".to_string(),
                            severity: Severity::Warning,
                        });
                    }

                    // Check for Result<T, SomeError> with explicit error type
                    if let Some(cap) = explicit_result_pattern.captures(line) {
                        let error_type = cap.get(1).map_or("", |m| m.as_str());

                        // Allow certain standard error types
                        let allowed_errors = [
                            "Error",
                            "crate::Error",
                            "crate::error::Error",
                            "ValidationError",
                            "std::io::Error",
                            "anyhow::Error",
                        ];

                        if !allowed_errors.contains(&error_type)
                            && !error_type.starts_with("crate::")
                            && !error_type.starts_with("self::")
                        {
                            // This is informational - sometimes explicit error types are needed
                            // We won't flag this as a violation for now
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for PatternValidator {
    fn name(&self) -> &'static str {
        "patterns"
    }

    fn description(&self) -> &'static str {
        "Validates code patterns (DI, async traits, error handling)"
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
    fn test_concrete_type_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
use std::sync::Arc;

pub struct MyServiceImpl;

pub struct Container {
    service: Arc<MyServiceImpl>,
}
"#,
        );

        let validator = PatternValidator::new(temp.path());
        let violations = validator.validate_trait_based_di().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            PatternViolation::ConcreteTypeInDi {
                concrete_type,
                suggestion,
                ..
            } => {
                assert_eq!(concrete_type, "Arc<MyServiceImpl>");
                assert_eq!(suggestion, "Arc<dyn MyService>");
            }
            _ => panic!("Expected ConcreteTypeInDi"),
        }
    }

    #[test]
    fn test_dyn_trait_allowed() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
use std::sync::Arc;

pub trait MyService: Send + Sync {}

pub struct Container {
    service: Arc<dyn MyService>,
}
"#,
        );

        let validator = PatternValidator::new(temp.path());
        let violations = validator.validate_trait_based_di().unwrap();

        assert!(
            violations.is_empty(),
            "Arc<dyn Trait> should be allowed: {:?}",
            violations
        );
    }

    #[test]
    fn test_std_result_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn bad_function() -> std::result::Result<i32, String> {
    Ok(42)
}
"#,
        );

        let validator = PatternValidator::new(temp.path());
        let violations = validator.validate_result_types().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            PatternViolation::RawResultType { .. } => {}
            _ => panic!("Expected RawResultType"),
        }
    }
}
