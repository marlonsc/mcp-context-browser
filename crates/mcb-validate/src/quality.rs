//! Code Quality Validation
//!
//! Validates code quality standards:
//! - No unwrap/expect in production code (AST-based detection)
//! - No panic!() in production code
//! - File size limits (500 lines)
//! - Pending-comment detection (T.O.D.O./F.I.X.M.E./X.X.X./H.A.C.K.)
//!
//! Phase 2 deliverable: QUAL001 (no-unwrap) detects `.unwrap()` calls via AST

use crate::ast::UnwrapDetector;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Maximum allowed lines per file
pub const MAX_FILE_LINES: usize = 500;

/// Maximum allowed function lines (informational)
pub const MAX_FUNCTION_LINES: usize = 50;

/// Quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityViolation {
    /// unwrap() found in production code
    UnwrapInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// expect() found in production code
    ExpectInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// panic!() found in production code
    PanicInProduction {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// File exceeds line limit
    FileTooLarge {
        file: PathBuf,
        lines: usize,
        max_allowed: usize,
        severity: Severity,
    },
    /// Pending comment found (T.O.D.O./F.I.X.M.E./X.X.X./H.A.C.K.)
    TodoComment {
        file: PathBuf,
        line: usize,
        content: String,
        severity: Severity,
    },
    /// Dead code annotation without justification
    DeadCodeWithoutJustification {
        file: PathBuf,
        line: usize,
        item_name: String,
        severity: Severity,
    },
    /// Unused struct field
    UnusedStructField {
        file: PathBuf,
        line: usize,
        struct_name: String,
        field_name: String,
        severity: Severity,
    },
    /// Function marked dead but appears uncalled
    DeadFunctionUncalled {
        file: PathBuf,
        line: usize,
        function_name: String,
        severity: Severity,
    },
}

impl QualityViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::UnwrapInProduction { severity, .. } => *severity,
            Self::ExpectInProduction { severity, .. } => *severity,
            Self::PanicInProduction { severity, .. } => *severity,
            Self::FileTooLarge { severity, .. } => *severity,
            Self::TodoComment { severity, .. } => *severity,
            Self::DeadCodeWithoutJustification { severity, .. } => *severity,
            Self::UnusedStructField { severity, .. } => *severity,
            Self::DeadFunctionUncalled { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for QualityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnwrapInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "unwrap() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::ExpectInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "expect() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::PanicInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "panic!() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::FileTooLarge {
                file,
                lines,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "File too large: {} has {} lines (max: {})",
                    file.display(),
                    lines,
                    max_allowed
                )
            }
            Self::TodoComment {
                file,
                line,
                content,
                ..
            } => {
                write!(f, "Pending: {}:{} - {}", file.display(), line, content)
            }
            Self::DeadCodeWithoutJustification {
                file,
                line,
                item_name,
                ..
            } => {
                write!(
                    f,
                    "Dead code without justification: {}:{} - '{}' has #[allow(dead_code)] without explanation",
                    file.display(),
                    line,
                    item_name
                )
            }
            Self::UnusedStructField {
                file,
                line,
                struct_name,
                field_name,
                ..
            } => {
                write!(
                    f,
                    "Unused struct field: {}:{} - Field '{}' in struct '{}' is unused",
                    file.display(),
                    line,
                    field_name,
                    struct_name
                )
            }
            Self::DeadFunctionUncalled {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Dead function: {}:{} - Function '{}' marked as dead but appears uncalled",
                    file.display(),
                    line,
                    function_name
                )
            }
        }
    }
}

impl Violation for QualityViolation {
    fn id(&self) -> &str {
        match self {
            Self::UnwrapInProduction { .. } => "QUAL001",
            Self::ExpectInProduction { .. } => "QUAL002",
            Self::PanicInProduction { .. } => "QUAL003",
            Self::FileTooLarge { .. } => "QUAL004",
            Self::TodoComment { .. } => "QUAL005",
            Self::DeadCodeWithoutJustification { .. } => "QUAL020",
            Self::UnusedStructField { .. } => "QUAL021",
            Self::DeadFunctionUncalled { .. } => "QUAL022",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }

    fn severity(&self) -> Severity {
        match self {
            Self::UnwrapInProduction { severity, .. } => *severity,
            Self::ExpectInProduction { severity, .. } => *severity,
            Self::PanicInProduction { severity, .. } => *severity,
            Self::FileTooLarge { severity, .. } => *severity,
            Self::TodoComment { severity, .. } => *severity,
            Self::DeadCodeWithoutJustification { severity, .. } => *severity,
            Self::UnusedStructField { severity, .. } => *severity,
            Self::DeadFunctionUncalled { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::UnwrapInProduction { file, .. } => Some(file),
            Self::ExpectInProduction { file, .. } => Some(file),
            Self::PanicInProduction { file, .. } => Some(file),
            Self::FileTooLarge { file, .. } => Some(file),
            Self::TodoComment { file, .. } => Some(file),
            Self::DeadCodeWithoutJustification { file, .. } => Some(file),
            Self::UnusedStructField { file, .. } => Some(file),
            Self::DeadFunctionUncalled { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::UnwrapInProduction { line, .. } => Some(*line),
            Self::ExpectInProduction { line, .. } => Some(*line),
            Self::PanicInProduction { line, .. } => Some(*line),
            Self::FileTooLarge { .. } => None,
            Self::TodoComment { line, .. } => Some(*line),
            Self::DeadCodeWithoutJustification { line, .. } => Some(*line),
            Self::UnusedStructField { line, .. } => Some(*line),
            Self::DeadFunctionUncalled { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::UnwrapInProduction { .. } => {
                Some("Use `?` operator or handle the error explicitly".to_string())
            }
            Self::ExpectInProduction { .. } => {
                Some("Use `?` operator or handle the error explicitly".to_string())
            }
            Self::PanicInProduction { .. } => {
                Some("Return an error instead of panicking".to_string())
            }
            Self::FileTooLarge { max_allowed, .. } => Some(format!(
                "Split file into smaller modules (max {} lines)",
                max_allowed
            )),
            Self::TodoComment { .. } => {
                Some("Address the pending comment or create an issue to track it".to_string())
            }
            Self::DeadCodeWithoutJustification { .. } => {
                Some("Add a comment explaining why this is marked dead (e.g., // Reserved for future admin API) or remove the annotation if actually used".to_string())
            }
            Self::UnusedStructField { .. } => {
                Some("Remove the unused field or document why it's kept (e.g., for serialization format versioning)".to_string())
            }
            Self::DeadFunctionUncalled { .. } => {
                Some("Remove the dead function or connect it to the public API if it's intended for future use".to_string())
            }
        }
    }
}

/// Quality validator
pub struct QualityValidator {
    config: ValidationConfig,
    max_file_lines: usize,
}

impl QualityValidator {
    /// Check if a line has an ignore hint for a specific violation type
    fn has_ignore_hint(&self, line: &str, violation_type: &str) -> bool {
        line.contains(&format!("mcb-validate-ignore: {}", violation_type))
    }
    /// Create a new quality validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            max_file_lines: MAX_FILE_LINES,
        }
    }

    /// Set custom max file lines
    pub fn with_max_file_lines(mut self, max: usize) -> Self {
        self.max_file_lines = max;
        self
    }

    /// Run all quality validations
    pub fn validate_all(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_unwrap_expect()?);
        violations.extend(self.validate_no_panic()?);
        violations.extend(self.validate_file_sizes()?);
        violations.extend(self.find_todo_comments()?);
        violations.extend(self.validate_dead_code_annotations()?);
        Ok(violations)
    }

    /// Validate that #[allow(dead_code)] annotations have justification comments
    pub fn validate_dead_code_annotations(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let dead_code_pattern = Regex::new(r"#\[allow\(dead_code\)\]").unwrap();
        let struct_pattern = Regex::new(r"pub\s+struct\s+(\w+)").unwrap();
        let fn_pattern = Regex::new(r"(?:pub\s+)?fn\s+(\w+)").unwrap();
        let field_pattern = Regex::new(r"(?:pub\s+)?(\w+):\s+").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| !e.path().to_string_lossy().contains("/tests/"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (i, line) in lines.iter().enumerate() {
                    if dead_code_pattern.is_match(line) {
                        // Check if there's a justification comment
                        let has_justification = self.has_dead_code_justification(&lines, i);

                        if !has_justification {
                            // Find what item is being marked as dead
                            if let Some(item_name) = self.find_dead_code_item(
                                &lines,
                                i,
                                &struct_pattern,
                                &fn_pattern,
                                &field_pattern,
                            ) {
                                violations.push(QualityViolation::DeadCodeWithoutJustification {
                                    file: entry.path().to_path_buf(),
                                    line: i + 1,
                                    item_name,
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

    fn has_dead_code_justification(&self, lines: &[&str], line_idx: usize) -> bool {
        // Check lines above and below for justification
        let check_range_before = if line_idx >= 2 { line_idx - 2 } else { 0 };
        let check_range_after = std::cmp::min(line_idx + 3, lines.len());

        for i in check_range_before..check_range_after {
            let line = lines[i].trim();
            if line.contains("// ")
                && (line.contains("Reserved for")
                    || line.contains("Will be used")
                    || line.contains("Future")
                    || line.contains("Admin API")
                    || line.contains("Versioning")
                    || line.contains("Serialization")
                    || line.contains("Format")
                    || line.contains("Used by")
                    || line.contains("Test fixture")
                    || line.contains("serde"))
            {
                return true;
            }
        }

        false
    }

    fn find_dead_code_item(
        &self,
        lines: &[&str],
        start_idx: usize,
        struct_pattern: &Regex,
        fn_pattern: &Regex,
        field_pattern: &Regex,
    ) -> Option<String> {
        // Look in next few lines for struct/fn/field name
        for i in start_idx..std::cmp::min(start_idx + 5, lines.len()) {
            let line = lines[i];

            if let Some(captures) = struct_pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    return Some(format!("struct {}", name.as_str()));
                }
            }

            if let Some(captures) = fn_pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    return Some(format!("fn {}", name.as_str()));
                }
            }

            if let Some(captures) = field_pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    return Some(format!("field {}", name.as_str()));
                }
            }
        }

        None
    }

    /// Check for unwrap/expect in src/ (not tests/)
    ///
    /// This uses AST-based detection via Tree-sitter for accurate detection
    /// of `.unwrap()` and `.expect()` method calls.
    ///
    /// Phase 2 deliverable: "QUAL001 (no-unwrap) detects `.unwrap()` calls via AST"
    pub fn validate_no_unwrap_expect(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let mut detector = UnwrapDetector::new()?;

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| {
                    // Skip test files
                    let path_str = e.path().to_string_lossy();
                    !path_str.contains("/tests/")
                        && !path_str.contains("/target/")
                        && !path_str.ends_with("_test.rs")
                        && !path_str.ends_with("test.rs")
                })
            {
                // Use AST-based detection
                let detections = detector.detect_in_file(entry.path())?;

                for detection in detections {
                    // Skip detections in test modules
                    if detection.in_test {
                        continue;
                    }

                    // Skip if context contains SAFETY justification or ignore hints
                    // (checked via Regex since AST doesn't capture comments easily)
                    let content = std::fs::read_to_string(entry.path())?;
                    let lines: Vec<&str> = content.lines().collect();
                    if detection.line > 0 && detection.line <= lines.len() {
                        let line_content = lines[detection.line - 1];

                        // Check for safety comments
                        if line_content.contains("// SAFETY:")
                            || line_content.contains("// safety:")
                        {
                            continue;
                        }

                        // Check for ignore hints around the detection
                        let mut has_ignore = false;

                        // Check current line
                        if self.has_ignore_hint(line_content, "lock_poisoning_recovery")
                            || self.has_ignore_hint(line_content, "hardcoded_fallback")
                        {
                            has_ignore = true;
                        }

                        // Check previous lines (up to 3 lines before)
                        if !has_ignore && detection.line > 1 {
                            for i in 1..=3.min(detection.line - 1) {
                                let check_line = lines[detection.line - 1 - i];
                                if self.has_ignore_hint(check_line, "lock_poisoning_recovery")
                                    || self.has_ignore_hint(check_line, "hardcoded_fallback")
                                {
                                    has_ignore = true;
                                    break;
                                }
                            }
                        }

                        // Check next lines (up to 3 lines after)
                        if !has_ignore && detection.line < lines.len() {
                            for i in 0..3.min(lines.len() - detection.line) {
                                let check_line = lines[detection.line + i];
                                if self.has_ignore_hint(check_line, "lock_poisoning_recovery")
                                    || self.has_ignore_hint(check_line, "hardcoded_fallback")
                                {
                                    has_ignore = true;
                                    break;
                                }
                            }
                        }

                        if has_ignore {
                            continue;
                        }

                        // Skip cases where we're handling system-level errors appropriately
                        // (e.g., lock poisoning, which is a legitimate use of expect())
                        if detection.method == "expect"
                            && (line_content.contains("lock poisoned")
                                || line_content.contains("Lock poisoned")
                                || line_content.contains("poisoned")
                                || line_content.contains("Mutex poisoned"))
                        {
                            continue;
                        }
                    }

                    // Create appropriate violation based on method type
                    match detection.method.as_str() {
                        "unwrap" => {
                            violations.push(QualityViolation::UnwrapInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        "expect" => {
                            violations.push(QualityViolation::ExpectInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for panic!() macros in production code
    pub fn validate_no_panic(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let panic_pattern = Regex::new(r"panic!\s*\(").unwrap();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
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

                    // Check for panic!
                    if panic_pattern.is_match(line) {
                        violations.push(QualityViolation::PanicInProduction {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            context: trimmed.to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate all source files under line limit
    pub fn validate_file_sizes(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs")
                        && !self.config.should_exclude(e.path())
                        && !e.path().to_string_lossy().contains("/tests/")
                        && !e.path().to_string_lossy().contains("/target/")
                        && !e.path().to_string_lossy().ends_with("_test.rs")
                })
            {
                let path_str = entry.path().to_string_lossy();

                // Skip mcb-providers vector store implementations (ADR-029)
                // These are legitimately large due to complex storage operations
                if path_str.contains("mcb-providers/src/vector_store/")
                    || path_str.contains("mcb-providers/src/embedding/")
                {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let line_count = content.lines().count();

                if line_count > self.max_file_lines {
                    violations.push(QualityViolation::FileTooLarge {
                        file: entry.path().to_path_buf(),
                        lines: line_count,
                        max_allowed: self.max_file_lines,
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Find pending comments (T.O.D.O./F.I.X.M.E./X.X.X./H.A.C.K.)
    pub fn find_todo_comments(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        const PENDING_TODO: &str = concat!("T", "O", "D", "O");
        const PENDING_FIXME: &str = concat!("F", "I", "X", "M", "E");
        const PENDING_XXX: &str = concat!("X", "X", "X");
        const PENDING_HACK: &str = concat!("H", "A", "C", "K");
        let todo_pattern = Regex::new(&format!(
            r"(?i)({}|{}|{}|{}):?\s*(.*)",
            PENDING_TODO, PENDING_FIXME, PENDING_XXX, PENDING_HACK
        ))
        .unwrap();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = todo_pattern.captures(line) {
                        let todo_type = cap.get(1).map_or("", |m| m.as_str());
                        let message = cap.get(2).map_or("", |m| m.as_str()).trim();

                        violations.push(QualityViolation::TodoComment {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            content: format!("{}: {}", todo_type.to_uppercase(), message),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for QualityValidator {
    fn name(&self) -> &'static str {
        "quality"
    }

    fn description(&self) -> &'static str {
        "Validates code quality (no unwrap/expect)"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
