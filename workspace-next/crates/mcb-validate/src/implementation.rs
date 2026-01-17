//! Implementation Quality Validation
//!
//! Detects false, incomplete, or low-quality implementations:
//! - Empty method bodies (return Ok(()), None, Vec::new())
//! - Hardcoded return values (return true, return 0)
//! - Pass-through wrappers without transformation
//! - Log-only methods (no actual logic)
//! - Default-only trait implementations

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Implementation quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationViolation {
    /// Method body is empty or returns trivial value
    EmptyMethodBody {
        file: PathBuf,
        line: usize,
        method_name: String,
        pattern: String,
        severity: Severity,
    },
    /// Method returns hardcoded value bypassing logic
    HardcodedReturnValue {
        file: PathBuf,
        line: usize,
        method_name: String,
        return_value: String,
        severity: Severity,
    },
    /// Wrapper that just delegates without adding value
    PassThroughWrapper {
        file: PathBuf,
        line: usize,
        struct_name: String,
        method_name: String,
        delegated_to: String,
        severity: Severity,
    },
    /// Method body only contains logging/tracing
    LogOnlyMethod {
        file: PathBuf,
        line: usize,
        method_name: String,
        severity: Severity,
    },
    /// Stub implementation using todo!/unimplemented!
    StubMacro {
        file: PathBuf,
        line: usize,
        method_name: String,
        macro_type: String,
        severity: Severity,
    },
    /// Match arm with empty catch-all
    EmptyCatchAll {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
}

impl ImplementationViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::EmptyMethodBody { severity, .. } => *severity,
            Self::HardcodedReturnValue { severity, .. } => *severity,
            Self::PassThroughWrapper { severity, .. } => *severity,
            Self::LogOnlyMethod { severity, .. } => *severity,
            Self::StubMacro { severity, .. } => *severity,
            Self::EmptyCatchAll { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for ImplementationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMethodBody {
                file,
                line,
                method_name,
                pattern,
                ..
            } => {
                write!(
                    f,
                    "Empty method body: {}:{} - {}() returns {}",
                    file.display(),
                    line,
                    method_name,
                    pattern
                )
            }
            Self::HardcodedReturnValue {
                file,
                line,
                method_name,
                return_value,
                ..
            } => {
                write!(
                    f,
                    "Hardcoded return: {}:{} - {}() always returns {}",
                    file.display(),
                    line,
                    method_name,
                    return_value
                )
            }
            Self::PassThroughWrapper {
                file,
                line,
                struct_name,
                method_name,
                delegated_to,
                ..
            } => {
                write!(
                    f,
                    "Pass-through wrapper: {}:{} - {}::{}() only delegates to {}",
                    file.display(),
                    line,
                    struct_name,
                    method_name,
                    delegated_to
                )
            }
            Self::LogOnlyMethod {
                file,
                line,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "Log-only method: {}:{} - {}() only contains logging, no logic",
                    file.display(),
                    line,
                    method_name
                )
            }
            Self::StubMacro {
                file,
                line,
                method_name,
                macro_type,
                ..
            } => {
                write!(
                    f,
                    "Stub implementation: {}:{} - {}() uses {}!()",
                    file.display(),
                    line,
                    method_name,
                    macro_type
                )
            }
            Self::EmptyCatchAll {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "Empty catch-all: {}:{} - match arm '_ => {{}}' silently ignores cases: {}",
                    file.display(),
                    line,
                    context
                )
            }
        }
    }
}

/// Implementation quality validator
pub struct ImplementationQualityValidator {
    config: ValidationConfig,
}

impl ImplementationQualityValidator {
    /// Create a new implementation quality validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all implementation quality validations
    pub fn validate_all(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_empty_methods()?);
        violations.extend(self.validate_hardcoded_returns()?);
        violations.extend(self.validate_stub_macros()?);
        violations.extend(self.validate_empty_catch_alls()?);
        violations.extend(self.validate_pass_through_wrappers()?);
        violations.extend(self.validate_log_only_methods()?);
        Ok(violations)
    }

    /// Detect empty method bodies
    pub fn validate_empty_methods(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        // Patterns for empty/trivial method bodies
        let empty_patterns = [
            (r"\{\s*Ok\(\(\)\)\s*\}", "Ok(())"),
            (r"\{\s*None\s*\}", "None"),
            (r"\{\s*Vec::new\(\)\s*\}", "Vec::new()"),
            (r"\{\s*String::new\(\)\s*\}", "String::new()"),
            (r"\{\s*Default::default\(\)\s*\}", "Default::default()"),
            (r"\{\s*Ok\(Vec::new\(\)\)\s*\}", "Ok(Vec::new())"),
            (r"\{\s*Ok\(None\)\s*\}", "Ok(None)"),
            (r"\{\s*Ok\(false\)\s*\}", "Ok(false)"),
            (r"\{\s*Ok\(0\)\s*\}", "Ok(0)"),
        ];

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").ok();
        let compiled_patterns: Vec<_> = empty_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip null provider files - these are intentionally empty
                if file_name.contains("null") || file_name.contains("fake") {
                    continue;
                }

                // Skip test files
                if file_name.contains("_test") || entry.path().to_string_lossy().contains("/tests/")
                {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut current_fn_name = String::new();
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Track function names
                    if let Some(ref re) = fn_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_fn_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        }
                    }

                    // Check for empty patterns
                    for (pattern, desc) in &compiled_patterns {
                        if pattern.is_match(trimmed) {
                            violations.push(ImplementationViolation::EmptyMethodBody {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                method_name: current_fn_name.clone(),
                                pattern: desc.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect hardcoded return values
    pub fn validate_hardcoded_returns(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        let hardcoded_patterns = [
            (r"return\s+true\s*;", "true"),
            (r"return\s+false\s*;", "false"),
            (r"return\s+0\s*;", "0"),
            (r"return\s+1\s*;", "1"),
            (r#"return\s+""\s*;"#, "empty string"),
            (r#"return\s+"[^"]*"\s*;"#, "hardcoded string"),
        ];

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").ok();
        let compiled_patterns: Vec<_> = hardcoded_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip null/fake provider files
                if file_name.contains("null") || file_name.contains("fake") {
                    continue;
                }

                // Skip test files
                if file_name.contains("_test") || entry.path().to_string_lossy().contains("/tests/")
                {
                    continue;
                }

                // Skip constant files
                if file_name == "constants.rs" {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut current_fn_name = String::new();
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Track function names
                    if let Some(ref re) = fn_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_fn_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        }
                    }

                    // Check for hardcoded return patterns
                    // Skip guard clauses (return inside if block for validation checks)
                    // e.g., "if a.len() != b.len() { return false; }" is a valid guard
                    let is_guard_clause = trimmed.starts_with("if ")
                        || trimmed.starts_with("} else if ")
                        || trimmed.contains("return false; }")  // inline guard: if x { return false; }
                        || trimmed.contains("return true; }")   // inline guard: if x { return true; }
                        || (trimmed == "return false;" && content.lines().nth(line_num.saturating_sub(1))
                            .map(|l| l.trim().starts_with("if ")).unwrap_or(false))
                        || (trimmed == "return true;" && content.lines().nth(line_num.saturating_sub(1))
                            .map(|l| l.trim().starts_with("if ")).unwrap_or(false));

                    if is_guard_clause {
                        continue;
                    }

                    for (pattern, desc) in &compiled_patterns {
                        if pattern.is_match(trimmed) {
                            violations.push(ImplementationViolation::HardcodedReturnValue {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                method_name: current_fn_name.clone(),
                                return_value: desc.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect stub macros (todo!, unimplemented!)
    pub fn validate_stub_macros(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        let stub_patterns = [
            (r"todo!\s*\(", "todo"),
            (r"unimplemented!\s*\(", "unimplemented"),
            (r#"panic!\s*\(\s*"not\s+implemented"#, "panic(not implemented)"),
            (r#"panic!\s*\(\s*"TODO"#, "panic(TODO)"),
        ];

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").ok();
        let compiled_patterns: Vec<_> = stub_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut current_fn_name = String::new();
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Track function names
                    if let Some(ref re) = fn_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_fn_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        }
                    }

                    // Check for stub macros
                    for (pattern, macro_type) in &compiled_patterns {
                        if pattern.is_match(trimmed) {
                            violations.push(ImplementationViolation::StubMacro {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                method_name: current_fn_name.clone(),
                                macro_type: macro_type.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect empty catch-all match arms
    pub fn validate_empty_catch_alls(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        let catchall_patterns = [
            r"_\s*=>\s*\{\s*\}",                // _ => {}
            r"_\s*=>\s*\(\)",                   // _ => ()
            r"_\s*=>\s*Ok\(\(\)\)",             // _ => Ok(())
            r"_\s*=>\s*None",                   // _ => None
            r"_\s*=>\s*continue",               // _ => continue (may be intentional)
        ];

        let compiled_patterns: Vec<_> = catchall_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for catch-all patterns
                    for pattern in &compiled_patterns {
                        if pattern.is_match(trimmed) {
                            violations.push(ImplementationViolation::EmptyCatchAll {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                context: trimmed.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect pass-through wrappers
    pub fn validate_pass_through_wrappers(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        // Pattern: method body is just self.inner.method(...) or self.field.method(...)
        let passthrough_pattern = Regex::new(
            r"self\.(\w+)\.(\w+)\s*\([^)]*\)(?:\s*\.await)?(?:\s*\?)?$"
        ).ok();

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").ok();
        let impl_pattern = Regex::new(r"impl(?:\s+\w+\s+for)?\s+(\w+)").ok();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip adapter files - pass-through is expected there
                if file_name.contains("adapter") || file_name.contains("wrapper") {
                    continue;
                }

                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut current_fn_name = String::new();
                let mut current_struct_name = String::new();
                let mut in_test_module = false;
                let mut fn_start_line = 0;
                let mut fn_body_lines: Vec<String> = Vec::new();
                let mut brace_depth = 0;
                let mut in_fn = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Track impl blocks
                    if let Some(ref re) = impl_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_struct_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                        }
                    }

                    // Track function definitions
                    if let Some(ref re) = fn_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_fn_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                            fn_start_line = line_num + 1;
                            fn_body_lines.clear();
                            in_fn = true;
                            brace_depth = 0;
                        }
                    }

                    if in_fn {
                        let open = line.chars().filter(|c| *c == '{').count();
                        let close = line.chars().filter(|c| *c == '}').count();
                        brace_depth += open as i32 - close as i32;

                        // Collect non-empty, non-attribute lines in function body
                        if !trimmed.is_empty() && !trimmed.starts_with("#[") && !trimmed.starts_with("fn ") {
                            fn_body_lines.push(trimmed.to_string());
                        }

                        // End of function
                        if brace_depth <= 0 && open > 0 {
                            // Check if function body is just a single pass-through line
                            let meaningful_lines: Vec<_> = fn_body_lines
                                .iter()
                                .filter(|l| !l.starts_with("{") && !l.starts_with("}") && *l != "{" && *l != "}")
                                .collect();

                            if meaningful_lines.len() == 1 {
                                if let Some(ref re) = passthrough_pattern {
                                    if let Some(cap) = re.captures(meaningful_lines[0]) {
                                        let field = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                                        let method = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                                        // Only flag if method names match (pure delegation)
                                        if method == current_fn_name || method.starts_with(&current_fn_name) {
                                            violations.push(ImplementationViolation::PassThroughWrapper {
                                                file: entry.path().to_path_buf(),
                                                line: fn_start_line,
                                                struct_name: current_struct_name.clone(),
                                                method_name: current_fn_name.clone(),
                                                delegated_to: format!("self.{}.{}()", field, method),
                                                severity: Severity::Info,
                                            });
                                        }
                                    }
                                }
                            }

                            in_fn = false;
                            fn_body_lines.clear();
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect log-only methods
    pub fn validate_log_only_methods(&self) -> Result<Vec<ImplementationViolation>> {
        let mut violations = Vec::new();

        let log_patterns = [
            r"tracing::(info|debug|warn|error|trace)!",
            r"log::(info|debug|warn|error|trace)!",
            r"println!",
            r"eprintln!",
        ];

        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").ok();
        let compiled_log_patterns: Vec<_> = log_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut current_fn_name = String::new();
                let mut in_test_module = false;
                let mut fn_start_line = 0;
                let mut fn_body_lines: Vec<String> = Vec::new();
                let mut brace_depth = 0;
                let mut in_fn = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Track function definitions
                    if let Some(ref re) = fn_pattern {
                        if let Some(cap) = re.captures(trimmed) {
                            current_fn_name = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                            fn_start_line = line_num + 1;
                            fn_body_lines.clear();
                            in_fn = true;
                            brace_depth = 0;
                        }
                    }

                    if in_fn {
                        let open = line.chars().filter(|c| *c == '{').count();
                        let close = line.chars().filter(|c| *c == '}').count();
                        brace_depth += open as i32 - close as i32;

                        // Collect non-empty lines in function body
                        if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                            fn_body_lines.push(trimmed.to_string());
                        }

                        // End of function
                        if brace_depth <= 0 && open > 0 {
                            // Filter meaningful lines (not just braces)
                            let meaningful_lines: Vec<_> = fn_body_lines
                                .iter()
                                .filter(|l| {
                                    !l.starts_with("{") && !l.starts_with("}")
                                    && *l != "{" && *l != "}"
                                    && !l.starts_with("fn ")
                                })
                                .collect();

                            // Check if all meaningful lines are logging
                            if !meaningful_lines.is_empty() {
                                let all_logging = meaningful_lines.iter().all(|line| {
                                    compiled_log_patterns.iter().any(|p| p.is_match(line))
                                });

                                if all_logging && meaningful_lines.len() <= 3 {
                                    violations.push(ImplementationViolation::LogOnlyMethod {
                                        file: entry.path().to_path_buf(),
                                        line: fn_start_line,
                                        method_name: current_fn_name.clone(),
                                        severity: Severity::Warning,
                                    });
                                }
                            }

                            in_fn = false;
                            fn_body_lines.clear();
                        }
                    }
                }
            }
        }

        Ok(violations)
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
    fn test_empty_method_detection() {
        let temp = TempDir::new().unwrap();
        // The validator checks for single-line empty patterns like { Ok(()) }
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub trait Service {
    fn do_something(&self) -> Result<(), Error>;
}

impl Service for MyService {
    fn do_something(&self) -> Result<(), Error> { Ok(()) }
}
"#,
        );

        let validator = ImplementationQualityValidator::new(temp.path());
        let violations = validator.validate_empty_methods().unwrap();

        assert!(!violations.is_empty(), "Should detect empty Ok(()) method");
    }

    #[test]
    fn test_hardcoded_return_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn validate(&self) -> bool {
    return true;
}
"#,
        );

        let validator = ImplementationQualityValidator::new(temp.path());
        let violations = validator.validate_hardcoded_returns().unwrap();

        assert!(!violations.is_empty(), "Should detect hardcoded return true");
    }

    #[test]
    fn test_stub_macro_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn not_implemented_yet(&self) {
    todo!("implement this")
}

pub fn also_not_done(&self) {
    unimplemented!()
}
"#,
        );

        let validator = ImplementationQualityValidator::new(temp.path());
        let violations = validator.validate_stub_macros().unwrap();

        assert_eq!(violations.len(), 2, "Should detect both todo! and unimplemented!");
    }

    #[test]
    fn test_empty_catchall_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn handle_event(&self, event: Event) {
    match event {
        Event::Created => handle_created(),
        Event::Updated => handle_updated(),
        _ => {}
    }
}
"#,
        );

        let validator = ImplementationQualityValidator::new(temp.path());
        let violations = validator.validate_empty_catch_alls().unwrap();

        assert!(!violations.is_empty(), "Should detect empty catch-all _ => {{}}");
    }

    #[test]
    fn test_null_provider_exempt() {
        let temp = TempDir::new().unwrap();

        // Create a null provider file
        let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("null.rs"),
            r#"
pub fn do_nothing(&self) -> Result<(), Error> {
    Ok(())
}
"#,
        ).unwrap();

        fs::write(
            temp.path().join("crates").join("mcb-test").join("Cargo.toml"),
            r#"
[package]
name = "mcb-test"
version = "0.1.1"
"#,
        ).unwrap();

        let validator = ImplementationQualityValidator::new(temp.path());
        let violations = validator.validate_empty_methods().unwrap();

        assert!(violations.is_empty(), "Null provider files should be exempt");
    }
}
