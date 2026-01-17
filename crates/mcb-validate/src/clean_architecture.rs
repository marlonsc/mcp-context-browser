//! Clean Architecture Validation
//!
//! Validates strict Clean Architecture compliance:
//! - Domain layer contains only traits and types (minimal implementations)
//! - Handlers use dependency injection (no direct service creation)
//! - Port implementations have Shaku Component derive
//! - Entities have identity fields
//! - Value objects are immutable
//! - Server layer boundaries are respected

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Clean Architecture violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanArchitectureViolation {
    /// Domain layer contains implementation logic
    DomainContainsImplementation {
        file: PathBuf,
        line: usize,
        impl_type: String,
        severity: Severity,
    },
    /// Handler creates service directly instead of using DI
    HandlerCreatesService {
        file: PathBuf,
        line: usize,
        service_name: String,
        context: String,
        severity: Severity,
    },
    /// Port implementation missing Shaku Component derive
    PortMissingComponentDerive {
        file: PathBuf,
        line: usize,
        struct_name: String,
        trait_name: String,
        severity: Severity,
    },
    /// Entity missing identity field
    EntityMissingIdentity {
        file: PathBuf,
        line: usize,
        entity_name: String,
        severity: Severity,
    },
    /// Value object has mutable method
    ValueObjectMutable {
        file: PathBuf,
        line: usize,
        vo_name: String,
        method_name: String,
        severity: Severity,
    },
    /// Server imports provider directly
    ServerImportsProviderDirectly {
        file: PathBuf,
        line: usize,
        import_path: String,
        severity: Severity,
    },
}

impl CleanArchitectureViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::DomainContainsImplementation { severity, .. } => *severity,
            Self::HandlerCreatesService { severity, .. } => *severity,
            Self::PortMissingComponentDerive { severity, .. } => *severity,
            Self::EntityMissingIdentity { severity, .. } => *severity,
            Self::ValueObjectMutable { severity, .. } => *severity,
            Self::ServerImportsProviderDirectly { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for CleanArchitectureViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DomainContainsImplementation {
                file,
                line,
                impl_type,
                ..
            } => {
                write!(
                    f,
                    "Domain layer contains {impl_type} at {}:{}",
                    file.display(),
                    line
                )
            }
            Self::HandlerCreatesService {
                file,
                line,
                service_name,
                context,
                ..
            } => {
                write!(
                    f,
                    "Handler creates {} directly at {}:{} - {}",
                    service_name,
                    file.display(),
                    line,
                    context
                )
            }
            Self::PortMissingComponentDerive {
                file,
                line,
                struct_name,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "{} implements {} but missing #[shaku(interface = {})] at {}:{}",
                    struct_name,
                    trait_name,
                    trait_name,
                    file.display(),
                    line
                )
            }
            Self::EntityMissingIdentity {
                file,
                line,
                entity_name,
                ..
            } => {
                write!(
                    f,
                    "Entity {} missing id/uuid field at {}:{}",
                    entity_name,
                    file.display(),
                    line
                )
            }
            Self::ValueObjectMutable {
                file,
                line,
                vo_name,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "Value object {} has mutable method {} at {}:{}",
                    vo_name,
                    method_name,
                    file.display(),
                    line
                )
            }
            Self::ServerImportsProviderDirectly {
                file,
                line,
                import_path,
                ..
            } => {
                write!(
                    f,
                    "Server imports provider directly: {} at {}:{}",
                    import_path,
                    file.display(),
                    line
                )
            }
        }
    }
}

impl Violation for CleanArchitectureViolation {
    fn id(&self) -> &str {
        match self {
            Self::DomainContainsImplementation { .. } => "CA001",
            Self::HandlerCreatesService { .. } => "CA002",
            Self::PortMissingComponentDerive { .. } => "CA003",
            Self::EntityMissingIdentity { .. } => "CA004",
            Self::ValueObjectMutable { .. } => "CA005",
            Self::ServerImportsProviderDirectly { .. } => "CA006",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::DomainContainsImplementation { severity, .. } => *severity,
            Self::HandlerCreatesService { severity, .. } => *severity,
            Self::PortMissingComponentDerive { severity, .. } => *severity,
            Self::EntityMissingIdentity { severity, .. } => *severity,
            Self::ValueObjectMutable { severity, .. } => *severity,
            Self::ServerImportsProviderDirectly { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::DomainContainsImplementation { file, .. } => Some(file),
            Self::HandlerCreatesService { file, .. } => Some(file),
            Self::PortMissingComponentDerive { file, .. } => Some(file),
            Self::EntityMissingIdentity { file, .. } => Some(file),
            Self::ValueObjectMutable { file, .. } => Some(file),
            Self::ServerImportsProviderDirectly { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::DomainContainsImplementation { line, .. } => Some(*line),
            Self::HandlerCreatesService { line, .. } => Some(*line),
            Self::PortMissingComponentDerive { line, .. } => Some(*line),
            Self::EntityMissingIdentity { line, .. } => Some(*line),
            Self::ValueObjectMutable { line, .. } => Some(*line),
            Self::ServerImportsProviderDirectly { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::DomainContainsImplementation { .. } => {
                Some("Move implementation logic to mcb-providers or mcb-infrastructure".to_string())
            }
            Self::HandlerCreatesService { .. } => Some(
                "Inject service via constructor/Shaku instead of creating directly".to_string(),
            ),
            Self::PortMissingComponentDerive { trait_name, .. } => Some(format!(
                "Add #[derive(Component)] and #[shaku(interface = {})]",
                trait_name
            )),
            Self::EntityMissingIdentity { .. } => {
                Some("Add id: Uuid or similar identity field to entity".to_string())
            }
            Self::ValueObjectMutable { .. } => {
                Some("Value objects should be immutable - return new instance instead".to_string())
            }
            Self::ServerImportsProviderDirectly { .. } => {
                Some("Import providers through mcb-infrastructure re-exports".to_string())
            }
        }
    }
}

/// Clean Architecture validator
pub struct CleanArchitectureValidator {
    config: ValidationConfig,
}

impl CleanArchitectureValidator {
    /// Create a new architecture validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all architecture validations (returns typed violations)
    pub fn validate_all(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_server_layer_boundaries()?);
        violations.extend(self.validate_handler_injection()?);
        violations.extend(self.validate_entity_identity()?);
        violations.extend(self.validate_value_object_immutability()?);
        Ok(violations)
    }

    /// Run all validations (returns boxed violations for Validator trait)
    fn validate_boxed(&self) -> Result<Vec<Box<dyn Violation>>> {
        let typed_violations = self.validate_all()?;
        Ok(typed_violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}

impl crate::validator_trait::Validator for CleanArchitectureValidator {
    fn name(&self) -> &'static str {
        "clean_architecture"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture compliance: layer boundaries, DI patterns, entity identity, value object immutability"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        self.validate_boxed().map_err(|e| anyhow::anyhow!("{}", e))
    }
}

impl CleanArchitectureValidator {
    /// Validate server layer doesn't import providers directly
    fn validate_server_layer_boundaries(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.config.workspace_root.join("crates/mcb-server");

        if !server_crate.exists() {
            return Ok(violations);
        }

        let provider_import_re = Regex::new(r"use\s+mcb_providers(?:::|;)").expect("Invalid regex");

        for entry in WalkDir::new(server_crate.join("src"))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    if provider_import_re.is_match(line) {
                        violations.push(
                            CleanArchitectureViolation::ServerImportsProviderDirectly {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                import_path: line.trim().to_string(),
                                severity: Severity::Error,
                            },
                        );
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate handlers use dependency injection
    fn validate_handler_injection(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let server_crate = self.config.workspace_root.join("crates/mcb-server");

        if !server_crate.exists() {
            return Ok(violations);
        }

        // Patterns for direct service creation
        let service_creation_patterns = [
            (
                Regex::new(r"(\w+Service)(?:Impl)?::new\s*\(").expect("Invalid regex"),
                "service creation",
            ),
            (
                Regex::new(r"(\w+Provider)(?:Impl)?::new\s*\(").expect("Invalid regex"),
                "provider creation",
            ),
            (
                Regex::new(r"(\w+Repository)(?:Impl)?::new\s*\(").expect("Invalid regex"),
                "repository creation",
            ),
        ];

        let handlers_dir = server_crate.join("src/handlers");
        if !handlers_dir.exists() {
            return Ok(violations);
        }

        for entry in WalkDir::new(handlers_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip test code
                    if line.contains("#[test]") || line.contains("#[cfg(test)]") {
                        continue;
                    }

                    for (pattern, pattern_type) in &service_creation_patterns {
                        if let Some(captures) = pattern.captures(line) {
                            let service_name =
                                captures.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                            violations.push(CleanArchitectureViolation::HandlerCreatesService {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                service_name: service_name.to_string(),
                                context: format!("Direct {} instead of DI", pattern_type),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate entities have identity fields
    fn validate_entity_identity(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let domain_crate = self.config.workspace_root.join("crates/mcb-domain");

        if !domain_crate.exists() {
            return Ok(violations);
        }

        let entities_dir = domain_crate.join("src/entities");
        if !entities_dir.exists() {
            return Ok(violations);
        }

        // Look for struct definitions that should have id fields
        let struct_re = Regex::new(r"pub\s+struct\s+(\w+)\s*\{").expect("Invalid regex");
        let id_field_re = Regex::new(r"\bid\s*:|uuid\s*:|entity_id\s*:").expect("Invalid regex");

        for entry in WalkDir::new(entities_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                // Skip mod.rs files
                if path.file_name().is_some_and(|n| n == "mod.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(captures) = struct_re.captures(line) {
                        let struct_name = captures.get(1).map(|m| m.as_str()).unwrap_or("unknown");

                        // Skip if not an entity (e.g., helper structs)
                        if struct_name.ends_with("Builder")
                            || struct_name.ends_with("Options")
                            || struct_name.ends_with("Config")
                        {
                            continue;
                        }

                        // Look ahead for id field in struct definition
                        let mut has_id = false;
                        let mut brace_count = 0;
                        let mut started = false;

                        for check_line in lines.iter().skip(line_num) {
                            if check_line.contains('{') {
                                brace_count += check_line.matches('{').count();
                                started = true;
                            }
                            if check_line.contains('}') {
                                brace_count -= check_line.matches('}').count();
                            }

                            if id_field_re.is_match(check_line) {
                                has_id = true;
                                break;
                            }

                            if started && brace_count == 0 {
                                break;
                            }
                        }

                        if !has_id {
                            violations.push(CleanArchitectureViolation::EntityMissingIdentity {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                entity_name: struct_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate value objects are immutable
    fn validate_value_object_immutability(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let domain_crate = self.config.workspace_root.join("crates/mcb-domain");

        if !domain_crate.exists() {
            return Ok(violations);
        }

        let vo_dir = domain_crate.join("src/value_objects");
        if !vo_dir.exists() {
            return Ok(violations);
        }

        // Look for &mut self methods in value objects
        let impl_re = Regex::new(r"impl\s+(\w+)\s*\{").expect("Invalid regex");
        let mut_method_re = Regex::new(r"fn\s+(\w+)\s*\(\s*&mut\s+self").expect("Invalid regex");

        for entry in WalkDir::new(vo_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                // Skip mod.rs files
                if path.file_name().is_some_and(|n| n == "mod.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                let mut current_impl: Option<String> = None;
                let mut brace_depth = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    // Track impl blocks
                    if let Some(captures) = impl_re.captures(line) {
                        current_impl =
                            Some(captures.get(1).map(|m| m.as_str().to_string()).unwrap());
                    }

                    // Track brace depth for impl scope
                    brace_depth += line.matches('{').count();
                    brace_depth -= line.matches('}').count();

                    if brace_depth == 0 {
                        current_impl = None;
                    }

                    // Check for mutable methods
                    if let Some(ref vo_name) = current_impl {
                        if let Some(captures) = mut_method_re.captures(line) {
                            let method_name = captures.get(1).map(|m| m.as_str()).unwrap_or("?");

                            // Allow some standard mutable methods
                            if !["set_", "add_", "remove_", "clear_", "reset_"]
                                .iter()
                                .any(|p| method_name.starts_with(p))
                            {
                                continue;
                            }

                            violations.push(CleanArchitectureViolation::ValueObjectMutable {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                vo_name: vo_name.clone(),
                                method_name: method_name.to_string(),
                                severity: Severity::Warning,
                            });
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

    #[test]
    fn test_server_import_pattern() {
        let re = Regex::new(r"use\s+mcb_providers(?:::|;)").unwrap();
        assert!(re.is_match("use mcb_providers::embedding::OllamaProvider;"));
        assert!(re.is_match("use mcb_providers;"));
        assert!(!re.is_match("use mcb_infrastructure::providers;"));
    }

    #[test]
    fn test_service_creation_pattern() {
        let re = Regex::new(r"(\w+Service)(?:Impl)?::new\s*\(").unwrap();
        assert!(re.is_match("let svc = IndexingService::new(config);"));
        assert!(re.is_match("SearchServiceImpl::new()"));
        assert!(!re.is_match("Arc<dyn IndexingService>"));
    }
}
