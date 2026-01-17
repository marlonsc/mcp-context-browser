//! Refactoring Completeness Validation
//!
//! Validates that refactorings are complete and not left halfway:
//! - Orphan imports (use statements pointing to deleted/moved modules)
//! - Duplicate definitions (same type in multiple locations)
//! - Missing test files for new source files
//! - Stale re-exports (pub use of items that don't exist)
//! - Deleted module references
//! - Dead code from refactoring

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Known migration patterns that are expected during refactoring.
/// These are pairs of crates where duplicates are expected temporarily.
const KNOWN_MIGRATION_PAIRS: &[(&str, &str)] = &[
    ("mcb-providers", "mcb-infrastructure"),
    ("mcb-domain", "mcb-infrastructure"), // Domain types may be mirrored in infrastructure
];

/// Utility types that are intentionally duplicated to avoid cross-crate dependencies.
/// These are common patterns that don't indicate incomplete migration.
const UTILITY_TYPES: &[&str] = &[
    "JsonExt",           // JSON extension trait
    "HttpResponseUtils", // HTTP response helpers
    "CacheStats",        // Cache statistics (implementation-specific)
    "TimedOperation",    // Timing utilities
];

/// Generic type names that are expected to appear in multiple places.
/// These include common patterns like Error/Result as well as layer-specific
/// config types that are intentionally different per CA layer.
const GENERIC_TYPE_NAMES: &[&str] = &[
    "Error",
    "Result",
    "Config",
    "Builder",
    "Context",
    "State",
    "Options",
    "Params",
    "Settings",
    "Message", // Common for actor patterns
    "Request",
    "Response",
    "CacheConfig", // Layer-specific config schemas are valid in CA
];

/// Refactoring completeness violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefactoringViolation {
    /// Import statement referencing non-existent module/item
    OrphanImport {
        file: PathBuf,
        line: usize,
        import_path: String,
        suggestion: String,
        severity: Severity,
    },

    /// Same type name defined in multiple locations (incomplete migration)
    DuplicateDefinition {
        type_name: String,
        locations: Vec<PathBuf>,
        suggestion: String,
        severity: Severity,
    },

    /// New source file without corresponding test file
    MissingTestFile {
        source_file: PathBuf,
        expected_test: PathBuf,
        severity: Severity,
    },

    /// pub use/mod statement for item that doesn't exist
    StaleReExport {
        file: PathBuf,
        line: usize,
        re_export: String,
        severity: Severity,
    },

    /// File/module that was deleted but is still referenced
    DeletedModuleReference {
        referencing_file: PathBuf,
        line: usize,
        deleted_module: String,
        severity: Severity,
    },

    /// Dead code left from refactoring (unused after move)
    RefactoringDeadCode {
        file: PathBuf,
        item_name: String,
        item_type: String,
        severity: Severity,
    },
}

impl RefactoringViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::OrphanImport { severity, .. } => *severity,
            Self::DuplicateDefinition { severity, .. } => *severity,
            Self::MissingTestFile { severity, .. } => *severity,
            Self::StaleReExport { severity, .. } => *severity,
            Self::DeletedModuleReference { severity, .. } => *severity,
            Self::RefactoringDeadCode { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for RefactoringViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrphanImport {
                file,
                line,
                import_path,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Orphan import at {}:{} - '{}' - {}",
                    file.display(),
                    line,
                    import_path,
                    suggestion
                )
            }
            Self::DuplicateDefinition {
                type_name,
                locations,
                suggestion,
                ..
            } => {
                let locs: Vec<String> = locations.iter().map(|p| p.display().to_string()).collect();
                write!(
                    f,
                    "Duplicate definition '{}' in {} locations: [{}] - {}",
                    type_name,
                    locations.len(),
                    locs.join(", "),
                    suggestion
                )
            }
            Self::MissingTestFile {
                source_file,
                expected_test,
                ..
            } => {
                write!(
                    f,
                    "Missing test file for {} - expected {}",
                    source_file.display(),
                    expected_test.display()
                )
            }
            Self::StaleReExport {
                file,
                line,
                re_export,
                ..
            } => {
                write!(
                    f,
                    "Stale re-export at {}:{} - '{}'",
                    file.display(),
                    line,
                    re_export
                )
            }
            Self::DeletedModuleReference {
                referencing_file,
                line,
                deleted_module,
                ..
            } => {
                write!(
                    f,
                    "Reference to deleted module at {}:{} - '{}'",
                    referencing_file.display(),
                    line,
                    deleted_module
                )
            }
            Self::RefactoringDeadCode {
                file,
                item_name,
                item_type,
                ..
            } => {
                write!(
                    f,
                    "Dead {} '{}' from refactoring at {}",
                    item_type,
                    item_name,
                    file.display()
                )
            }
        }
    }
}

impl Violation for RefactoringViolation {
    fn id(&self) -> &str {
        match self {
            Self::OrphanImport { .. } => "REF001",
            Self::DuplicateDefinition { .. } => "REF002",
            Self::MissingTestFile { .. } => "REF003",
            Self::StaleReExport { .. } => "REF004",
            Self::DeletedModuleReference { .. } => "REF005",
            Self::RefactoringDeadCode { .. } => "REF006",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Refactoring
    }

    fn severity(&self) -> Severity {
        match self {
            Self::OrphanImport { severity, .. } => *severity,
            Self::DuplicateDefinition { severity, .. } => *severity,
            Self::MissingTestFile { severity, .. } => *severity,
            Self::StaleReExport { severity, .. } => *severity,
            Self::DeletedModuleReference { severity, .. } => *severity,
            Self::RefactoringDeadCode { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::OrphanImport { file, .. } => Some(file),
            Self::DuplicateDefinition { locations, .. } => locations.first(),
            Self::MissingTestFile { source_file, .. } => Some(source_file),
            Self::StaleReExport { file, .. } => Some(file),
            Self::DeletedModuleReference {
                referencing_file, ..
            } => Some(referencing_file),
            Self::RefactoringDeadCode { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::OrphanImport { line, .. } => Some(*line),
            Self::DuplicateDefinition { .. } => None,
            Self::MissingTestFile { .. } => None,
            Self::StaleReExport { line, .. } => Some(*line),
            Self::DeletedModuleReference { line, .. } => Some(*line),
            Self::RefactoringDeadCode { .. } => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::OrphanImport { suggestion, .. } => Some(suggestion.clone()),
            Self::DuplicateDefinition { suggestion, .. } => Some(suggestion.clone()),
            Self::MissingTestFile {
                source_file,
                expected_test,
                ..
            } => Some(format!(
                "Create test file {} for source {}",
                expected_test.display(),
                source_file.display()
            )),
            Self::StaleReExport { re_export, .. } => {
                Some(format!("Remove or update the re-export '{}'", re_export))
            }
            Self::DeletedModuleReference { deleted_module, .. } => Some(format!(
                "Remove the mod declaration for '{}' or create the module file",
                deleted_module
            )),
            Self::RefactoringDeadCode {
                item_name,
                item_type,
                ..
            } => Some(format!(
                "Remove the unused {} '{}' or add #[allow(dead_code)] if intentional",
                item_type, item_name
            )),
        }
    }
}

/// Refactoring completeness validator
pub struct RefactoringValidator {
    config: ValidationConfig,
}

impl RefactoringValidator {
    /// Create a new refactoring validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all refactoring validations
    pub fn validate_all(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_duplicate_definitions()?);
        violations.extend(self.validate_missing_test_files()?);
        violations.extend(self.validate_mod_declarations()?);
        Ok(violations)
    }

    /// Check for same type defined in multiple locations
    pub fn validate_duplicate_definitions(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        let definition_pattern =
            Regex::new(r"(?:pub\s+)?(?:struct|trait|enum)\s+([A-Z][a-zA-Z0-9_]*)(?:\s*<|\s*\{|\s*;|\s*\(|\s+where)")
                .expect("Invalid regex");

        // Map: type_name -> Vec<file_path>
        let mut definitions: HashMap<String, Vec<PathBuf>> = HashMap::new();

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
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files and archived directories
                if path_str.contains("/tests/")
                    || path_str.contains("_test.rs")
                    || path_str.contains(".archived")
                    || path_str.contains(".bak")
                {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;

                for cap in definition_pattern.captures_iter(&content) {
                    let type_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                    // Skip generic names that are expected to appear in multiple places
                    if GENERIC_TYPE_NAMES.contains(&type_name) {
                        continue;
                    }

                    definitions
                        .entry(type_name.to_string())
                        .or_default()
                        .push(path.to_path_buf());
                }
            }
        }

        // Find duplicates (same name in different files)
        for (type_name, locations) in definitions {
            if locations.len() > 1 {
                // Check if duplicates are in different crates
                let crates: HashSet<String> = locations
                    .iter()
                    .filter_map(|p| {
                        p.to_string_lossy()
                            .split("/crates/")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .map(|s| s.to_string())
                    })
                    .collect();

                if crates.len() > 1 {
                    // Cross-crate duplicate - categorize by pattern
                    let severity = self.categorize_duplicate_severity(&type_name, &crates);

                    let suggestion = match severity {
                        Severity::Info => format!(
                            "Type '{}' exists in {:?}. This is a known migration pattern - consolidate when migration completes.",
                            type_name, crates
                        ),
                        Severity::Warning => format!(
                            "Type '{}' is defined in {:?}. Consider consolidating to one location.",
                            type_name, crates
                        ),
                        Severity::Error => format!(
                            "Type '{}' is unexpectedly defined in multiple crates: {:?}. This requires immediate consolidation.",
                            type_name, crates
                        ),
                    };

                    violations.push(RefactoringViolation::DuplicateDefinition {
                        type_name: type_name.clone(),
                        locations: locations.clone(),
                        suggestion,
                        severity,
                    });
                } else if locations.len() >= 2 {
                    // Same crate but duplicates - check if in different directories
                    let dirs: HashSet<String> = locations
                        .iter()
                        .filter_map(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
                        .collect();

                    // Only flag if duplicates are in different directories (not just mod.rs + impl.rs)
                    if dirs.len() >= 2 {
                        violations.push(RefactoringViolation::DuplicateDefinition {
                            type_name: type_name.clone(),
                            locations: locations.clone(),
                            suggestion: format!(
                                "Type '{}' is defined {} times in different directories within the same crate. Consolidate to single location.",
                                type_name,
                                locations.len()
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Categorize duplicate severity based on known patterns
    fn categorize_duplicate_severity(&self, type_name: &str, crates: &HashSet<String>) -> Severity {
        // Check if this is an intentionally duplicated utility type
        if UTILITY_TYPES.contains(&type_name) {
            return Severity::Info;
        }

        // Check if the crates match a known migration pattern
        let crate_vec: Vec<&String> = crates.iter().collect();
        if crate_vec.len() == 2 {
            for (crate_a, crate_b) in KNOWN_MIGRATION_PAIRS {
                if (crate_vec[0].as_str() == *crate_a && crate_vec[1].as_str() == *crate_b)
                    || (crate_vec[0].as_str() == *crate_b && crate_vec[1].as_str() == *crate_a)
                {
                    // This is a known migration pair - downgrade to Info
                    return Severity::Info;
                }
            }
        }

        // Check for patterns that suggest migration in progress
        // Types ending with Provider, Processor, etc. between known pairs
        let migration_type_patterns = [
            "Provider",
            "Processor",
            "Handler",
            "Service",
            "Repository",
            "Adapter",
            "Factory",
            "Publisher",
            "Subscriber",
        ];

        if migration_type_patterns
            .iter()
            .any(|p| type_name.ends_with(p))
        {
            // Check if any known migration pair is involved
            for (crate_a, crate_b) in KNOWN_MIGRATION_PAIRS {
                if crates.contains(*crate_a) || crates.contains(*crate_b) {
                    return Severity::Warning; // Migration-related, but should be tracked
                }
            }
        }

        // Unknown cross-crate duplicate - Error
        Severity::Error
    }

    /// Check for source files without corresponding test files
    pub fn validate_missing_test_files(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();

        // Files that don't need dedicated tests (re-exports, utilities, infrastructure)
        const SKIP_FILES: &[&str] = &[
            // Standard files
            "mod",
            "lib",
            "main",
            "prelude",
            "constants",
            "types",
            "error",
            "errors",
            "helpers",
            "utils",
            "common",
            "config",
            "builder",
            "factory",
            // Domain service interfaces (tested via integration)
            "indexing",
            "search_repository",
            // Server infrastructure (tested via e2e/integration tests)
            "metrics",
            "components",
            "operations",
            "rate_limit_middleware",
            "security",
            "mcp_server",
            "init",
        ];

        // Directory patterns that are tested via integration tests
        // These directories have tests in tests/{dir_name}/ subdirectories
        const SKIP_DIR_PATTERNS: &[&str] = &[
            "providers",
            "adapters",
            "language",
            "embedding",
            "vector_store",
            "cache",
            "hybrid_search",
            "events",
            "chunking",
            "http",
            "di",
            "admin",    // Admin handlers have tests in tests/admin/
            "handlers", // Handlers have tests in tests/handlers/
            "config",   // Config modules have tests in tests/config/
            "tools",    // Tools have tests in tests/tools/
            "utils",    // Utilities have tests in tests/utils/
            "ports",    // Port traits have tests in tests/ports/
        ];

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            let tests_dir = crate_dir.join("tests");

            if !src_dir.exists() {
                continue;
            }

            // If tests directory doesn't exist, skip this crate (no test infrastructure)
            if !tests_dir.exists() {
                continue;
            }

            // Collect existing test files and directories
            let mut test_files: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut test_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();

            for entry in WalkDir::new(&tests_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        test_files.insert(stem.to_string());
                        // Also normalize _test and _tests suffixes
                        if let Some(base) = stem.strip_suffix("_test") {
                            test_files.insert(base.to_string());
                        }
                        if let Some(base) = stem.strip_suffix("_tests") {
                            test_files.insert(base.to_string());
                        }
                    }
                } else if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        test_dirs.insert(name.to_string());
                    }
                }
            }

            // Check each source file
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip common files that don't need dedicated tests
                if SKIP_FILES.contains(&file_name) {
                    continue;
                }

                // Get relative path for directory checks
                let relative = path.strip_prefix(&src_dir).unwrap_or(path);
                let path_str = relative.to_string_lossy();

                // Skip files in directories that are tested via integration tests
                let in_skip_dir = SKIP_DIR_PATTERNS
                    .iter()
                    .any(|pattern| path_str.contains(pattern));
                if in_skip_dir {
                    continue;
                }

                // Check if this file or its parent module has a test
                let has_test = test_files.contains(file_name)
                    || test_files.contains(&format!("{}_test", file_name))
                    || test_files.contains(&format!("{}_tests", file_name));

                // For files in subdirectories, also check parent directory coverage
                let parent_covered = if relative.components().count() > 1 {
                    let parent_name = relative
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    test_files.contains(parent_name)
                        || test_dirs.contains(parent_name)
                        || test_files.contains(&format!("{}_test", parent_name))
                        || test_files.contains(&format!("{}_tests", parent_name))
                } else {
                    false
                };

                if !has_test && !parent_covered {
                    violations.push(RefactoringViolation::MissingTestFile {
                        source_file: path.to_path_buf(),
                        expected_test: tests_dir.join(format!("{}_test.rs", file_name)),
                        severity: Severity::Warning, // Warning, not Error - tests are quality, not critical
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Check for mod declarations that reference non-existent files
    pub fn validate_mod_declarations(&self) -> Result<Vec<RefactoringViolation>> {
        let mut violations = Vec::new();
        let mod_pattern =
            Regex::new(r"(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)(?:\s*;)").expect("Invalid regex");

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
                let path = entry.path();
                let parent_dir = path.parent().unwrap_or(Path::new("."));
                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = mod_pattern.captures(line) {
                        let mod_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Check if module file exists
                        let mod_file = parent_dir.join(format!("{}.rs", mod_name));
                        let mod_dir = parent_dir.join(mod_name).join("mod.rs");

                        if !mod_file.exists() && !mod_dir.exists() {
                            violations.push(RefactoringViolation::DeletedModuleReference {
                                referencing_file: path.to_path_buf(),
                                line: line_num + 1,
                                deleted_module: mod_name.to_string(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Get all crate directories
    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();
        let crates_dir = self.config.workspace_root.join("crates");

        if crates_dir.exists() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip mcb-validate
                if path
                    .file_name()
                    .is_some_and(|n| n == "mcb-validate" || n == "mcb")
                {
                    continue;
                }

                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }

        Ok(dirs)
    }
}

impl crate::validator_trait::Validator for RefactoringValidator {
    fn name(&self) -> &'static str {
        "refactoring"
    }

    fn description(&self) -> &'static str {
        "Validates refactoring completeness (duplicate definitions, missing tests, stale references)"
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

        // Create tests directory
        let tests_dir = temp.path().join("crates").join(name).join("tests");
        fs::create_dir_all(&tests_dir).unwrap();
    }

    #[test]
    fn test_duplicate_definition_detection() {
        let temp = TempDir::new().unwrap();

        // Create first crate with MyService
        create_test_crate(
            &temp,
            "mcb-domain",
            r#"
pub struct MyService {
    pub name: String,
}
"#,
        );

        // Create second crate with same MyService
        create_test_crate(
            &temp,
            "mcb-server",
            r#"
pub struct MyService {
    pub id: u64,
}
"#,
        );

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_duplicate_definitions().unwrap();

        assert!(!violations.is_empty(), "Should detect duplicate MyService");
    }

    #[test]
    fn test_missing_module_reference() {
        let temp = TempDir::new().unwrap();

        // Create crate with reference to non-existent module
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub mod existing;
pub mod deleted_module;  // This module doesn't exist
"#,
        );

        // Create existing.rs
        let src_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::write(src_dir.join("existing.rs"), "// exists").unwrap();

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_mod_declarations().unwrap();

        assert_eq!(violations.len(), 1, "Should detect missing deleted_module");
        match &violations[0] {
            RefactoringViolation::DeletedModuleReference { deleted_module, .. } => {
                assert_eq!(deleted_module, "deleted_module");
            }
            _ => panic!("Expected DeletedModuleReference violation"),
        }
    }

    #[test]
    fn test_no_false_positives_for_inline_mods() {
        let temp = TempDir::new().unwrap();

        // Create crate with inline module (not a reference to file)
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub mod inline {
    pub fn hello() {}
}
"#,
        );

        let validator = RefactoringValidator::new(temp.path());
        let violations = validator.validate_mod_declarations().unwrap();

        assert!(
            violations.is_empty(),
            "Inline modules should not trigger violations"
        );
    }
}
