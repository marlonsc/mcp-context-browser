//! KISS Principle Validation
//!
//! Validates code against the KISS principle (Keep It Simple, Stupid):
//! - Struct field count (max 7)
//! - Function parameter count (max 5)
//! - Builder complexity (max 7 optional fields)
//! - Nesting depth (max 3 levels)
//! - Function length (max 50 lines)

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Maximum allowed struct fields (KISS)
pub const MAX_STRUCT_FIELDS: usize = 7;

/// Maximum allowed function parameters (KISS)
pub const MAX_FUNCTION_PARAMS: usize = 5;

/// Maximum allowed builder optional fields (KISS)
pub const MAX_BUILDER_FIELDS: usize = 7;

/// Maximum allowed nesting depth (KISS)
pub const MAX_NESTING_DEPTH: usize = 3;

/// Maximum allowed function lines (KISS/SRP)
pub const MAX_FUNCTION_LINES: usize = 50;

/// KISS violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KissViolation {
    /// Struct with too many fields (>7)
    StructTooManyFields {
        file: PathBuf,
        line: usize,
        struct_name: String,
        field_count: usize,
        max_allowed: usize,
        severity: Severity,
    },

    /// Function with too many parameters (>5)
    FunctionTooManyParams {
        file: PathBuf,
        line: usize,
        function_name: String,
        param_count: usize,
        max_allowed: usize,
        severity: Severity,
    },

    /// Builder with too many optional fields (>7)
    BuilderTooComplex {
        file: PathBuf,
        line: usize,
        builder_name: String,
        optional_field_count: usize,
        max_allowed: usize,
        severity: Severity,
    },

    /// Nested conditionals too deep (>3 levels)
    DeepNesting {
        file: PathBuf,
        line: usize,
        nesting_level: usize,
        max_allowed: usize,
        context: String,
        severity: Severity,
    },

    /// Function too long (>50 lines)
    FunctionTooLong {
        file: PathBuf,
        line: usize,
        function_name: String,
        line_count: usize,
        max_allowed: usize,
        severity: Severity,
    },
}

impl KissViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::StructTooManyFields { severity, .. } => *severity,
            Self::FunctionTooManyParams { severity, .. } => *severity,
            Self::BuilderTooComplex { severity, .. } => *severity,
            Self::DeepNesting { severity, .. } => *severity,
            Self::FunctionTooLong { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for KissViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructTooManyFields {
                file,
                line,
                struct_name,
                field_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Struct {} has too many fields: {}:{} ({} fields, max: {})",
                    struct_name,
                    file.display(),
                    line,
                    field_count,
                    max_allowed
                )
            }
            Self::FunctionTooManyParams {
                file,
                line,
                function_name,
                param_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Function {} has too many parameters: {}:{} ({} params, max: {})",
                    function_name,
                    file.display(),
                    line,
                    param_count,
                    max_allowed
                )
            }
            Self::BuilderTooComplex {
                file,
                line,
                builder_name,
                optional_field_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Builder {} is too complex: {}:{} ({} optional fields, max: {})",
                    builder_name,
                    file.display(),
                    line,
                    optional_field_count,
                    max_allowed
                )
            }
            Self::DeepNesting {
                file,
                line,
                nesting_level,
                max_allowed,
                context,
                ..
            } => {
                write!(
                    f,
                    "KISS: Deep nesting at {}:{} ({} levels, max: {}) - {}",
                    file.display(),
                    line,
                    nesting_level,
                    max_allowed,
                    context
                )
            }
            Self::FunctionTooLong {
                file,
                line,
                function_name,
                line_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "KISS: Function {} is too long: {}:{} ({} lines, max: {})",
                    function_name,
                    file.display(),
                    line,
                    line_count,
                    max_allowed
                )
            }
        }
    }
}

/// KISS principle validator
pub struct KissValidator {
    config: ValidationConfig,
    max_struct_fields: usize,
    max_function_params: usize,
    max_builder_fields: usize,
    max_nesting_depth: usize,
    max_function_lines: usize,
}

impl KissValidator {
    /// Create a new KISS validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            max_struct_fields: MAX_STRUCT_FIELDS,
            max_function_params: MAX_FUNCTION_PARAMS,
            max_builder_fields: MAX_BUILDER_FIELDS,
            max_nesting_depth: MAX_NESTING_DEPTH,
            max_function_lines: MAX_FUNCTION_LINES,
        }
    }

    /// Set custom max struct fields
    pub fn with_max_struct_fields(mut self, max: usize) -> Self {
        self.max_struct_fields = max;
        self
    }

    /// Set custom max function parameters
    pub fn with_max_function_params(mut self, max: usize) -> Self {
        self.max_function_params = max;
        self
    }

    /// Run all KISS validations
    pub fn validate_all(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_struct_fields()?);
        violations.extend(self.validate_function_params()?);
        violations.extend(self.validate_builder_complexity()?);
        violations.extend(self.validate_nesting_depth()?);
        violations.extend(self.validate_function_length()?);
        Ok(violations)
    }

    /// Check for structs with too many fields
    pub fn validate_struct_fields(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)\s*\{").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        test_brace_depth = brace_depth;
                    }

                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Exit test module when braces close
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }

                    // Skip test modules
                    if in_test_module {
                        continue;
                    }

                    if let Some(cap) = struct_pattern.captures(line) {
                        let struct_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Count fields in struct
                        let field_count = self.count_struct_fields(&lines, line_num);

                        if field_count > self.max_struct_fields {
                            violations.push(KissViolation::StructTooManyFields {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                struct_name: struct_name.to_string(),
                                field_count,
                                max_allowed: self.max_struct_fields,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for functions with too many parameters
    pub fn validate_function_params(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern = Regex::new(
            r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)",
        )
        .expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        test_brace_depth = brace_depth;
                    }

                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Exit test module when braces close
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }

                    // Skip test modules
                    if in_test_module {
                        continue;
                    }

                    // Only check lines that contain function definitions
                    if !line.contains("fn ") {
                        continue;
                    }

                    // Build full function signature (may span multiple lines)
                    let mut full_line = line.to_string();
                    let mut idx = line_num + 1;
                    while !full_line.contains(')') && idx < lines.len() {
                        full_line.push_str(lines[idx]);
                        idx += 1;
                    }

                    if let Some(cap) = fn_pattern.captures(&full_line) {
                        let fn_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let params = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                        // Count parameters (comma-separated, excluding &self/self)
                        let param_count = self.count_function_params(params);

                        if param_count > self.max_function_params {
                            violations.push(KissViolation::FunctionTooManyParams {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                function_name: fn_name.to_string(),
                                param_count,
                                max_allowed: self.max_function_params,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for builders with too many optional fields
    pub fn validate_builder_complexity(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let builder_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Builder)\s*").expect("Invalid regex");
        let option_pattern = Regex::new(r"Option<").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
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
                    if let Some(cap) = builder_pattern.captures(line) {
                        let builder_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Count Option<> fields in builder struct
                        let optional_count =
                            self.count_optional_fields(&lines, line_num, &option_pattern);

                        if optional_count > self.max_builder_fields {
                            violations.push(KissViolation::BuilderTooComplex {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                builder_name: builder_name.to_string(),
                                optional_field_count: optional_count,
                                max_allowed: self.max_builder_fields,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for deeply nested code
    pub fn validate_nesting_depth(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let control_flow_pattern =
            Regex::new(r"\b(if|match|for|while|loop)\b").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;

                // Track nesting depth
                let mut nesting_depth: usize = 0;
                let mut brace_depth: i32 = 0;
                let mut reported_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        test_brace_depth = brace_depth;
                    }

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track control flow nesting
                    if control_flow_pattern.is_match(line) && line.contains('{') {
                        nesting_depth += 1;

                        // Check if too deep and not already reported nearby
                        if nesting_depth > self.max_nesting_depth {
                            let nearby_reported = reported_lines
                                .iter()
                                .any(|&l| (l as isize - line_num as isize).abs() < 5);

                            if !nearby_reported {
                                violations.push(KissViolation::DeepNesting {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    nesting_level: nesting_depth,
                                    max_allowed: self.max_nesting_depth,
                                    context: trimmed.chars().take(60).collect(),
                                    severity: Severity::Warning,
                                });
                                reported_lines.insert(line_num);
                            }
                        }
                    }

                    // Track brace depth for control flow
                    let open_braces = line.chars().filter(|c| *c == '{').count();
                    let close_braces = line.chars().filter(|c| *c == '}').count();

                    brace_depth += open_braces as i32;
                    brace_depth -= close_braces as i32;

                    // Decrease nesting on closing braces
                    if close_braces > 0 && nesting_depth > 0 {
                        nesting_depth = nesting_depth.saturating_sub(close_braces);
                    }

                    // Exit test module when braces close
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for functions that are too long
    pub fn validate_function_length(&self) -> Result<Vec<KissViolation>> {
        let mut violations = Vec::new();
        let fn_pattern =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track test modules to skip
                let mut in_test_module = false;
                let mut test_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Track test module boundaries
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        test_brace_depth = brace_depth;
                    }

                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Exit test module when braces close
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }

                    // Skip test modules
                    if in_test_module {
                        continue;
                    }

                    if let Some(cap) = fn_pattern.captures(line) {
                        let fn_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip test functions
                        if fn_name.starts_with("test_") {
                            continue;
                        }

                        // Skip trait function declarations (no body, ends with ;)
                        if self.is_trait_fn_declaration(&lines, line_num) {
                            continue;
                        }

                        // Count lines in function
                        let line_count = self.count_function_lines(&lines, line_num);

                        if line_count > self.max_function_lines {
                            violations.push(KissViolation::FunctionTooLong {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                function_name: fn_name.to_string(),
                                line_count,
                                max_allowed: self.max_function_lines,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Count fields in a struct
    fn count_struct_fields(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_struct = false;
        let mut field_count = 0;
        let field_pattern = Regex::new(r"^\s*(?:pub\s+)?[a-z_][a-z0-9_]*\s*:").expect("Invalid regex");

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_struct = true;
            }
            if in_struct {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count field declarations (lines with `:` that look like fields)
                if brace_depth >= 1 && field_pattern.is_match(line) {
                    field_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        field_count
    }

    /// Count function parameters (excluding self)
    fn count_function_params(&self, params: &str) -> usize {
        if params.trim().is_empty() {
            return 0;
        }

        // Split by comma and count, excluding &self, self, &mut self
        let parts: Vec<&str> = params.split(',').collect();
        let mut count = 0;

        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("&self")
                && !trimmed.starts_with("self")
                && !trimmed.starts_with("&mut self")
            {
                count += 1;
            }
        }

        count
    }

    /// Count Option<> fields in a struct
    fn count_optional_fields(
        &self,
        lines: &[&str],
        start_line: usize,
        option_pattern: &Regex,
    ) -> usize {
        let mut brace_depth = 0;
        let mut in_struct = false;
        let mut optional_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_struct = true;
            }
            if in_struct {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count Option<> types
                if brace_depth >= 1 && option_pattern.is_match(line) {
                    optional_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        optional_count
    }

    /// Check if a function declaration is a trait method without a body
    /// (ends with `;` before any `{`)
    fn is_trait_fn_declaration(&self, lines: &[&str], start_line: usize) -> bool {
        // Look at the function signature lines until we find either { or ;
        // If we find ; first, it's a trait function declaration without a body
        for line in &lines[start_line..] {
            // Check for opening brace (function body starts)
            if line.contains('{') {
                return false;
            }
            // Check for semicolon (trait function declaration ends)
            if line.trim().ends_with(';') {
                return true;
            }
            // Check for semicolon after return type annotation
            // e.g., "fn foo(&self) -> Result<T>;"
            if line.contains(';') && !line.contains('{') {
                return true;
            }
        }
        false
    }

    /// Count lines in a function
    fn count_function_lines(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_fn = false;
        let mut line_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_fn = true;
            }
            if in_fn {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();
                line_count += 1;

                if brace_depth == 0 {
                    break;
                }
            }
        }

        line_count
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
version = "0.1.0"
"#,
                name
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_struct_too_many_fields() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub struct TooManyFields {
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
}
"#,
        );

        let validator = KissValidator::new(temp.path());
        let violations = validator.validate_struct_fields().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            KissViolation::StructTooManyFields { field_count, .. } => {
                assert!(*field_count > 7);
            }
            _ => panic!("Expected StructTooManyFields"),
        }
    }

    #[test]
    fn test_function_too_many_params() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn too_many_params(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    a + b + c + d + e + f
}
"#,
        );

        let validator = KissValidator::new(temp.path());
        let violations = validator.validate_function_params().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            KissViolation::FunctionTooManyParams { param_count, .. } => {
                assert!(*param_count > 5);
            }
            _ => panic!("Expected FunctionTooManyParams"),
        }
    }

    #[test]
    fn test_function_too_long() {
        let temp = TempDir::new().unwrap();
        let long_function = format!(
            r#"
pub fn long_function() {{
{}
}}
"#,
            (0..60).map(|i| format!("    let x{} = {};", i, i)).collect::<Vec<_>>().join("\n")
        );
        create_test_crate(&temp, "mcb-test", &long_function);

        let validator = KissValidator::new(temp.path());
        let violations = validator.validate_function_length().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            KissViolation::FunctionTooLong { line_count, .. } => {
                assert!(*line_count > 50);
            }
            _ => panic!("Expected FunctionTooLong"),
        }
    }

    #[test]
    fn test_acceptable_struct() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub struct AcceptableFields {
    field1: String,
    field2: String,
    field3: String,
}
"#,
        );

        let validator = KissValidator::new(temp.path());
        let violations = validator.validate_struct_fields().unwrap();

        assert!(violations.is_empty());
    }
}
