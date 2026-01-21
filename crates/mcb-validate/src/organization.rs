//! Code Organization Validation
//!
//! Validates code organization:
//! - Constants centralization (magic numbers, duplicate strings)
//! - Type centralization (types should be in domain layer)
//! - File placement (files in correct architectural layers)
//! - Declaration collision detection

use crate::scan::{for_each_crate_rs_path, for_each_scan_rs_path, is_test_path};
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{ComponentType, Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

    /// Constants file too large (should be split by domain)
    ConstantsFileTooLarge {
        file: PathBuf,
        line_count: usize,
        max_allowed: usize,
        severity: Severity,
    },

    /// Common magic number pattern detected (vector dimensions, timeouts, pool sizes)
    CommonMagicNumber {
        file: PathBuf,
        line: usize,
        value: String,
        pattern_type: String,
        suggestion: String,
        severity: Severity,
    },

    /// File too large without module decomposition
    LargeFileWithoutModules {
        file: PathBuf,
        line_count: usize,
        max_allowed: usize,
        suggestion: String,
        severity: Severity,
    },

    /// Same service/type defined in multiple layers
    DualLayerDefinition {
        type_name: String,
        locations: Vec<(PathBuf, String)>, // (file, layer)
        severity: Severity,
    },

    /// Server layer creating application services directly
    ServerCreatingServices {
        file: PathBuf,
        line: usize,
        service_name: String,
        suggestion: String,
        severity: Severity,
    },

    /// Application layer importing from server
    ApplicationImportsServer {
        file: PathBuf,
        line: usize,
        import_statement: String,
        severity: Severity,
    },

    /// Strict directory violation - component in wrong directory for its type
    StrictDirectoryViolation {
        file: PathBuf,
        component_type: ComponentType,
        current_directory: String,
        expected_directory: String,
        severity: Severity,
    },

    /// Domain layer contains implementation (should be trait-only)
    DomainLayerImplementation {
        file: PathBuf,
        line: usize,
        impl_type: String,
        type_name: String,
        severity: Severity,
    },

    /// Handler file outside handlers directory
    HandlerOutsideHandlers {
        file: PathBuf,
        line: usize,
        handler_name: String,
        severity: Severity,
    },

    /// Port trait outside ports directory
    PortOutsidePorts {
        file: PathBuf,
        line: usize,
        trait_name: String,
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
            Self::ConstantsFileTooLarge { severity, .. } => *severity,
            Self::CommonMagicNumber { severity, .. } => *severity,
            Self::LargeFileWithoutModules { severity, .. } => *severity,
            Self::DualLayerDefinition { severity, .. } => *severity,
            Self::ServerCreatingServices { severity, .. } => *severity,
            Self::ApplicationImportsServer { severity, .. } => *severity,
            Self::StrictDirectoryViolation { severity, .. } => *severity,
            Self::DomainLayerImplementation { severity, .. } => *severity,
            Self::HandlerOutsideHandlers { severity, .. } => *severity,
            Self::PortOutsidePorts { severity, .. } => *severity,
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
            Self::ConstantsFileTooLarge {
                file,
                line_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "Constants file too large: {} has {} lines (max: {}) - consider splitting by domain",
                    file.display(),
                    line_count,
                    max_allowed
                )
            }
            Self::CommonMagicNumber {
                file,
                line,
                value,
                pattern_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Common magic number: {}:{} - {} ({}) - {}",
                    file.display(),
                    line,
                    value,
                    pattern_type,
                    suggestion
                )
            }
            Self::LargeFileWithoutModules {
                file,
                line_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Large file without modules: {} has {} lines (max: {}) - {}",
                    file.display(),
                    line_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::DualLayerDefinition {
                type_name,
                locations,
                ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, layer)| format!("{}({})", p.display(), layer))
                    .collect();
                write!(
                    f,
                    "CA: Dual layer definition for {}: [{}]",
                    type_name,
                    locs.join(", ")
                )
            }
            Self::ServerCreatingServices {
                file,
                line,
                service_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "CA: Server creating service: {}:{} - {} ({})",
                    file.display(),
                    line,
                    service_name,
                    suggestion
                )
            }
            Self::ApplicationImportsServer {
                file,
                line,
                import_statement,
                ..
            } => {
                write!(
                    f,
                    "CA: Application imports server: {}:{} - {}",
                    file.display(),
                    line,
                    import_statement
                )
            }
            Self::StrictDirectoryViolation {
                file,
                component_type,
                current_directory,
                expected_directory,
                ..
            } => {
                write!(
                    f,
                    "CA: {} in wrong directory: {} is in '{}' but should be in '{}'",
                    component_type,
                    file.display(),
                    current_directory,
                    expected_directory
                )
            }
            Self::DomainLayerImplementation {
                file,
                line,
                impl_type,
                type_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Domain layer has {} for {}: {}:{} (domain should be trait-only)",
                    impl_type,
                    type_name,
                    file.display(),
                    line
                )
            }
            Self::HandlerOutsideHandlers {
                file,
                line,
                handler_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Handler {} outside handlers directory: {}:{}",
                    handler_name,
                    file.display(),
                    line
                )
            }
            Self::PortOutsidePorts {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Port trait {} outside ports directory: {}:{}",
                    trait_name,
                    file.display(),
                    line
                )
            }
        }
    }
}

impl Violation for OrganizationViolation {
    fn id(&self) -> &str {
        match self {
            Self::MagicNumber { .. } => "ORG001",
            Self::DuplicateStringLiteral { .. } => "ORG002",
            Self::DecentralizedConstant { .. } => "ORG003",
            Self::TypeInWrongLayer { .. } => "ORG004",
            Self::FileInWrongLocation { .. } => "ORG005",
            Self::DeclarationCollision { .. } => "ORG006",
            Self::TraitOutsidePorts { .. } => "ORG007",
            Self::AdapterOutsideInfrastructure { .. } => "ORG008",
            Self::ConstantsFileTooLarge { .. } => "ORG009",
            Self::CommonMagicNumber { .. } => "ORG010",
            Self::LargeFileWithoutModules { .. } => "ORG011",
            Self::DualLayerDefinition { .. } => "ORG012",
            Self::ServerCreatingServices { .. } => "ORG013",
            Self::ApplicationImportsServer { .. } => "ORG014",
            Self::StrictDirectoryViolation { .. } => "ORG015",
            Self::DomainLayerImplementation { .. } => "ORG016",
            Self::HandlerOutsideHandlers { .. } => "ORG017",
            Self::PortOutsidePorts { .. } => "ORG018",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Organization
    }

    fn severity(&self) -> Severity {
        match self {
            Self::MagicNumber { severity, .. } => *severity,
            Self::DuplicateStringLiteral { severity, .. } => *severity,
            Self::DecentralizedConstant { severity, .. } => *severity,
            Self::TypeInWrongLayer { severity, .. } => *severity,
            Self::FileInWrongLocation { severity, .. } => *severity,
            Self::DeclarationCollision { severity, .. } => *severity,
            Self::TraitOutsidePorts { severity, .. } => *severity,
            Self::AdapterOutsideInfrastructure { severity, .. } => *severity,
            Self::ConstantsFileTooLarge { severity, .. } => *severity,
            Self::CommonMagicNumber { severity, .. } => *severity,
            Self::LargeFileWithoutModules { severity, .. } => *severity,
            Self::DualLayerDefinition { severity, .. } => *severity,
            Self::ServerCreatingServices { severity, .. } => *severity,
            Self::ApplicationImportsServer { severity, .. } => *severity,
            Self::StrictDirectoryViolation { severity, .. } => *severity,
            Self::DomainLayerImplementation { severity, .. } => *severity,
            Self::HandlerOutsideHandlers { severity, .. } => *severity,
            Self::PortOutsidePorts { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::MagicNumber { file, .. } => Some(file),
            Self::DuplicateStringLiteral { .. } => None, // Multiple files
            Self::DecentralizedConstant { file, .. } => Some(file),
            Self::TypeInWrongLayer { file, .. } => Some(file),
            Self::FileInWrongLocation { file, .. } => Some(file),
            Self::DeclarationCollision { .. } => None, // Multiple files
            Self::TraitOutsidePorts { file, .. } => Some(file),
            Self::AdapterOutsideInfrastructure { file, .. } => Some(file),
            Self::ConstantsFileTooLarge { file, .. } => Some(file),
            Self::CommonMagicNumber { file, .. } => Some(file),
            Self::LargeFileWithoutModules { file, .. } => Some(file),
            Self::DualLayerDefinition { .. } => None, // Multiple files
            Self::ServerCreatingServices { file, .. } => Some(file),
            Self::ApplicationImportsServer { file, .. } => Some(file),
            Self::StrictDirectoryViolation { file, .. } => Some(file),
            Self::DomainLayerImplementation { file, .. } => Some(file),
            Self::HandlerOutsideHandlers { file, .. } => Some(file),
            Self::PortOutsidePorts { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::MagicNumber { line, .. } => Some(*line),
            Self::DuplicateStringLiteral { .. } => None, // Multiple lines
            Self::DecentralizedConstant { line, .. } => Some(*line),
            Self::TypeInWrongLayer { line, .. } => Some(*line),
            Self::FileInWrongLocation { .. } => None, // File-level issue
            Self::DeclarationCollision { .. } => None, // Multiple lines
            Self::TraitOutsidePorts { line, .. } => Some(*line),
            Self::AdapterOutsideInfrastructure { line, .. } => Some(*line),
            Self::ConstantsFileTooLarge { .. } => None, // File-level issue
            Self::CommonMagicNumber { line, .. } => Some(*line),
            Self::LargeFileWithoutModules { .. } => None, // File-level issue
            Self::DualLayerDefinition { .. } => None,     // Multiple files
            Self::ServerCreatingServices { line, .. } => Some(*line),
            Self::ApplicationImportsServer { line, .. } => Some(*line),
            Self::StrictDirectoryViolation { .. } => None, // File-level issue
            Self::DomainLayerImplementation { line, .. } => Some(*line),
            Self::HandlerOutsideHandlers { line, .. } => Some(*line),
            Self::PortOutsidePorts { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::MagicNumber { suggestion, .. } => Some(suggestion.clone()),
            Self::DuplicateStringLiteral { suggestion, .. } => Some(suggestion.clone()),
            Self::DecentralizedConstant { suggestion, .. } => Some(suggestion.clone()),
            Self::TypeInWrongLayer { expected_layer, .. } => {
                Some(format!("Move type to {} layer", expected_layer))
            }
            Self::FileInWrongLocation {
                expected_location, ..
            } => Some(format!("Move file to {}", expected_location)),
            Self::DeclarationCollision { .. } => {
                Some("Consolidate declarations or use different names".to_string())
            }
            Self::TraitOutsidePorts { .. } => Some("Move trait to domain/ports".to_string()),
            Self::AdapterOutsideInfrastructure { .. } => {
                Some("Move adapter to infrastructure/adapters".to_string())
            }
            Self::ConstantsFileTooLarge { .. } => {
                Some("Split constants file by domain".to_string())
            }
            Self::CommonMagicNumber { suggestion, .. } => Some(suggestion.clone()),
            Self::LargeFileWithoutModules { suggestion, .. } => Some(suggestion.clone()),
            Self::DualLayerDefinition { .. } => {
                Some("Keep definition in one layer only".to_string())
            }
            Self::ServerCreatingServices { suggestion, .. } => Some(suggestion.clone()),
            Self::ApplicationImportsServer { .. } => {
                Some("Remove server import from application layer".to_string())
            }
            Self::StrictDirectoryViolation {
                expected_directory, ..
            } => Some(format!("Move to {}", expected_directory)),
            Self::DomainLayerImplementation { .. } => {
                Some("Move implementation to application or infrastructure layer".to_string())
            }
            Self::HandlerOutsideHandlers { .. } => {
                Some("Move handler to server/handlers".to_string())
            }
            Self::PortOutsidePorts { .. } => Some("Move port trait to domain/ports".to_string()),
        }
    }
}

/// Organization validator
pub struct OrganizationValidator {
    config: ValidationConfig,
}

impl OrganizationValidator {
    /// Create a new organization validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all organization validations
    pub fn validate_all(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_magic_numbers()?);
        violations.extend(self.validate_duplicate_strings()?);
        violations.extend(self.validate_file_placement()?);
        violations.extend(self.validate_trait_placement()?);
        // NOTE: validate_declaration_collisions() removed - RefactoringValidator handles
        // duplicate definitions with better categorization (known migration pairs, severity)
        violations.extend(self.validate_layer_violations()?);
        // Strict CA directory and layer compliance
        violations.extend(self.validate_strict_directory()?);
        violations.extend(self.validate_domain_traits_only()?);
        Ok(violations)
    }

    /// Check for magic numbers (non-trivial numeric literals)
    pub fn validate_magic_numbers(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Pattern for numeric literals: 5+ digits (skip 4-digit numbers to reduce noise)
        let magic_pattern = Regex::new(r"\b(\d{5,})\b").expect("Invalid regex");

        // Allowed patterns (common safe numbers, powers of 2, well-known values, etc.)
        let allowed = [
            // Powers of 2
            "16384",
            "32768",
            "65535",
            "65536",
            "131072",
            "262144",
            "524288",
            "1048576",
            "2097152",
            "4194304",
            // Common memory sizes (in bytes)
            "100000",
            "1000000",
            "10000000",
            "100000000",
            // Time values (seconds)
            "86400",
            "604800",
            "2592000",
            "31536000",
            // Large round numbers (often limits)
            "100000",
            "1000000",
        ];

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            // Skip constants.rs files (they're allowed to have numbers)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| n.contains("constant") || n.contains("config")) {
                return Ok(());
            }

            // Skip test files
            let path_str = path.to_string_lossy();
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions (they're creating constants)
                if trimmed.starts_with("const ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("pub static ")
                {
                    continue;
                }

                // Skip attribute macros (derive, cfg, etc.)
                if trimmed.starts_with("#[") {
                    continue;
                }

                // Skip doc comments
                if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                    continue;
                }

                // Skip assert macros (often use expected values)
                if trimmed.contains("assert") {
                    continue;
                }

                for cap in magic_pattern.captures_iter(line) {
                    let num = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed numbers
                    if allowed.contains(&num) {
                        continue;
                    }

                    // Skip numbers that are clearly part of a constant reference
                    // e.g., _1024, SIZE_16384
                    if line.contains(&format!("_{}", num)) || line.contains(&format!("{}_", num)) {
                        continue;
                    }

                    // Skip underscored numbers (100_000) - they're usually constants
                    if line.contains(&format!(
                        "{}_{}",
                        &num[..num.len().min(3)],
                        &num[num.len().min(3)..]
                    )) {
                        continue;
                    }

                    violations.push(OrganizationViolation::MagicNumber {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        value: num.to_string(),
                        context: trimmed.to_string(),
                        suggestion: "Consider using a named constant".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Check for duplicate string literals that should be constants
    pub fn validate_duplicate_strings(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

        // Pattern for string literals (15+ chars to reduce noise)
        let string_pattern = Regex::new(r#""([^"\\]{15,})""#).expect("Invalid regex");

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            // Skip constants files (they define string constants)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| n.contains("constant")) {
                return Ok(());
            }

            // Skip test files
            let path_str = path.to_string_lossy();
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments and doc strings
                if trimmed.starts_with("//") || trimmed.starts_with("#[") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions
                if trimmed.starts_with("const ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("pub static ")
                {
                    continue;
                }

                for cap in string_pattern.captures_iter(line) {
                    let string_val = cap.get(1).map_or("", |m| m.as_str());

                    // Skip common patterns that are OK to repeat
                    if string_val.contains("{}")           // Format strings
                        || string_val.starts_with("test_")  // Test names
                        || string_val.starts_with("Error")  // Error messages
                        || string_val.starts_with("error")
                        || string_val.starts_with("Failed")
                        || string_val.starts_with("Invalid")
                        || string_val.starts_with("Cannot")
                        || string_val.starts_with("Unable")
                        || string_val.starts_with("Missing")
                        || string_val.contains("://")       // URLs
                        || string_val.contains(".rs")       // File paths
                        || string_val.contains(".json")
                        || string_val.contains(".toml")
                        || string_val.ends_with("_id")      // ID fields
                        || string_val.ends_with("_key")     // Key fields
                        || string_val.starts_with("pub ")   // Code patterns
                        || string_val.starts_with("fn ")
                        || string_val.starts_with("let ")
                        || string_val.starts_with("CARGO_") // env!() macros
                        || string_val.contains("serde_json")// Code patterns
                        || string_val.contains(".to_string()")
                    // Method chains
                    {
                        continue;
                    }

                    string_occurrences
                        .entry(string_val.to_string())
                        .or_default()
                        .push((path.to_path_buf(), line_num + 1));
                }
            }

            Ok(())
        })?;

        // Report strings that appear in 4+ files (higher threshold)
        for (value, occurrences) in string_occurrences {
            let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
            if unique_files.len() >= 4 {
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

        for_each_crate_rs_path(&self.config, |path, src_dir, crate_name| {
            let rel_path = path.strip_prefix(src_dir).ok();
            let path_str = path.to_string_lossy();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check for adapter implementations in domain crate
            if crate_name.contains("domain") && path_str.contains("/adapters/") {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "domain/adapters".to_string(),
                    expected_location: "infrastructure/adapters".to_string(),
                    reason: "Adapters belong in infrastructure layer".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for port definitions in infrastructure
            if crate_name.contains("infrastructure") && path_str.contains("/ports/") {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "infrastructure/ports".to_string(),
                    expected_location: "domain/ports".to_string(),
                    reason: "Ports (interfaces) belong in domain layer".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for config files outside config directories
            // Exclude handler files (e.g., config_handlers.rs) - these are HTTP handlers, not config files
            if file_name.contains("config")
                && !file_name.contains("handler")
                && !path_str.contains("/config/")
                && !path_str.contains("/config.rs")
                && !path_str.contains("/admin/")
            // Admin config handlers are valid
            {
                // Allow config.rs at root level
                if rel_path.is_some_and(|p| p.components().count() > 1) {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: path.to_path_buf(),
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
                        file: path.to_path_buf(),
                        current_location: "nested error.rs".to_string(),
                        expected_location: "crate root or error/ module".to_string(),
                        reason: "Error types should be centralized".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Check for traits defined outside domain/ports
    pub fn validate_trait_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

        // Traits that are OK outside ports (standard patterns)
        let allowed_traits = [
            "Debug",
            "Clone",
            "Default",
            "Display",
            "Error",
            "From",
            "Into",
            "AsRef",
            "Deref",
            "Iterator",
            "Send",
            "Sync",
            "Sized",
            "Copy",
            "Eq",
            "PartialEq",
            "Ord",
            "PartialOrd",
            "Hash",
            "Serialize",
            "Deserialize",
        ];

        // Trait suffixes that are OK in infrastructure (implementation details)
        let allowed_suffixes = [
            "Ext",       // Extension traits
            "Factory",   // Factory patterns
            "Builder",   // Builder patterns
            "Helper",    // Helper traits
            "Internal",  // Internal traits
            "Impl",      // Implementation traits
            "Adapter",   // Adapter-specific traits
            "Handler",   // Handler traits (event handlers, etc.)
            "Listener",  // Event listeners
            "Callback",  // Callback traits
            "Module",    // DI module traits
            "Component", // DI component traits
        ];

        for_each_crate_rs_path(&self.config, |path, _src_dir, crate_name| {
            // Skip domain crate (traits are allowed there)
            if crate_name.contains("domain") {
                return Ok(());
            }

            let path_str = path.to_string_lossy();

            // Skip if in ports directory (re-exports are OK)
            if path_str.contains("/ports/") {
                return Ok(());
            }

            // Skip DI modules (they often define internal traits)
            if path_str.contains("/di/") {
                return Ok(());
            }

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip traits in test modules
                if in_test_module {
                    continue;
                }

                if let Some(cap) = trait_pattern.captures(line) {
                    let trait_name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed traits
                    if allowed_traits.contains(&trait_name) {
                        continue;
                    }

                    // Skip internal/private traits (starts with underscore)
                    if trait_name.starts_with('_') {
                        continue;
                    }

                    // Skip traits with allowed suffixes
                    if allowed_suffixes
                        .iter()
                        .any(|suffix| trait_name.ends_with(suffix))
                    {
                        continue;
                    }

                    // Skip traits that are clearly internal (private trait declarations)
                    if trimmed.starts_with("trait ") && !trimmed.starts_with("pub trait ") {
                        continue;
                    }

                    // Infrastructure-specific provider traits that are OK outside ports
                    // These are implementation details, not domain contracts
                    let infra_provider_patterns = [
                        "CacheProvider",      // Caching is infrastructure
                        "HttpClientProvider", // HTTP client is infrastructure
                        "ConfigProvider",     // Config loading is infrastructure
                        "LogProvider",        // Logging is infrastructure
                        "MetricsProvider",    // Metrics is infrastructure
                        "TracingProvider",    // Tracing is infrastructure
                        "StorageProvider",    // Low-level storage is infra
                    ];

                    // Skip infrastructure-specific providers
                    if infra_provider_patterns.contains(&trait_name) {
                        continue;
                    }

                    // Only flag Provider/Service/Repository traits that look like ports
                    if trait_name.contains("Provider")
                        || trait_name.contains("Service")
                        || trait_name.contains("Repository")
                        || trait_name.ends_with("Interface")
                    {
                        violations.push(OrganizationViolation::TraitOutsidePorts {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            trait_name: trait_name.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Check for declaration collisions (same name defined in multiple places)
    /// Validate Clean Architecture layer violations
    pub fn validate_layer_violations(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Patterns for detecting layer violations
        let arc_new_service_pattern =
            Regex::new(r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new")
                .expect("Invalid regex");
        let server_import_pattern =
            Regex::new(r"use\s+(?:crate::|super::)*server::").expect("Invalid regex");

        for_each_scan_rs_path(&self.config, true, |path, _src_dir| {
            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Determine current layer
            let is_server_layer = path_str.contains("/server/");
            let is_application_layer = path_str.contains("/application/");
            let is_infrastructure_layer = path_str.contains("/infrastructure/");

            let content = std::fs::read_to_string(path)?;
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

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Check: Server layer creating services directly
                if is_server_layer {
                    if let Some(cap) = arc_new_service_pattern.captures(line) {
                        let service_name = cap.get(1).map_or("", |m| m.as_str());

                        // Skip if it's in a builder or factory file
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if file_name.contains("builder")
                            || file_name.contains("factory")
                            || file_name.contains("bootstrap")
                        {
                            continue;
                        }

                        violations.push(OrganizationViolation::ServerCreatingServices {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            service_name: service_name.to_string(),
                            suggestion: "Use DI container to resolve services".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check: Application layer importing from server
                if (is_application_layer || is_infrastructure_layer)
                    && server_import_pattern.is_match(line)
                    && !trimmed.contains("pub use")
                {
                    violations.push(OrganizationViolation::ApplicationImportsServer {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        import_statement: trimmed.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Validate strict directory placement based on component type
    ///
    /// Enforces that components are in their expected directories:
    /// - Port traits in `domain/ports/`
    /// - Adapters in `infrastructure/adapters/`
    /// - Handlers in `server/handlers/`
    /// - Repositories in appropriate locations
    pub fn validate_strict_directory(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Patterns for detecting component types
        let port_trait_pattern = Regex::new(
            r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*(?:Provider|Service|Repository|Interface))\s*:",
        )
        .expect("Invalid regex");
        let handler_struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Handler)").expect("Invalid regex");
        let adapter_impl_pattern = Regex::new(
            r"impl\s+(?:async\s+)?([A-Z][a-zA-Z0-9_]*(?:Provider|Repository))\s+for\s+([A-Z][a-zA-Z0-9_]*)"
        ).expect("Invalid regex");

        for_each_scan_rs_path(&self.config, true, |path, src_dir| {
            let is_domain_crate = src_dir.to_string_lossy().contains("domain");
            let is_infrastructure_crate = src_dir.to_string_lossy().contains("infrastructure");
            let is_server_crate = src_dir.to_string_lossy().contains("server");

            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Skip mod.rs and lib.rs (aggregator files)
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "mod.rs" || file_name == "lib.rs" {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;

            // Check for port traits outside allowed directories
            if is_domain_crate {
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = port_trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map_or("", |m| m.as_str());

                        // Allowed in: ports/, domain_services/, repositories/
                        // Domain service interfaces belong in domain_services
                        // Repository interfaces belong in repositories
                        let in_allowed_dir = path_str.contains("/ports/")
                            || path_str.contains("/domain_services/")
                            || path_str.contains("/repositories/");

                        if !in_allowed_dir {
                            violations.push(OrganizationViolation::PortOutsidePorts {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            // Check for handlers outside allowed directories
            if is_server_crate {
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = handler_struct_pattern.captures(line) {
                        let handler_name = cap.get(1).map_or("", |m| m.as_str());

                        // Allowed in: handlers/, admin/, tools/, and cross-cutting files
                        // Admin handlers belong in admin/
                        // Tool handlers belong in tools/
                        // Auth handlers are cross-cutting concerns
                        let in_allowed_location = path_str.contains("/handlers/")
                            || path_str.contains("/admin/")
                            || path_str.contains("/tools/")
                            || file_name == "auth.rs"
                            || file_name == "middleware.rs";

                        if !in_allowed_location {
                            violations.push(OrganizationViolation::HandlerOutsideHandlers {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                handler_name: handler_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            // Check for adapter implementations outside allowed directories
            if is_infrastructure_crate {
                for line in content.lines() {
                    if let Some(cap) = adapter_impl_pattern.captures(line) {
                        let _trait_name = cap.get(1).map_or("", |m| m.as_str());
                        let _impl_name = cap.get(2).map_or("", |m| m.as_str());

                        // Allowed in: adapters/, di/, and cross-cutting concern directories
                        // crypto/, cache/, health/, events/ are infrastructure cross-cutting concerns
                        let in_allowed_dir = path_str.contains("/adapters/")
                            || path_str.contains("/di/")
                            || path_str.contains("/crypto/")
                            || path_str.contains("/cache/")
                            || path_str.contains("/health/")
                            || path_str.contains("/events/")
                            || path_str.contains("/sync/")
                            || path_str.contains("/config/")
                            || path_str.contains("/infrastructure/") // Null impls for DI
                            || file_name.contains("factory")
                            || file_name.contains("bootstrap");

                        if !in_allowed_dir {
                            let current_dir = path
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();

                            violations.push(OrganizationViolation::StrictDirectoryViolation {
                                file: path.to_path_buf(),
                                component_type: ComponentType::Adapter,
                                current_directory: current_dir,
                                expected_directory: "infrastructure/adapters/".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Validate domain layer is trait-only (no impl blocks with business logic)
    ///
    /// Domain layer should only contain:
    /// - Trait definitions
    /// - Struct/enum data definitions
    /// - Simple constructors and accessors
    /// - Derived impls (Default, Clone, etc.)
    pub fn validate_domain_traits_only(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Pattern for impl blocks with methods
        let impl_block_pattern =
            Regex::new(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s*\{").expect("Invalid regex");
        let method_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")
            .expect("Invalid regex");

        // Allowed method names (constructors, accessors, conversions, simple getters)
        let allowed_methods = [
            "new",
            "default",
            "from",
            "into",
            "as_ref",
            "as_mut",
            "clone",
            "fmt",
            "eq",
            "cmp",
            "hash",
            "partial_cmp",
            "is_empty",
            "len",
            "iter",
            "into_iter",
            // Value object utility methods
            "total_changes",
            "from_ast",
            "from_fallback",
            // Simple getters that start with common prefixes
        ];
        // Also allow any method starting with common prefixes (factory methods on value objects)
        // Note: These are checked inline below rather than via this array for performance
        let _allowed_prefixes = [
            "from_", "into_", "as_", "to_", "get_", "is_", "has_", "with_",
        ];

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            // Only check domain crate
            if !src_dir.to_string_lossy().contains("domain") {
                return Ok(());
            }

            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Skip ports (trait definitions expected there)
            if path_str.contains("/ports/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_impl_block = false;
            let mut impl_name = String::new();
            let mut brace_depth = 0;
            let mut impl_start_brace = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track impl blocks
                if let Some(cap) = impl_block_pattern.captures(line) {
                    if !trimmed.contains("trait ") {
                        in_impl_block = true;
                        impl_name = cap.get(1).map_or("", |m| m.as_str()).to_string();
                        impl_start_brace = brace_depth;
                    }
                }

                // Track brace depth
                brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                // Exit impl block when braces close
                if in_impl_block && brace_depth <= impl_start_brace {
                    in_impl_block = false;
                }

                // Check methods in impl blocks
                if in_impl_block {
                    if let Some(cap) = method_pattern.captures(line) {
                        let method_name = cap.get(1).map_or("", |m| m.as_str());

                        // Skip allowed methods
                        if allowed_methods.contains(&method_name) {
                            continue;
                        }

                        // Skip if method name starts with allowed prefix
                        if method_name.starts_with("get_")
                            || method_name.starts_with("is_")
                            || method_name.starts_with("has_")
                            || method_name.starts_with("to_")
                            || method_name.starts_with("as_")
                            || method_name.starts_with("with_")
                            || method_name.starts_with("from_")
                            || method_name.starts_with("into_")
                        {
                            continue;
                        }

                        // This looks like business logic in domain layer
                        violations.push(OrganizationViolation::DomainLayerImplementation {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            impl_type: "method".to_string(),
                            type_name: format!("{}::{}", impl_name, method_name),
                            severity: Severity::Info,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for OrganizationValidator {
    fn name(&self) -> &'static str {
        "organization"
    }

    fn description(&self) -> &'static str {
        "Validates code organization patterns"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
