//! SOLID Principles Validation
//!
//! Validates code against SOLID principles:
//! - SRP: Single Responsibility Principle (file/struct size, cohesion)
//! - OCP: Open/Closed Principle (excessive match statements)
//! - LSP: Liskov Substitution Principle (partial implementations)
//! - ISP: Interface Segregation Principle (large traits)
//! - DIP: Dependency Inversion Principle (concrete dependencies)

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Maximum methods for a single trait (ISP)
pub const MAX_TRAIT_METHODS: usize = 10;

/// Maximum lines for a single struct/impl block (SRP)
pub const MAX_STRUCT_LINES: usize = 200;

/// Maximum match arms before warning (OCP)
pub const MAX_MATCH_ARMS: usize = 10;

/// Maximum methods for a single impl block (SRP)
pub const MAX_IMPL_METHODS: usize = 15;

/// SOLID violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolidViolation {
    /// SRP: Struct/Impl has too many responsibilities (too large)
    TooManyResponsibilities {
        file: PathBuf,
        line: usize,
        item_type: String,
        item_name: String,
        line_count: usize,
        max_allowed: usize,
        suggestion: String,
        severity: Severity,
    },

    /// OCP: Large match statement that may need extension pattern
    ExcessiveMatchArms {
        file: PathBuf,
        line: usize,
        arm_count: usize,
        max_recommended: usize,
        suggestion: String,
        severity: Severity,
    },

    /// ISP: Trait has too many methods
    TraitTooLarge {
        file: PathBuf,
        line: usize,
        trait_name: String,
        method_count: usize,
        max_allowed: usize,
        suggestion: String,
        severity: Severity,
    },

    /// DIP: Module depends on concrete implementation
    ConcreteDependency {
        file: PathBuf,
        line: usize,
        dependency: String,
        layer: String,
        suggestion: String,
        severity: Severity,
    },

    /// SRP: File has multiple unrelated structs
    MultipleUnrelatedStructs {
        file: PathBuf,
        struct_names: Vec<String>,
        suggestion: String,
        severity: Severity,
    },

    /// LSP: Trait method not implemented (only panic/todo)
    PartialTraitImplementation {
        file: PathBuf,
        line: usize,
        impl_name: String,
        method_name: String,
        severity: Severity,
    },

    /// SRP: Impl block has too many methods
    ImplTooManyMethods {
        file: PathBuf,
        line: usize,
        type_name: String,
        method_count: usize,
        max_allowed: usize,
        suggestion: String,
        severity: Severity,
    },

    /// OCP: String-based type dispatch instead of polymorphism
    StringBasedDispatch {
        file: PathBuf,
        line: usize,
        match_expression: String,
        suggestion: String,
        severity: Severity,
    },
}

impl SolidViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::TooManyResponsibilities { severity, .. } => *severity,
            Self::ExcessiveMatchArms { severity, .. } => *severity,
            Self::TraitTooLarge { severity, .. } => *severity,
            Self::ConcreteDependency { severity, .. } => *severity,
            Self::MultipleUnrelatedStructs { severity, .. } => *severity,
            Self::PartialTraitImplementation { severity, .. } => *severity,
            Self::ImplTooManyMethods { severity, .. } => *severity,
            Self::StringBasedDispatch { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for SolidViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyResponsibilities {
                file,
                line,
                item_type,
                item_name,
                line_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: {} {} too large: {}:{} ({} lines, max: {}) - {}",
                    item_type,
                    item_name,
                    file.display(),
                    line,
                    line_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::ExcessiveMatchArms {
                file,
                line,
                arm_count,
                max_recommended,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "OCP: Excessive match arms: {}:{} ({} arms, recommended max: {}) - {}",
                    file.display(),
                    line,
                    arm_count,
                    max_recommended,
                    suggestion
                )
            }
            Self::TraitTooLarge {
                file,
                line,
                trait_name,
                method_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "ISP: Trait {} too large: {}:{} ({} methods, max: {}) - {}",
                    trait_name,
                    file.display(),
                    line,
                    method_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::ConcreteDependency {
                file,
                line,
                dependency,
                layer,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "DIP: Concrete dependency: {}:{} - {} in {} layer - {}",
                    file.display(),
                    line,
                    dependency,
                    layer,
                    suggestion
                )
            }
            Self::MultipleUnrelatedStructs {
                file,
                struct_names,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: Multiple unrelated structs in {}: [{}] - {}",
                    file.display(),
                    struct_names.join(", "),
                    suggestion
                )
            }
            Self::PartialTraitImplementation {
                file,
                line,
                impl_name,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "LSP: Partial implementation: {}:{} - {}::{} uses panic!/todo!",
                    file.display(),
                    line,
                    impl_name,
                    method_name
                )
            }
            Self::ImplTooManyMethods {
                file,
                line,
                type_name,
                method_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: Impl {} has too many methods: {}:{} ({} methods, max: {}) - {}",
                    type_name,
                    file.display(),
                    line,
                    method_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::StringBasedDispatch {
                file,
                line,
                match_expression,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "OCP: String-based dispatch: {}:{} - {} - {}",
                    file.display(),
                    line,
                    match_expression,
                    suggestion
                )
            }
        }
    }
}

/// SOLID principles validator
pub struct SolidValidator {
    config: ValidationConfig,
    max_trait_methods: usize,
    max_struct_lines: usize,
    max_match_arms: usize,
    max_impl_methods: usize,
}

impl SolidValidator {
    /// Create a new SOLID validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            max_trait_methods: MAX_TRAIT_METHODS,
            max_struct_lines: MAX_STRUCT_LINES,
            max_match_arms: MAX_MATCH_ARMS,
            max_impl_methods: MAX_IMPL_METHODS,
        }
    }

    /// Run all SOLID validations
    pub fn validate_all(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_srp()?);
        violations.extend(self.validate_ocp()?);
        violations.extend(self.validate_isp()?);
        violations.extend(self.validate_lsp()?);
        violations.extend(self.validate_impl_method_count()?);
        violations.extend(self.validate_string_dispatch()?);
        Ok(violations)
    }

    /// SRP: Check for structs/impls that are too large
    pub fn validate_srp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_pattern = Regex::new(r"impl(?:<[^>]*>)?\s+(?:([A-Z][a-zA-Z0-9_]*)|[A-Z][a-zA-Z0-9_]*\s+for\s+([A-Z][a-zA-Z0-9_]*))").expect("Invalid regex");
        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

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
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Skip files that typically have multiple related structs
                let is_collection_file = file_name == "types.rs"
                    || file_name == "models.rs"
                    || file_name == "args.rs"
                    || file_name == "handlers.rs"
                    || file_name == "responses.rs"
                    || file_name == "requests.rs"
                    || file_name == "dto.rs"
                    || file_name == "entities.rs"
                    || file_name == "admin.rs";  // Port files group related types

                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                // Track struct definitions and their sizes
                let mut structs_in_file: Vec<(String, usize)> = Vec::new();

                for (line_num, line) in lines.iter().enumerate() {
                    // Check struct sizes
                    if let Some(cap) = struct_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        structs_in_file.push((name.to_string(), line_num + 1));
                    }

                    // Check impl block sizes
                    if let Some(cap) = impl_pattern.captures(line) {
                        let name = cap.get(1).or(cap.get(2)).map(|m| m.as_str()).unwrap_or("");

                        // Count lines in impl block
                        let block_lines = self.count_block_lines(&lines, line_num);

                        if block_lines > self.max_struct_lines {
                            violations.push(SolidViolation::TooManyResponsibilities {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                item_type: "impl".to_string(),
                                item_name: name.to_string(),
                                line_count: block_lines,
                                max_allowed: self.max_struct_lines,
                                suggestion: "Consider splitting into smaller, focused impl blocks"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }

                // Check if file has many unrelated structs (potential SRP violation)
                // Skip collection files which intentionally group related types
                if structs_in_file.len() > 3 && !is_collection_file {
                    let struct_names: Vec<String> =
                        structs_in_file.iter().map(|(n, _)| n.clone()).collect();

                    // Check if structs seem unrelated (different prefixes/suffixes)
                    if !self.structs_seem_related(&struct_names) {
                        violations.push(SolidViolation::MultipleUnrelatedStructs {
                            file: entry.path().to_path_buf(),
                            struct_names,
                            suggestion: "Consider splitting into separate modules".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// OCP: Check for excessive match statements
    pub fn validate_ocp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let match_pattern = Regex::new(r"\bmatch\b").expect("Invalid regex");

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
                    if match_pattern.is_match(line) {
                        // Count match arms
                        let arm_count = self.count_match_arms(&lines, line_num);

                        if arm_count > self.max_match_arms {
                            violations.push(SolidViolation::ExcessiveMatchArms {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                arm_count,
                                max_recommended: self.max_match_arms,
                                suggestion: "Consider using visitor pattern, enum dispatch, or trait objects".to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// ISP: Check for traits with too many methods
    pub fn validate_isp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let fn_pattern =
            Regex::new(r"(?:async\s+)?fn\s+[a-z_][a-z0-9_]*\s*[<(]").expect("Invalid regex");

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
                    if let Some(cap) = trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Count methods in trait
                        let method_count = self.count_trait_methods(&lines, line_num, &fn_pattern);

                        if method_count > self.max_trait_methods {
                            violations.push(SolidViolation::TraitTooLarge {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                method_count,
                                max_allowed: self.max_trait_methods,
                                suggestion: "Consider splitting into smaller, focused traits"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// LSP: Check for partial trait implementations (panic!/todo! in trait methods)
    pub fn validate_lsp(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_for_pattern =
            Regex::new(r"impl(?:<[^>]*>)?\s+([A-Z][a-zA-Z0-9_]*)\s+for\s+([A-Z][a-zA-Z0-9_]*)")
                .expect("Invalid regex");
        let fn_pattern = Regex::new(r"fn\s+([a-z_][a-z0-9_]*)\s*[<(]").expect("Invalid regex");
        let panic_todo_pattern =
            Regex::new(r"(panic!|todo!|unimplemented!)").expect("Invalid regex");

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
                    if let Some(cap) = impl_for_pattern.captures(line) {
                        let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let impl_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                        // Check methods in impl block for panic!/unimplemented macros
                        let mut brace_depth = 0;
                        let mut in_impl = false;
                        let mut current_method: Option<(String, usize)> = None;

                        for (idx, impl_line) in lines[line_num..].iter().enumerate() {
                            if impl_line.contains('{') {
                                in_impl = true;
                            }
                            if in_impl {
                                brace_depth += impl_line.chars().filter(|c| *c == '{').count();
                                brace_depth -= impl_line.chars().filter(|c| *c == '}').count();

                                // Track current method
                                if let Some(fn_cap) = fn_pattern.captures(impl_line) {
                                    let method_name =
                                        fn_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                                    current_method =
                                        Some((method_name.to_string(), line_num + idx + 1));
                                }

                                // Check for panic!/todo! in method body
                                if let Some((ref method_name, method_line)) = current_method {
                                    if panic_todo_pattern.is_match(impl_line) {
                                        violations.push(
                                            SolidViolation::PartialTraitImplementation {
                                                file: entry.path().to_path_buf(),
                                                line: method_line,
                                                impl_name: format!("{}::{}", impl_name, trait_name),
                                                method_name: method_name.clone(),
                                                severity: Severity::Warning,
                                            },
                                        );
                                        current_method = None; // Don't report same method twice
                                    }
                                }

                                if brace_depth == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// SRP: Check for impl blocks with too many methods
    pub fn validate_impl_method_count(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        let impl_pattern =
            Regex::new(r"impl(?:<[^>]*>)?\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let fn_pattern =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+[a-z_][a-z0-9_]*\s*[<(]").expect("Invalid regex");

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
                    // Skip impl Trait for Type (already checked in ISP)
                    if line.contains(" for ") {
                        continue;
                    }

                    if let Some(cap) = impl_pattern.captures(line) {
                        let type_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Count methods in impl block
                        let method_count = self.count_impl_methods(&lines, line_num, &fn_pattern);

                        if method_count > self.max_impl_methods {
                            violations.push(SolidViolation::ImplTooManyMethods {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                type_name: type_name.to_string(),
                                method_count,
                                max_allowed: self.max_impl_methods,
                                suggestion: "Consider splitting into smaller, focused impl blocks or extracting to traits".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// OCP: Check for string-based type dispatch
    pub fn validate_string_dispatch(&self) -> Result<Vec<SolidViolation>> {
        let mut violations = Vec::new();
        // Pattern: match on .as_str() or match with string literals
        let string_match_pattern =
            Regex::new(r#"match\s+\w+\.as_str\(\)|match\s+[&]?\w+\s*\{\s*"[^"]+"\s*=>"#)
                .expect("Invalid regex");
        let string_arm_pattern =
            Regex::new(r#"^\s*"[^"]+"\s*=>"#).expect("Invalid regex");

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

                    // Exit test module when braces close (use < not <= to avoid premature exit)
                    if in_test_module && brace_depth < test_brace_depth {
                        in_test_module = false;
                    }

                    // Skip test modules
                    if in_test_module {
                        continue;
                    }

                    // Check for string-based match dispatch
                    if string_match_pattern.is_match(line) {
                        // Count string arms in the match
                        let string_arm_count = self.count_string_match_arms(&lines, line_num, &string_arm_pattern);

                        if string_arm_count >= 3 {
                            violations.push(SolidViolation::StringBasedDispatch {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                match_expression: trimmed.chars().take(60).collect(),
                                suggestion: "Consider using enum types with FromStr or a registry pattern".to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Count methods in an impl block
    fn count_impl_methods(&self, lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut brace_depth = 0;
        let mut in_impl = false;
        let mut method_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_impl = true;
            }
            if in_impl {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if fn_pattern.is_match(line) {
                    method_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        method_count
    }

    /// Count string match arms
    fn count_string_match_arms(&self, lines: &[&str], start_line: usize, string_arm_pattern: &Regex) -> usize {
        let mut brace_depth = 0;
        let mut in_match = false;
        let mut arm_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_match = true;
            }
            if in_match {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if string_arm_pattern.is_match(line) {
                    arm_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        arm_count
    }

    /// Count lines in a code block (impl, struct, etc.)
    fn count_block_lines(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_block = true;
            }
            if in_block {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();
                count += 1;

                if brace_depth == 0 {
                    break;
                }
            }
        }

        count
    }

    /// Count match arms in a match statement
    fn count_match_arms(&self, lines: &[&str], start_line: usize) -> usize {
        let mut brace_depth = 0;
        let mut in_match = false;
        let mut arm_count = 0;
        let arrow_pattern = Regex::new(r"=>").expect("Invalid regex");

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_match = true;
            }
            if in_match {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                // Count arrows (match arms)
                if arrow_pattern.is_match(line) && brace_depth >= 1 {
                    arm_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        arm_count
    }

    /// Count methods in a trait definition
    fn count_trait_methods(&self, lines: &[&str], start_line: usize, fn_pattern: &Regex) -> usize {
        let mut brace_depth = 0;
        let mut in_trait = false;
        let mut method_count = 0;

        for line in &lines[start_line..] {
            if line.contains('{') {
                in_trait = true;
            }
            if in_trait {
                brace_depth += line.chars().filter(|c| *c == '{').count();
                brace_depth -= line.chars().filter(|c| *c == '}').count();

                if fn_pattern.is_match(line) {
                    method_count += 1;
                }

                if brace_depth == 0 {
                    break;
                }
            }
        }

        method_count
    }

    /// Check if structs seem related (share common prefix/suffix)
    fn structs_seem_related(&self, names: &[String]) -> bool {
        if names.len() < 2 {
            return true;
        }

        // Check for common prefix (at least 3 chars)
        let first = &names[0];
        for len in (3..=first.len().min(10)).rev() {
            let prefix = &first[..len];
            if names.iter().all(|n| n.starts_with(prefix)) {
                return true;
            }
        }

        // Check for common suffix (at least 3 chars)
        for len in (3..=first.len().min(10)).rev() {
            let suffix = &first[first.len().saturating_sub(len)..];
            if names.iter().all(|n| n.ends_with(suffix)) {
                return true;
            }
        }

        // Check for common domain keywords (most structs contain one of these)
        let domain_keywords = [
            "Config",
            "Options",
            "Settings",
            "Error",
            "Result",
            "Builder",
            "Handler",
            "Provider",
            "Service",
            "Health",
            "Crypto",
            "Admin",
            "Http",
            "Args",
            "Request",
            "Response",
            "State",
            "Status",
            "Info",
            "Data",
            "Message",
            "Event",
            "Token",
            "Auth",
            "Cache",
            "Index",
            "Search",
            "Chunk",
            "Embed",
            "Vector",
            "Transport",
            "Operation",
            "Mcp",
            "Protocol",
            "Server",
            "Client",
            "Connection",
            "Session",
            "Route",
            "Endpoint",
        ];

        // Check if structs share related purpose suffixes (Config, State, Error, etc.)
        let purpose_suffixes = [
            "Config", "State", "Error", "Request", "Response", "Options", "Args",
        ];
        let prefix_count: usize = names
            .iter()
            .filter(|n| purpose_suffixes.iter().any(|suffix| n.ends_with(suffix)))
            .count();
        // If majority of structs have purpose suffixes, they're likely related
        if prefix_count > names.len() / 2 {
            return true;
        }
        for keyword in domain_keywords {
            let has_keyword: Vec<_> = names.iter().filter(|n| n.contains(keyword)).collect();
            // If majority (>50%) of structs share a keyword, they're related
            if has_keyword.len() > names.len() / 2 {
                return true;
            }
        }

        // Check for partial word overlaps (e.g., Validation and ValidationResult share "Validation")
        let words: Vec<Vec<&str>> = names
            .iter()
            .map(|n| {
                // Split CamelCase into words
                let mut words = Vec::new();
                let mut start = 0;
                for (i, c) in n.char_indices() {
                    if c.is_uppercase() && i > 0 {
                        if start < i {
                            words.push(&n[start..i]);
                        }
                        start = i;
                    }
                }
                if start < n.len() {
                    words.push(&n[start..]);
                }
                words
            })
            .collect();

        // Count common words across all struct names
        if let Some(first_words) = words.first() {
            for word in first_words {
                if word.len() >= 4 {
                    // Only consider meaningful words (4+ chars)
                    let count = words.iter().filter(|w| w.contains(word)).count();
                    if count > names.len() / 2 {
                        return true;
                    }
                }
            }
        }

        false
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
    fn test_large_trait_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub trait TooManyMethods {
    fn method1(&self);
    fn method2(&self);
    fn method3(&self);
    fn method4(&self);
    fn method5(&self);
    fn method6(&self);
    fn method7(&self);
    fn method8(&self);
    fn method9(&self);
    fn method10(&self);
    fn method11(&self);
    fn method12(&self);
}
"#,
        );

        let validator = SolidValidator::new(temp.path());
        let violations = validator.validate_isp().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            SolidViolation::TraitTooLarge { method_count, .. } => {
                assert!(*method_count > 10);
            }
            _ => panic!("Expected TraitTooLarge"),
        }
    }

    #[test]
    fn test_partial_impl_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub trait MyTrait {
    fn do_something(&self);
}

pub struct MyStruct;

impl MyTrait for MyStruct {
    fn do_something(&self) {
        todo!("not implemented")
    }
}
"#,
        );

        let validator = SolidValidator::new(temp.path());
        let violations = validator.validate_lsp().unwrap();

        assert_eq!(violations.len(), 1);
        match &violations[0] {
            SolidViolation::PartialTraitImplementation { method_name, .. } => {
                assert_eq!(method_name, "do_something");
            }
            _ => panic!("Expected PartialTraitImplementation"),
        }
    }
}
