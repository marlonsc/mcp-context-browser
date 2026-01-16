//! Code Organization Validation
//!
//! Validates code organization:
//! - Constants centralization (magic numbers, duplicate strings)
//! - Type centralization (types should be in domain layer)
//! - File placement (files in correct architectural layers)
//! - Declaration collision detection

use crate::{Result, Severity};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Organization violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationViolation {
    /// Magic number found (should be a named constant)
    MagicNumber {
        file: PathBuf,
        line: usize,
        value: String,
        context: String,
        suggestion: String,
        severity: Severity,
    },

    /// Duplicate string literal across files
    DuplicateStringLiteral {
        value: String,
        occurrences: Vec<(PathBuf, usize)>,
        suggestion: String,
        severity: Severity,
    },

    /// Constant defined in wrong module (should be centralized)
    DecentralizedConstant {
        file: PathBuf,
        line: usize,
        constant_name: String,
        suggestion: String,
        severity: Severity,
    },

    /// Type defined in wrong layer
    TypeInWrongLayer {
        file: PathBuf,
        line: usize,
        type_name: String,
        current_layer: String,
        expected_layer: String,
        severity: Severity,
    },

    /// File in wrong architectural location
    FileInWrongLocation {
        file: PathBuf,
        current_location: String,
        expected_location: String,
        reason: String,
        severity: Severity,
    },

    /// Declaration collision (same name in multiple places)
    DeclarationCollision {
        name: String,
        locations: Vec<(PathBuf, usize, String)>, // (file, line, type)
        severity: Severity,
    },

    /// Trait defined outside ports layer
    TraitOutsidePorts {
        file: PathBuf,
        line: usize,
        trait_name: String,
        severity: Severity,
    },

    /// Provider/Adapter implementation outside infrastructure
    AdapterOutsideInfrastructure {
        file: PathBuf,
        line: usize,
        impl_name: String,
        severity: Severity,
    },
}

impl OrganizationViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::MagicNumber { severity, .. } => *severity,
            Self::DuplicateStringLiteral { severity, .. } => *severity,
            Self::DecentralizedConstant { severity, .. } => *severity,
            Self::TypeInWrongLayer { severity, .. } => *severity,
            Self::FileInWrongLocation { severity, .. } => *severity,
            Self::DeclarationCollision { severity, .. } => *severity,
            Self::TraitOutsidePorts { severity, .. } => *severity,
            Self::AdapterOutsideInfrastructure { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for OrganizationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MagicNumber {
                file,
                line,
                value,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Magic number: {}:{} - {} ({})",
                    file.display(),
                    line,
                    value,
                    suggestion
                )
            }
            Self::DuplicateStringLiteral {
                value,
                occurrences,
                suggestion,
                ..
            } => {
                let locations: Vec<String> = occurrences
                    .iter()
                    .map(|(p, l)| format!("{}:{}", p.display(), l))
                    .collect();
                write!(
                    f,
                    "Duplicate string literal \"{}\": [{}] - {}",
                    value,
                    locations.join(", "),
                    suggestion
                )
            }
            Self::DecentralizedConstant {
                file,
                line,
                constant_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Decentralized constant: {}:{} - {} ({})",
                    file.display(),
                    line,
                    constant_name,
                    suggestion
                )
            }
            Self::TypeInWrongLayer {
                file,
                line,
                type_name,
                current_layer,
                expected_layer,
                ..
            } => {
                write!(
                    f,
                    "Type in wrong layer: {}:{} - {} is in {} but should be in {}",
                    file.display(),
                    line,
                    type_name,
                    current_layer,
                    expected_layer
                )
            }
            Self::FileInWrongLocation {
                file,
                current_location,
                expected_location,
                reason,
                ..
            } => {
                write!(
                    f,
                    "File in wrong location: {} is in {} but should be in {} ({})",
                    file.display(),
                    current_location,
                    expected_location,
                    reason
                )
            }
            Self::DeclarationCollision {
                name, locations, ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, l, t)| format!("{}:{}({})", p.display(), l, t))
                    .collect();
                write!(
                    f,
                    "Declaration collision: {} found at [{}]",
                    name,
                    locs.join(", ")
                )
            }
            Self::TraitOutsidePorts {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Trait outside ports: {}:{} - {} should be in domain/ports",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::AdapterOutsideInfrastructure {
                file,
                line,
                impl_name,
                ..
            } => {
                write!(
                    f,
                    "Adapter outside infrastructure: {}:{} - {} should be in infrastructure/adapters",
                    file.display(),
                    line,
                    impl_name
                )
            }
        }
    }
}

/// Organization validator
pub struct OrganizationValidator {
    workspace_root: PathBuf,
}

impl OrganizationValidator {
    /// Create a new organization validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Run all organization validations
    pub fn validate_all(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_magic_numbers()?);
        violations.extend(self.validate_duplicate_strings()?);
        violations.extend(self.validate_file_placement()?);
        violations.extend(self.validate_trait_placement()?);
        violations.extend(self.validate_declaration_collisions()?);
        Ok(violations)
    }

    /// Check for magic numbers (non-trivial numeric literals)
    pub fn validate_magic_numbers(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Pattern for numeric literals that are suspicious
        let magic_pattern = Regex::new(r"(?<![A-Za-z_])(\d{4,}|[2-9]\d{2,})(?![A-Za-z_\d])").expect("Invalid regex");

        // Allowed patterns (common safe numbers)
        let allowed = ["1000", "1024", "2048", "4096", "8192", "65535", "100", "200", "300", "400", "500"];

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
                // Skip constants.rs files (they're allowed to have numbers)
                let file_name = entry.path().file_name().and_then(|n| n.to_str());
                if file_name.is_some_and(|n| n.contains("constant") || n.contains("config")) {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Skip const/static definitions (they're creating constants)
                    if trimmed.starts_with("const ") || trimmed.starts_with("pub const ")
                        || trimmed.starts_with("static ") || trimmed.starts_with("pub static ") {
                        continue;
                    }

                    // Skip test modules
                    if trimmed.contains("#[cfg(test)]") {
                        continue;
                    }

                    for cap in magic_pattern.captures_iter(line) {
                        let num = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip allowed numbers
                        if allowed.contains(&num) {
                            continue;
                        }

                        // Skip if it looks like a version or date
                        if num.len() == 4 && (num.starts_with("20") || num.starts_with("19")) {
                            continue;
                        }

                        violations.push(OrganizationViolation::MagicNumber {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            value: num.to_string(),
                            context: trimmed.to_string(),
                            suggestion: "Consider using a named constant".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for duplicate string literals that should be constants
    pub fn validate_duplicate_strings(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

        // Pattern for string literals (simplified, catches most cases)
        let string_pattern = Regex::new(r#""([^"\\]{10,})""#).expect("Invalid regex");

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
                    let trimmed = line.trim();

                    // Skip comments and doc strings
                    if trimmed.starts_with("//") || trimmed.starts_with("#[") {
                        continue;
                    }

                    for cap in string_pattern.captures_iter(line) {
                        let string_val = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip common patterns that are OK to repeat
                        if string_val.contains("{}")
                            || string_val.starts_with("test_")
                            || string_val.starts_with("Error")
                            || string_val.contains("://") // URLs
                        {
                            continue;
                        }

                        string_occurrences
                            .entry(string_val.to_string())
                            .or_default()
                            .push((entry.path().to_path_buf(), line_num + 1));
                    }
                }
            }
        }

        // Report strings that appear in multiple files
        for (value, occurrences) in string_occurrences {
            let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
            if unique_files.len() >= 3 {
                violations.push(OrganizationViolation::DuplicateStringLiteral {
                    value,
                    occurrences,
                    suggestion: "Consider creating a named constant".to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Check for files in wrong architectural locations
    pub fn validate_file_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let rel_path = entry.path().strip_prefix(&src_dir).ok();
                let path_str = entry.path().to_string_lossy();
                let file_name = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Check for adapter implementations in domain crate
                if crate_name.contains("domain") && path_str.contains("/adapters/") {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: entry.path().to_path_buf(),
                        current_location: "domain/adapters".to_string(),
                        expected_location: "infrastructure/adapters".to_string(),
                        reason: "Adapters belong in infrastructure layer".to_string(),
                        severity: Severity::Error,
                    });
                }

                // Check for port definitions in infrastructure
                if crate_name.contains("infrastructure") && path_str.contains("/ports/") {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: entry.path().to_path_buf(),
                        current_location: "infrastructure/ports".to_string(),
                        expected_location: "domain/ports".to_string(),
                        reason: "Ports (interfaces) belong in domain layer".to_string(),
                        severity: Severity::Error,
                    });
                }

                // Check for config files outside config directories
                if file_name.contains("config") && !path_str.contains("/config/") && !path_str.contains("/config.rs") {
                    // Allow config.rs at root level
                    if rel_path.is_some_and(|p| p.components().count() > 1) {
                        violations.push(OrganizationViolation::FileInWrongLocation {
                            file: entry.path().to_path_buf(),
                            current_location: "scattered".to_string(),
                            expected_location: "config/ directory".to_string(),
                            reason: "Configuration should be centralized".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }

                // Check for error handling spread across modules
                if file_name == "error.rs" {
                    // Check that it's at the crate root or in a designated error module
                    if rel_path.is_some_and(|p| {
                        let depth = p.components().count();
                        depth > 2 && !path_str.contains("/error/")
                    }) {
                        violations.push(OrganizationViolation::FileInWrongLocation {
                            file: entry.path().to_path_buf(),
                            current_location: "nested error.rs".to_string(),
                            expected_location: "crate root or error/ module".to_string(),
                            reason: "Error types should be centralized".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for traits defined outside domain/ports
    pub fn validate_trait_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let trait_pattern = Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

        // Traits that are OK outside ports (standard patterns)
        let allowed_traits = [
            "Debug", "Clone", "Default", "Display", "Error",
            "From", "Into", "AsRef", "Deref", "Iterator",
            "Send", "Sync", "Sized", "Copy", "Eq", "PartialEq",
            "Ord", "PartialOrd", "Hash", "Serialize", "Deserialize",
        ];

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip domain crate (traits are allowed there)
            if crate_name.contains("domain") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip if in ports directory (re-exports are OK)
                if path_str.contains("/ports/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    if let Some(cap) = trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip allowed traits
                        if allowed_traits.contains(&trait_name) {
                            continue;
                        }

                        // Skip internal/private traits (lowercase or starts with underscore)
                        if trait_name.starts_with('_') {
                            continue;
                        }

                        // Skip traits ending with "Ext" (extension traits are OK locally)
                        if trait_name.ends_with("Ext") {
                            continue;
                        }

                        // Provider/Service traits should be in domain/ports
                        if trait_name.contains("Provider")
                            || trait_name.contains("Service")
                            || trait_name.contains("Repository")
                            || trait_name.ends_with("Interface")
                        {
                            violations.push(OrganizationViolation::TraitOutsidePorts {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for declaration collisions (same name defined in multiple places)
    pub fn validate_declaration_collisions(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut declarations: HashMap<String, Vec<(PathBuf, usize, String)>> = HashMap::new();

        let struct_pattern = Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let enum_pattern = Regex::new(r"(?:pub\s+)?enum\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let trait_pattern = Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

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
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check structs
                    if let Some(cap) = struct_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations
                            .entry(name.to_string())
                            .or_default()
                            .push((entry.path().to_path_buf(), line_num + 1, "struct".to_string()));
                    }

                    // Check enums
                    if let Some(cap) = enum_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations
                            .entry(name.to_string())
                            .or_default()
                            .push((entry.path().to_path_buf(), line_num + 1, "enum".to_string()));
                    }

                    // Check traits
                    if let Some(cap) = trait_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations
                            .entry(name.to_string())
                            .or_default()
                            .push((entry.path().to_path_buf(), line_num + 1, "trait".to_string()));
                    }
                }
            }
        }

        // Report names with multiple declarations
        for (name, locations) in declarations {
            // Check if declarations are in different crates
            let unique_crates: HashSet<_> = locations
                .iter()
                .filter_map(|(path, _, _)| {
                    path.components()
                        .find(|c| c.as_os_str().to_string_lossy().starts_with("mcb-"))
                })
                .collect();

            if unique_crates.len() > 1 {
                // Skip common names that are expected to have multiple declarations
                let common_names = ["Error", "Result", "Config", "Options", "Builder"];
                if common_names.contains(&name.as_str()) {
                    continue;
                }

                violations.push(OrganizationViolation::DeclarationCollision {
                    name,
                    locations,
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
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
    fn test_magic_number_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn process_data() {
    let timeout = 30000;  // magic number
    let buffer_size = 16384;  // magic number
}
"#,
        );

        let validator = OrganizationValidator::new(temp.path());
        let violations = validator.validate_magic_numbers().unwrap();

        assert!(violations.len() >= 1, "Should detect magic numbers");
    }

    #[test]
    fn test_constants_file_exemption() {
        let temp = TempDir::new().unwrap();

        let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("constants.rs"),
            r#"
pub const TIMEOUT_MS: u64 = 30000;
pub const BUFFER_SIZE: usize = 16384;
"#,
        )
        .unwrap();

        fs::write(
            temp.path().join("crates").join("mcb-test").join("Cargo.toml"),
            r#"
[package]
name = "mcb-test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let validator = OrganizationValidator::new(temp.path());
        let violations = validator.validate_magic_numbers().unwrap();

        assert!(violations.is_empty(), "Constants files should be exempt");
    }
}
