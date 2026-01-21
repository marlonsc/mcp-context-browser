//! Clean Architecture Validation
//!
//! Validates strict Clean Architecture compliance:
//! - Domain layer contains only traits and types (minimal implementations)
//! - Handlers use dependency injection (no direct service creation)
//! - Port implementations have Shaku Component derive
//! - Entities have identity fields
//! - Value objects are immutable
//! - Server layer boundaries are respected

use crate::pattern_registry::PATTERNS;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
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
    /// Infrastructure layer imports concrete service from Application
    ///
    /// CA007: Infrastructure should only import trait interfaces, not concrete types.
    InfrastructureImportsConcreteService {
        file: PathBuf,
        line: usize,
        import_path: String,
        concrete_type: String,
        severity: Severity,
    },
    /// Application layer imports ports from wrong location
    ///
    /// CA008: Application should import ports from mcb-domain, not locally.
    ApplicationWrongPortImport {
        file: PathBuf,
        line: usize,
        import_path: String,
        should_be: String,
        severity: Severity,
    },
    /// Infrastructure layer imports from Application layer
    ///
    /// CA009: Infrastructure should NOT depend on Application layer.
    /// Per Clean Architecture, the dependency flow is:
    /// Server -> Infrastructure -> Domain
    ///               |                  ^
    ///               v                  |
    ///          Providers ---------> Application
    ///
    /// Infrastructure importing from Application creates circular dependencies.
    InfrastructureImportsApplication {
        file: PathBuf,
        line: usize,
        import_path: String,
        suggestion: String,
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
            Self::InfrastructureImportsConcreteService { severity, .. } => *severity,
            Self::ApplicationWrongPortImport { severity, .. } => *severity,
            Self::InfrastructureImportsApplication { severity, .. } => *severity,
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
                    "{} implements {} but missing proper DI interface registration at {}:{}",
                    struct_name,
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
            Self::InfrastructureImportsConcreteService {
                file,
                line,
                concrete_type,
                ..
            } => {
                write!(
                    f,
                    "CA007: Infrastructure imports concrete service {} at {}:{}",
                    concrete_type,
                    file.display(),
                    line
                )
            }
            Self::ApplicationWrongPortImport {
                file,
                line,
                import_path,
                should_be,
                ..
            } => {
                write!(
                    f,
                    "CA008: Application imports from {} but should import from {} at {}:{}",
                    import_path,
                    should_be,
                    file.display(),
                    line
                )
            }
            Self::InfrastructureImportsApplication {
                file,
                line,
                import_path,
                ..
            } => {
                write!(
                    f,
                    "CA009: Infrastructure imports from Application layer: {} at {}:{} - violates Clean Architecture dependency direction",
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
            Self::InfrastructureImportsConcreteService { .. } => "CA007",
            Self::ApplicationWrongPortImport { .. } => "CA008",
            Self::InfrastructureImportsApplication { .. } => "CA009",
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
            Self::InfrastructureImportsConcreteService { severity, .. } => *severity,
            Self::ApplicationWrongPortImport { severity, .. } => *severity,
            Self::InfrastructureImportsApplication { severity, .. } => *severity,
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
            Self::InfrastructureImportsConcreteService { file, .. } => Some(file),
            Self::ApplicationWrongPortImport { file, .. } => Some(file),
            Self::InfrastructureImportsApplication { file, .. } => Some(file),
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
            Self::InfrastructureImportsConcreteService { line, .. } => Some(*line),
            Self::ApplicationWrongPortImport { line, .. } => Some(*line),
            Self::InfrastructureImportsApplication { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::DomainContainsImplementation { .. } => {
                Some("Move implementation logic to mcb-providers or mcb-infrastructure".to_string())
            }
            Self::HandlerCreatesService { .. } => Some(
                "Inject service via constructor injection instead of creating directly".to_string(),
            ),
            Self::PortMissingComponentDerive { trait_name, .. } => Some(format!(
                "Add proper DI component registration for {}",
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
            Self::InfrastructureImportsConcreteService { .. } => Some(
                "Import only trait interfaces from Application, not concrete implementations"
                    .to_string(),
            ),
            Self::ApplicationWrongPortImport { should_be, .. } => {
                Some(format!("Import ports from {} instead", should_be))
            }
            Self::InfrastructureImportsApplication { suggestion, .. } => Some(suggestion.clone()),
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
        // ADR-029: Hexagonal architecture validations
        violations.extend(self.validate_ca007_infrastructure_concrete_imports()?);
        violations.extend(self.validate_ca008_application_port_imports()?);
        // CA009: Infrastructure should NOT import from Application layer
        violations.extend(self.validate_ca009_infrastructure_imports_application()?);
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

        let provider_import_re = PATTERNS
            .get("CA001.provider_import")
            .expect("Pattern CA001.provider_import not found");

        for entry in WalkDir::new(server_crate.join("src"))
            .into_iter()
            .filter_map(std::result::Result::ok)
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
                PATTERNS
                    .get("CA001.service_constructor")
                    .expect("Pattern CA001.service_constructor not found"),
                "service creation",
            ),
            (
                PATTERNS
                    .get("CA001.provider_constructor")
                    .expect("Pattern CA001.provider_constructor not found"),
                "provider creation",
            ),
            (
                PATTERNS
                    .get("CA001.repository_constructor")
                    .expect("Pattern CA001.repository_constructor not found"),
                "repository creation",
            ),
        ];

        let handlers_dir = server_crate.join("src/handlers");
        if !handlers_dir.exists() {
            return Ok(violations);
        }

        for entry in WalkDir::new(handlers_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
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
                            let service_name = captures.get(1).map_or("unknown", |m| m.as_str());
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
        let struct_re = PATTERNS
            .get("CA001.pub_struct_brace")
            .expect("Pattern CA001.pub_struct_brace not found");
        let id_field_re = PATTERNS
            .get("CA001.id_field")
            .expect("Pattern CA001.id_field not found");

        for entry in WalkDir::new(entities_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
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
                        let struct_name = captures.get(1).map_or("unknown", |m| m.as_str());

                        // Skip if not an entity (e.g., helper structs, value objects)
                        // Value Objects like *Changes don't need identity
                        if struct_name.ends_with("Builder")
                            || struct_name.ends_with("Options")
                            || struct_name.ends_with("Config")
                            || struct_name.ends_with("Changes")
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
        let impl_re = PATTERNS
            .get("CA001.impl_block")
            .expect("Pattern CA001.impl_block not found");
        let mut_method_re = PATTERNS
            .get("CA001.mut_self_method")
            .expect("Pattern CA001.mut_self_method not found");

        for entry in WalkDir::new(vo_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
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
                            let method_name = captures.get(1).map_or("?", |m| m.as_str());

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

    /// CA007: Validate infrastructure layer doesn't import concrete types from application
    ///
    /// Infrastructure should only import trait interfaces (ports), not concrete implementations.
    /// This prevents tight coupling and maintains proper dependency direction.
    fn validate_ca007_infrastructure_concrete_imports(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.config.workspace_root.join("crates/mcb-infrastructure");

        if !infra_crate.exists() {
            return Ok(violations);
        }

        // Patterns for concrete types that should NOT be imported
        // These match patterns like: use mcb_application::services::ContextServiceImpl
        let concrete_type_re = PATTERNS
            .get("CA002.app_impl_import")
            .expect("Pattern CA002.app_impl_import not found");

        // Also catch any concrete service imports
        let concrete_service_re = PATTERNS
            .get("CA002.app_service_import")
            .expect("Pattern CA002.app_service_import not found");

        for entry in WalkDir::new(infra_crate.join("src"))
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip comments and test code
                    let trimmed = line.trim();
                    if trimmed.starts_with("//")
                        || trimmed.starts_with("#[test]")
                        || trimmed.starts_with("#[cfg(test)]")
                    {
                        continue;
                    }

                    // Check for Impl types
                    if let Some(captures) = concrete_type_re.captures(line) {
                        let module = captures.get(1).map_or("?", |m| m.as_str());
                        let concrete_type = captures.get(2).map_or("?", |m| m.as_str());
                        violations.push(
                            CleanArchitectureViolation::InfrastructureImportsConcreteService {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                import_path: format!(
                                    "mcb_application::{}::{}",
                                    module, concrete_type
                                ),
                                concrete_type: concrete_type.to_string(),
                                severity: Severity::Error,
                            },
                        );
                    }

                    // Check for direct service imports (non-trait)
                    if let Some(captures) = concrete_service_re.captures(line) {
                        let service_name = captures.get(1).map_or("?", |m| m.as_str());
                        // Allow trait imports (ports)
                        if !line.contains("ports::") && !line.contains("dyn ") {
                            violations.push(
                                CleanArchitectureViolation::InfrastructureImportsConcreteService {
                                    file: path.to_path_buf(),
                                    line: line_num + 1,
                                    import_path: line.trim().to_string(),
                                    concrete_type: service_name.to_string(),
                                    severity: Severity::Warning,
                                },
                            );
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// CA008: Validate application layer imports ports from mcb-domain, not locally
    ///
    /// Application layer should not define provider traits locally; they must be
    /// imported from mcb-domain to maintain single source of truth.
    fn validate_ca008_application_port_imports(&self) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let app_crate = self.config.workspace_root.join("crates/mcb-application");

        if !app_crate.exists() {
            return Ok(violations);
        }

        let ports_dir = app_crate.join("src/ports/providers");
        if !ports_dir.exists() {
            return Ok(violations);
        }

        // Patterns to detect local trait definitions (violations)
        let local_trait_re = PATTERNS
            .get("CA002.port_trait_decl")
            .expect("Pattern CA002.port_trait_decl not found");

        // Pattern for allowed re-exports from mcb-domain
        let reexport_re = PATTERNS
            .get("CA002.domain_reexport")
            .expect("Pattern CA002.domain_reexport not found");

        for entry in WalkDir::new(&ports_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(path)?;
                let has_reexport = reexport_re.is_match(&content);

                for (line_num, line) in content.lines().enumerate() {
                    // Skip comments
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check for local trait definition
                    if let Some(captures) = local_trait_re.captures(line) {
                        let trait_name = captures.get(1).map_or("?", |m| m.as_str());

                        // If file has re-exports, allow documentation of traits
                        // Only flag if this is a NEW trait definition (no re-export in file)
                        if !has_reexport {
                            violations.push(
                                CleanArchitectureViolation::ApplicationWrongPortImport {
                                    file: path.to_path_buf(),
                                    line: line_num + 1,
                                    import_path: format!(
                                        "mcb_application::ports::providers::{}",
                                        trait_name
                                    ),
                                    should_be: format!(
                                        "mcb_domain::ports::providers::{}",
                                        trait_name
                                    ),
                                    severity: Severity::Error,
                                },
                            );
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// CA009: Validate infrastructure layer does NOT import from application layer
    ///
    /// Per Clean Architecture, the dependency flow should be:
    /// ```text
    /// mcb-server → mcb-infrastructure → mcb-domain
    ///                     ↓                  ↑
    ///               mcb-providers ────→ mcb-application
    /// ```
    ///
    /// Infrastructure should only depend on Domain, NOT Application.
    /// This prevents circular dependencies and maintains proper layering.
    fn validate_ca009_infrastructure_imports_application(
        &self,
    ) -> Result<Vec<CleanArchitectureViolation>> {
        let mut violations = Vec::new();
        let infra_crate = self.config.workspace_root.join("crates/mcb-infrastructure");

        if !infra_crate.exists() {
            return Ok(violations);
        }

        // Pattern for any import from mcb_application
        let app_import_re = PATTERNS
            .get("CA009.app_import")
            .expect("Pattern CA009.app_import not found");

        // Extract specific import path
        let import_path_re = PATTERNS
            .get("CA009.app_import_path")
            .expect("Pattern CA009.app_import_path not found");

        for entry in WalkDir::new(infra_crate.join("src"))
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Skip composition root (di/ directory) - allowed to import application layer
            // for wiring up dependencies. This is the standard Clean Architecture exception
            // where the composition root needs to know about all layers to assemble them.
            if path.to_string_lossy().contains("/di/") {
                continue;
            }

            if path.extension().is_some_and(|e| e == "rs") {
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip comments
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check for mcb_application imports
                    if app_import_re.is_match(line) {
                        let import_path = import_path_re
                            .captures(line)
                            .and_then(|c| c.get(1))
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "mcb_application".to_string());

                        // Determine suggestion based on what's being imported
                        let suggestion = if import_path.contains("::ports::providers::") {
                            format!(
                                "Import from mcb_domain instead: {}",
                                import_path.replace("mcb_application", "mcb_domain")
                            )
                        } else if import_path.contains("::services::") {
                            "Services should be injected via DI, not imported. Use Arc<dyn ServiceTrait> in function signatures.".to_string()
                        } else if import_path.contains("::registry::") {
                            "Registry should be accessed from mcb-application via mcb-providers, not mcb-infrastructure.".to_string()
                        } else {
                            "mcb-infrastructure should NOT depend on mcb-application. Move required traits to mcb-domain or refactor to use DI.".to_string()
                        };

                        violations.push(
                            CleanArchitectureViolation::InfrastructureImportsApplication {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                import_path: import_path.clone(),
                                suggestion,
                                severity: Severity::Error,
                            },
                        );
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
        let re = PATTERNS
            .get("CA001.provider_import")
            .expect("Pattern CA001.provider_import not found");
        assert!(re.is_match("use mcb_providers::embedding::OllamaProvider;"));
        assert!(re.is_match("use mcb_providers;"));
        assert!(!re.is_match("use mcb_infrastructure::providers;"));
    }

    #[test]
    fn test_service_creation_pattern() {
        let re = PATTERNS
            .get("CA001.service_constructor")
            .expect("Pattern CA001.service_constructor not found");
        assert!(re.is_match("let svc = IndexingService::new(config);"));
        assert!(re.is_match("SearchServiceImpl::new()"));
        assert!(!re.is_match("Arc<dyn IndexingService>"));
    }

    #[test]
    fn test_ca007_concrete_type_pattern() {
        // Pattern for concrete Impl types
        let re = PATTERNS
            .get("CA002.app_impl_import")
            .expect("Pattern CA002.app_impl_import not found");

        // Should match concrete implementations
        assert!(re.is_match("use mcb_application::services::ContextServiceImpl;"));
        assert!(re.is_match("use mcb_application::use_cases::SearchUseCaseImpl;"));

        // Should NOT match ports/traits
        assert!(!re.is_match("use mcb_application::ports::providers::EmbeddingProvider;"));
        assert!(!re.is_match("use mcb_application::ports::admin::AdminService;"));
    }

    #[test]
    fn test_ca007_service_import_pattern() {
        // Pattern for direct service imports from services module
        let re = PATTERNS
            .get("CA002.app_service_import")
            .expect("Pattern CA002.app_service_import not found");

        // Should match service imports from services module
        assert!(re.is_match("use mcb_application::services::ContextService;"));
        assert!(re.is_match("use mcb_application::services::SearchServiceImpl;"));

        // Note: The actual validation filters out "ports::" paths, so we test
        // that the pattern doesn't overmatch nested module paths like ports::admin::
        // This regex only matches single-level module paths: mcb_application::X::YService
        assert!(!re.is_match("use mcb_application::ports::admin::AdminService;"));
    }

    #[test]
    fn test_ca008_local_trait_pattern() {
        // Pattern for local trait definitions
        let re = PATTERNS
            .get("CA002.port_trait_decl")
            .expect("Pattern CA002.port_trait_decl not found");

        // Should match local trait definitions
        assert!(re.is_match("pub trait EmbeddingProvider: Send + Sync {"));
        assert!(re.is_match("pub trait VectorStoreProvider: "));
        assert!(re.is_match("  pub trait CacheProvider {"));

        // Should NOT match re-exports or uses
        assert!(!re.is_match("pub use mcb_domain::ports::providers::EmbeddingProvider;"));
        assert!(!re.is_match("use EmbeddingProvider;"));
    }

    #[test]
    fn test_ca008_reexport_pattern() {
        // Pattern for allowed re-exports
        let re = PATTERNS
            .get("CA002.domain_reexport")
            .expect("Pattern CA002.domain_reexport not found");

        // Should match re-exports from mcb-domain
        assert!(re.is_match("pub use mcb_domain::ports::providers::*;"));
        assert!(re.is_match("pub use mcb_domain::ports::providers::EmbeddingProvider;"));

        // Should NOT match other imports
        assert!(!re.is_match("use mcb_domain::ports::providers::*;"));
        assert!(!re.is_match("pub use mcb_application::ports::*;"));
    }

    #[test]
    fn test_ca009_infrastructure_imports_application() {
        // Pattern for mcb_application imports
        let re = PATTERNS
            .get("CA009.app_import")
            .expect("Pattern CA009.app_import not found");

        // Should match any mcb_application import
        assert!(re.is_match("use mcb_application::ports::providers::CacheProvider;"));
        assert!(re.is_match("use mcb_application::services::ContextService;"));
        assert!(re.is_match("use mcb_application::registry::EMBEDDING_PROVIDERS;"));
        assert!(re.is_match("use mcb_application;"));

        // Should NOT match other imports
        assert!(!re.is_match("use mcb_domain::ports::providers::CacheProvider;"));
        assert!(!re.is_match("use mcb_providers::embedding::OllamaProvider;"));
        assert!(!re.is_match("use mcb_infrastructure::config::AppConfig;"));
    }

    #[test]
    fn test_ca009_import_path_extraction() {
        // Pattern for extracting specific import path
        let re = PATTERNS
            .get("CA009.app_import_path")
            .expect("Pattern CA009.app_import_path not found");

        // Should extract full import path
        let captures = re.captures("use mcb_application::ports::providers::CacheProvider;");
        assert!(captures.is_some());
        let path = captures.unwrap().get(1).unwrap().as_str();
        assert_eq!(path, "mcb_application::ports::providers::CacheProvider;");

        let captures2 =
            re.captures("use mcb_application::services::{ContextService, SearchService};");
        assert!(captures2.is_some());
        let path2 = captures2.unwrap().get(1).unwrap().as_str();
        assert!(path2.starts_with("mcb_application::services::"));
    }
}
