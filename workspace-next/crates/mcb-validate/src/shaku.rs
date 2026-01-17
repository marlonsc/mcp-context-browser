//! DI/Shaku Pattern Validation
//!
//! Validates code against Dependency Injection and Shaku patterns:
//! - Direct ::new() calls for service types (should use DI)
//! - Service structs without #[derive(Component)]
//! - Null/Fake/Mock types in production code
//! - Dual initialization paths (both DI and manual)
//! - Services not wired to the system

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// DI/Shaku violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShakuViolation {
    /// Direct ::new() call for type that should use DI
    DirectInstantiation {
        file: PathBuf,
        line: usize,
        type_name: String,
        suggestion: String,
        severity: Severity,
    },

    /// Service struct without #[derive(Component)]
    UnregisteredService {
        file: PathBuf,
        line: usize,
        service_name: String,
        severity: Severity,
    },

    /// Null/Fake/Mock type in production code
    FakeImplementation {
        file: PathBuf,
        line: usize,
        type_name: String,
        context: String,
        severity: Severity,
    },

    /// Dual initialization path (both DI and manual)
    DualInitializationPath {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },

    /// File named null.rs in production code
    NullProviderFile {
        file: PathBuf,
        severity: Severity,
    },

    /// Adapter implementation missing #[derive(Component)]
    MissingComponentDerive {
        file: PathBuf,
        line: usize,
        struct_name: String,
        implemented_trait: String,
        severity: Severity,
    },

    /// Adapter missing #[shaku(interface = ...)] attribute
    MissingShakuInterface {
        file: PathBuf,
        line: usize,
        struct_name: String,
        implemented_trait: Option<String>,
        severity: Severity,
    },

    /// Port trait missing Interface + Send + Sync bounds
    PortMissingInterfaceBound {
        file: PathBuf,
        line: usize,
        trait_name: String,
        missing_bounds: Vec<String>,
        severity: Severity,
    },

    /// Handler injecting concrete type instead of Arc<dyn Trait>
    HandlerInjectingConcreteType {
        file: PathBuf,
        line: usize,
        field_name: String,
        concrete_type: String,
        expected_pattern: String,
        severity: Severity,
    },

    /// Injected field missing #[shaku(inject)] attribute
    MissingShakuInject {
        file: PathBuf,
        line: usize,
        field_name: String,
        field_type: String,
        severity: Severity,
    },

    /// Null/Dev provider used in production entry point (main.rs, bootstrap.rs)
    DevNullInProduction {
        file: PathBuf,
        line: usize,
        type_name: String,
        severity: Severity,
    },

    /// Fallback chain that silently uses Null provider
    FallbackChain {
        file: PathBuf,
        line: usize,
        pattern: String,
        severity: Severity,
    },

    /// Conditional fake in production using cfg feature
    ConditionalFakeInProduction {
        file: PathBuf,
        line: usize,
        feature_name: String,
        fake_type: String,
        severity: Severity,
    },

    /// Enum containing concrete provider types (violates OCP)
    ConcreteTypesInEnum {
        file: PathBuf,
        line: usize,
        enum_name: String,
        concrete_types: Vec<String>,
        severity: Severity,
    },

    /// Forbidden import of concrete type from mcb-providers in infrastructure/server
    CrossCrateConcreteImport {
        file: PathBuf,
        line: usize,
        imported_type: String,
        source_crate: String,
        severity: Severity,
    },

    /// Service creating its own dependencies via ::new()
    ServiceCreatingDependency {
        file: PathBuf,
        line: usize,
        service_name: String,
        created_type: String,
        severity: Severity,
    },
}

impl ShakuViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::DirectInstantiation { severity, .. } => *severity,
            Self::UnregisteredService { severity, .. } => *severity,
            Self::FakeImplementation { severity, .. } => *severity,
            Self::DualInitializationPath { severity, .. } => *severity,
            Self::NullProviderFile { severity, .. } => *severity,
            Self::MissingComponentDerive { severity, .. } => *severity,
            Self::MissingShakuInterface { severity, .. } => *severity,
            Self::PortMissingInterfaceBound { severity, .. } => *severity,
            Self::HandlerInjectingConcreteType { severity, .. } => *severity,
            Self::MissingShakuInject { severity, .. } => *severity,
            Self::DevNullInProduction { severity, .. } => *severity,
            Self::FallbackChain { severity, .. } => *severity,
            Self::ConditionalFakeInProduction { severity, .. } => *severity,
            Self::ConcreteTypesInEnum { severity, .. } => *severity,
            Self::CrossCrateConcreteImport { severity, .. } => *severity,
            Self::ServiceCreatingDependency { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for ShakuViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectInstantiation {
                file,
                line,
                type_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "DI: Direct instantiation of {}: {}:{} - {}",
                    type_name,
                    file.display(),
                    line,
                    suggestion
                )
            }
            Self::UnregisteredService {
                file,
                line,
                service_name,
                ..
            } => {
                write!(
                    f,
                    "DI: Service {} not registered with DI container: {}:{}",
                    service_name,
                    file.display(),
                    line
                )
            }
            Self::FakeImplementation {
                file,
                line,
                type_name,
                context,
                ..
            } => {
                write!(
                    f,
                    "DI: Fake/Null implementation {} in production: {}:{} - {}",
                    type_name,
                    file.display(),
                    line,
                    context
                )
            }
            Self::DualInitializationPath {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "DI: Dual initialization path at {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::NullProviderFile { file, .. } => {
                write!(
                    f,
                    "DI: Null provider file in production: {}",
                    file.display()
                )
            }
            Self::MissingComponentDerive {
                file,
                line,
                struct_name,
                implemented_trait,
                ..
            } => {
                write!(
                    f,
                    "Shaku: {} implements {} but missing #[derive(Component)]: {}:{}",
                    struct_name,
                    implemented_trait,
                    file.display(),
                    line
                )
            }
            Self::MissingShakuInterface {
                file,
                line,
                struct_name,
                implemented_trait,
                ..
            } => {
                let trait_info = implemented_trait
                    .as_ref()
                    .map(|t| format!(" (implements {})", t))
                    .unwrap_or_default();
                write!(
                    f,
                    "Shaku: {} missing #[shaku(interface = ...)]{}: {}:{}",
                    struct_name,
                    trait_info,
                    file.display(),
                    line
                )
            }
            Self::PortMissingInterfaceBound {
                file,
                line,
                trait_name,
                missing_bounds,
                ..
            } => {
                write!(
                    f,
                    "Port: {} missing bounds [{}] for Shaku DI: {}:{}",
                    trait_name,
                    missing_bounds.join(", "),
                    file.display(),
                    line
                )
            }
            Self::HandlerInjectingConcreteType {
                file,
                line,
                field_name,
                concrete_type,
                expected_pattern,
                ..
            } => {
                write!(
                    f,
                    "Handler: {} uses concrete {} instead of {}: {}:{}",
                    field_name,
                    concrete_type,
                    expected_pattern,
                    file.display(),
                    line
                )
            }
            Self::MissingShakuInject {
                file,
                line,
                field_name,
                field_type,
                ..
            } => {
                write!(
                    f,
                    "Shaku: {} ({}) missing #[shaku(inject)]: {}:{}",
                    field_name,
                    field_type,
                    file.display(),
                    line
                )
            }
            Self::DevNullInProduction {
                file,
                line,
                type_name,
                ..
            } => {
                write!(
                    f,
                    "DI: Null/Dev provider {} in production entry point: {}:{}",
                    type_name,
                    file.display(),
                    line
                )
            }
            Self::FallbackChain {
                file,
                line,
                pattern,
                ..
            } => {
                write!(
                    f,
                    "DI: Fallback chain silently uses Null provider: {}:{} - {}",
                    file.display(),
                    line,
                    pattern
                )
            }
            Self::ConditionalFakeInProduction {
                file,
                line,
                feature_name,
                fake_type,
                ..
            } => {
                write!(
                    f,
                    "DI: Conditional fake {} behind feature '{}': {}:{}",
                    fake_type,
                    feature_name,
                    file.display(),
                    line
                )
            }
            Self::ConcreteTypesInEnum {
                file,
                line,
                enum_name,
                concrete_types,
                ..
            } => {
                write!(
                    f,
                    "OCP: Enum {} contains concrete types [{}] - use Arc<dyn Trait>: {}:{}",
                    enum_name,
                    concrete_types.join(", "),
                    file.display(),
                    line
                )
            }
            Self::CrossCrateConcreteImport {
                file,
                line,
                imported_type,
                source_crate,
                ..
            } => {
                write!(
                    f,
                    "DI: Forbidden concrete import {} from {} - use traits from mcb-domain: {}:{}",
                    imported_type,
                    source_crate,
                    file.display(),
                    line
                )
            }
            Self::ServiceCreatingDependency {
                file,
                line,
                service_name,
                created_type,
                ..
            } => {
                write!(
                    f,
                    "DI: Service {} creates dependency {} via ::new() - inject via parameter: {}:{}",
                    service_name,
                    created_type,
                    file.display(),
                    line
                )
            }
        }
    }
}

/// DI/Shaku validator
pub struct ShakuValidator {
    config: ValidationConfig,
}

impl ShakuValidator {
    /// Create a new Shaku validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all DI/Shaku validations
    pub fn validate_all(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_direct_instantiation()?);
        violations.extend(self.validate_fake_implementations()?);
        violations.extend(self.validate_null_provider_files()?);
        violations.extend(self.validate_dual_initialization()?);
        violations.extend(self.validate_dev_null_in_production()?);
        violations.extend(self.validate_fallback_chains()?);
        violations.extend(self.validate_conditional_fakes()?);
        violations.extend(self.validate_port_bounds()?);
        violations.extend(self.validate_adapter_decorators()?);
        violations.extend(self.validate_handler_injections()?);
        violations.extend(self.validate_concrete_in_enum()?);
        violations.extend(self.validate_cross_crate_imports()?);
        violations.extend(self.validate_service_dependencies()?);
        Ok(violations)
    }

    /// Check for direct ::new() calls on service types
    pub fn validate_direct_instantiation(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: SomeService::new() or SomeProvider::new() or SomeRepository::new()
        let new_pattern =
            Regex::new(r"([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository|Handler|Manager))::new\s*\(")
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
                let path = entry.path();

                // Skip test files and directories
                if path.to_string_lossy().contains("/tests/")
                    || path.to_string_lossy().contains("_test.rs")
                    || path.to_string_lossy().contains("/test_")
                {
                    continue;
                }

                // Skip DI bootstrap, factory, module, and composition root files (they're allowed to instantiate)
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let path_str = path.to_string_lossy();
                if file_name == "bootstrap.rs"
                    || file_name == "module.rs"
                    || file_name == "container.rs"
                    || file_name == "factory.rs"
                    || file_name == "builder.rs"
                    || file_name == "implementation.rs"
                    || file_name == "provider.rs"  // Provider aggregators create instances
                    || file_name == "providers.rs"  // Provider config creates providers
                    || file_name == "mcp_server.rs"  // Composition root for MCP server
                    || file_name == "server.rs"  // Server composition roots
                    || file_name == "main.rs"  // Application entry point
                    || path_str.contains("/di/modules/")  // All DI module files - ALLOWED for Shaku modules
                    || path_str.contains("/di/factory/")  // All DI factory files - ALLOWED for factories
                {
                    continue;
                }

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

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
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

                    // Check for direct instantiation
                    if let Some(cap) = new_pattern.captures(line) {
                        let type_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip if it's in a builder or factory method
                        if trimmed.contains("fn new") || trimmed.contains("fn build") {
                            continue;
                        }

                        violations.push(ShakuViolation::DirectInstantiation {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            type_name: type_name.to_string(),
                            suggestion: "Use DI container.resolve() instead".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for Null/Fake/Mock types in production code
    pub fn validate_fake_implementations(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: NullXxx, FakeXxx, MockXxx, DummyXxx
        let fake_pattern =
            Regex::new(r"\b(Null|Fake|Mock|Dummy|Stub)([A-Z][a-zA-Z0-9_]*)")
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
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files and directories
                if path_str.contains("/tests/")
                    || path_str.contains("_test.rs")
                    || path_str.contains("/test_")
                {
                    continue;
                }

                // Skip files where Null/Fake types are legitimately defined or re-exported
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let path_str = path.to_string_lossy();
                if file_name == "null.rs"
                    || file_name == "mock.rs"
                    || file_name == "fake.rs"
                    || file_name == "stub.rs"
                    || file_name == "null_provider.rs"  // Null provider definitions
                    || file_name == "mod.rs"  // Re-export modules
                    || file_name == "factory.rs"  // Factory files create providers
                    || file_name == "lib.rs"  // Crate roots re-export
                    || file_name == "provider.rs"  // Provider aggregators
                    || file_name == "providers.rs"  // Provider lists/enums
                    || path_str.contains("/di/modules/")  // Shaku DI modules use null providers as defaults
                {
                    continue;
                }

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

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
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

                    // Check for fake implementations
                    if let Some(cap) = fake_pattern.captures(line) {
                        let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                        let full_name = format!("{}{}", prefix, name);

                        // Skip if it's in an import/re-export statement (definitions are elsewhere)
                        let is_import_or_reexport = trimmed.starts_with("use ")
                            || trimmed.starts_with("mod ")
                            || trimmed.starts_with("pub use ")
                            || trimmed.starts_with("pub mod ")
                            || trimmed.contains("::null::")
                            || trimmed.contains("::fake::")
                            || trimmed.contains("::mock::");

                        if !is_import_or_reexport {
                            violations.push(ShakuViolation::FakeImplementation {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                type_name: full_name,
                                context: trimmed.chars().take(60).collect(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for null.rs files in production code
    pub fn validate_null_provider_files(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test directories
                if path_str.contains("/tests/") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Note: null.rs, fake.rs, mock.rs are intentional patterns for testing
                // in mcb-providers crate, so we don't flag them as violations.
                // Only flag them if they're in inappropriate locations.
                if (file_name == "null.rs" || file_name == "fake.rs" || file_name == "mock.rs")
                    && !path_str.contains("mcb-providers")
                    && !path_str.contains("/testing/")
                {
                    violations.push(ShakuViolation::NullProviderFile {
                        file: path.to_path_buf(),
                        severity: Severity::Info,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Check for dual initialization paths
    pub fn validate_dual_initialization(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: Both Arc::new(SomeService::new()) AND container.resolve()
        let arc_new_pattern =
            Regex::new(r"Arc::new\s*\(\s*[A-Z][a-zA-Z0-9_]*::new").expect("Invalid regex");
        let resolve_pattern = Regex::new(r"\.resolve\s*[:<]").expect("Invalid regex");

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

                // Skip test files and DI modules
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                // Allow concrete imports in DI modules - they register concrete types as Shaku components
                if path_str.contains("/di/modules/") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;

                // Check if file has both patterns (indicates dual initialization)
                let has_arc_new = arc_new_pattern.is_match(&content);
                let has_resolve = resolve_pattern.is_match(&content);

                if has_arc_new && has_resolve {
                    // Find the Arc::new lines for reporting
                    let lines: Vec<&str> = content.lines().collect();

                    for (line_num, line) in lines.iter().enumerate() {
                        if arc_new_pattern.is_match(line) {
                            violations.push(ShakuViolation::DualInitializationPath {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                context: "Both manual Arc::new() and DI resolve() in same file"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                            break; // Only report once per file
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for Null/Fake providers in production entry points (main.rs, bootstrap.rs, server.rs)
    pub fn validate_dev_null_in_production(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: NullXxx, FakeXxx, MockXxx, DevXxx providers
        let null_pattern =
            Regex::new(r"\b(Null|Fake|Mock|Dev|Dummy)([A-Z][a-zA-Z0-9_]*(?:Provider|Service|Repository))")
                .expect("Invalid regex");

        // Production entry point files
        let production_files = ["main.rs", "bootstrap.rs", "server.rs", "init.rs"];

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
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                // Only check production entry point files
                if !production_files.contains(&file_name) {
                    continue;
                }

                // Skip test directories
                if path.to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check for null/fake providers
                    if let Some(cap) = null_pattern.captures(line) {
                        let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                        let full_name = format!("{}{}", prefix, name);

                        // Skip if it's in an import or type definition (we want usages)
                        if !trimmed.starts_with("use ") && !trimmed.starts_with("type ") {
                            violations.push(ShakuViolation::DevNullInProduction {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                type_name: full_name,
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for fallback chains that silently use Null providers
    pub fn validate_fallback_chains(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Patterns: .unwrap_or_else(|| Null...), .or_else(|_| Null...), .unwrap_or(Null...)
        let fallback_patterns = [
            Regex::new(r"unwrap_or_else\s*\(\s*\|\s*\|?\s*\w*(Null|Fake|Mock)[A-Z]")
                .expect("Invalid regex"),
            Regex::new(r"or_else\s*\(\s*\|_?\|?\s*\w*(Null|Fake|Mock)[A-Z]")
                .expect("Invalid regex"),
            Regex::new(r"unwrap_or\s*\(\s*(Null|Fake|Mock)[A-Z]")
                .expect("Invalid regex"),
        ];

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

                // Skip test files and DI modules
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                // Allow concrete imports in DI modules - they register concrete types as Shaku components
                if path_str.contains("/di/modules/") {
                    continue;
                }

                // Skip null/fake provider definition files
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "null.rs" || file_name == "fake.rs" || file_name == "mock.rs" {
                    continue;
                }

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

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
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

                    // Check for fallback patterns
                    for pattern in &fallback_patterns {
                        if pattern.is_match(line) {
                            violations.push(ShakuViolation::FallbackChain {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                pattern: trimmed.chars().take(80).collect(),
                                severity: Severity::Info,
                            });
                            break; // Only report once per line
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for conditional fakes using cfg features
    pub fn validate_conditional_fakes(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: #[cfg(feature = "mock")] or #[cfg(feature = "test")] or #[cfg(feature = "fake")]
        let cfg_pattern =
            Regex::new(r#"#\[cfg\(feature\s*=\s*"(mock|test|fake|dev|stub)"\)\]"#)
                .expect("Invalid regex");
        let fake_type_pattern =
            Regex::new(r"\b(Null|Fake|Mock|Dummy|Stub)([A-Z][a-zA-Z0-9_]*)")
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
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files and directories
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                let mut pending_cfg_feature: Option<(usize, String)> = None;

                for (line_num, line) in lines.iter().enumerate() {
                    let trimmed = line.trim();

                    // Check for cfg feature attribute
                    if let Some(cap) = cfg_pattern.captures(trimmed) {
                        let feature = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        pending_cfg_feature = Some((line_num + 1, feature.to_string()));
                        continue;
                    }

                    // If we have a pending cfg feature, check if next line has fake type
                    if let Some((cfg_line, feature)) = pending_cfg_feature.take() {
                        if let Some(cap) = fake_type_pattern.captures(trimmed) {
                            let prefix = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                            let full_name = format!("{}{}", prefix, name);

                            violations.push(ShakuViolation::ConditionalFakeInProduction {
                                file: path.to_path_buf(),
                                line: cfg_line,
                                feature_name: feature,
                                fake_type: full_name,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Validate port traits have required bounds for Shaku DI (Interface + Send + Sync)
    pub fn validate_port_bounds(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)\s*(?::\s*([^{]+))?\s*\{")
                .expect("Invalid regex");

        for crate_dir in self.config.get_source_dirs()? {
            let crate_name = crate_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if crate_name != "mcb-domain" {
                continue;
            }

            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                if !path_str.contains("/ports/") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "mod.rs" {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let bounds = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                        let mut missing_bounds = Vec::new();
                        if !bounds.contains("Interface") {
                            missing_bounds.push("Interface".to_string());
                        }
                        if !bounds.contains("Send") {
                            missing_bounds.push("Send".to_string());
                        }
                        if !bounds.contains("Sync") {
                            missing_bounds.push("Sync".to_string());
                        }
                        if !missing_bounds.is_empty() {
                            violations.push(ShakuViolation::PortMissingInterfaceBound {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                missing_bounds,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    /// Validate infrastructure adapters have proper Shaku decorators
    pub fn validate_adapter_decorators(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let impl_trait_pattern =
            Regex::new(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s+for\s+([A-Z][a-zA-Z0-9_]*)")
                .expect("Invalid regex");

        for crate_dir in self.config.get_source_dirs()? {
            let crate_name = crate_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if crate_name != "mcb-infrastructure" {
                continue;
            }

            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                if !path_str.contains("/adapters/") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "mod.rs" || file_name == "null.rs" {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                for (line_num, line) in lines.iter().enumerate() {
                    if let Some(cap) = struct_pattern.captures(line) {
                        let struct_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if !struct_name.ends_with("Provider")
                            && !struct_name.ends_with("Service")
                            && !struct_name.ends_with("Repository")
                            && !struct_name.ends_with("Adapter")
                        {
                            continue;
                        }

                        let mut has_derive_component = false;
                        let mut has_shaku_interface = false;
                        for prev_line_idx in (line_num.saturating_sub(10)..line_num).rev() {
                            let prev_line = lines[prev_line_idx];
                            if prev_line.contains("#[derive(") && prev_line.contains("Component") {
                                has_derive_component = true;
                            }
                            if prev_line.contains("#[shaku(interface") {
                                has_shaku_interface = true;
                            }
                            if prev_line.contains("struct ") || prev_line.contains("fn ") {
                                break;
                            }
                        }

                        let mut implemented_trait = None;
                        for check_line in lines.iter() {
                            if let Some(impl_cap) = impl_trait_pattern.captures(check_line) {
                                let impl_struct = impl_cap.get(2).map(|m| m.as_str()).unwrap_or("");
                                if impl_struct == struct_name {
                                    implemented_trait = Some(
                                        impl_cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string(),
                                    );
                                    break;
                                }
                            }
                        }

                        if !has_derive_component {
                            violations.push(ShakuViolation::MissingComponentDerive {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                struct_name: struct_name.to_string(),
                                implemented_trait: implemented_trait
                                    .clone()
                                    .unwrap_or_else(|| "unknown".to_string()),
                                severity: Severity::Warning,
                            });
                        }

                        if !has_shaku_interface && implemented_trait.is_some() {
                            violations.push(ShakuViolation::MissingShakuInterface {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                struct_name: struct_name.to_string(),
                                implemented_trait,
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    /// Validate handlers use trait injection (Arc<dyn Trait>) not concrete types
    pub fn validate_handler_injections(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        let arc_concrete_pattern =
            Regex::new(r"(\w+):\s*Arc<([A-Z][a-zA-Z0-9_]+)>").expect("Invalid regex");
        let arc_dyn_pattern =
            Regex::new(r"Arc<dyn\s+[A-Z][a-zA-Z0-9_]+>").expect("Invalid regex");

        for crate_dir in self.config.get_source_dirs()? {
            let crate_name = crate_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if crate_name != "mcb-server" {
                continue;
            }

            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();
                if !path_str.contains("/handlers/") {
                    continue;
                }

                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "mod.rs" {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let mut in_struct = false;
                let mut struct_brace_depth: i32 = 0;
                let mut brace_depth: i32 = 0;

                for (line_num, line) in content.lines().enumerate() {
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    if line.contains("struct ") && line.contains('{') {
                        in_struct = true;
                        struct_brace_depth = brace_depth;
                    }
                    if in_struct && brace_depth < struct_brace_depth {
                        in_struct = false;
                    }
                    if !in_struct {
                        continue;
                    }
                    if arc_dyn_pattern.is_match(line) {
                        continue;
                    }

                    if let Some(cap) = arc_concrete_pattern.captures(line) {
                        let field_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let concrete_type = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                        if concrete_type == "RwLock"
                            || concrete_type == "Mutex"
                            || concrete_type == "String"
                            || concrete_type == "str"
                        {
                            continue;
                        }
                        if concrete_type.ends_with("Service")
                            || concrete_type.ends_with("Provider")
                            || concrete_type.ends_with("Repository")
                            || concrete_type.ends_with("Handler")
                        {
                            violations.push(ShakuViolation::HandlerInjectingConcreteType {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                field_name: field_name.to_string(),
                                concrete_type: concrete_type.to_string(),
                                expected_pattern: format!("Arc<dyn {}>", concrete_type),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    /// Check for enums containing concrete provider types (violates OCP)
    pub fn validate_concrete_in_enum(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: enum Name { Variant(ConcreteType), ... }
        let enum_pattern = Regex::new(r"(?:pub\s+)?enum\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let variant_pattern = Regex::new(r"([A-Z][a-zA-Z0-9_]*)\s*\(([A-Z][a-zA-Z0-9_]*)\)").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate and mcb-providers (where concrete types are defined)
            let src_str = src_dir.to_string_lossy();
            if src_str.contains("mcb-validate") || src_str.contains("mcb-providers") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files and DI modules
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                // Allow concrete imports in DI modules - they register concrete types as Shaku components
                if path_str.contains("/di/modules/") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                let mut in_enum = false;
                let mut enum_name = String::new();
                let mut enum_line = 0;
                let mut concrete_types: Vec<String> = Vec::new();
                let mut brace_depth: i32 = 0;
                let mut enum_brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Detect enum start
                    if let Some(cap) = enum_pattern.captures(line) {
                        in_enum = true;
                        enum_name = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                        enum_line = line_num + 1;
                        enum_brace_depth = brace_depth;
                        concrete_types.clear();
                    }

                    // Detect enum end
                    if in_enum && brace_depth < enum_brace_depth {
                        // Report if we found concrete provider types
                        if !concrete_types.is_empty() {
                            violations.push(ShakuViolation::ConcreteTypesInEnum {
                                file: path.to_path_buf(),
                                line: enum_line,
                                enum_name: enum_name.clone(),
                                concrete_types: concrete_types.clone(),
                                severity: Severity::Warning,
                            });
                        }
                        in_enum = false;
                    }

                    // Look for variant with concrete provider type
                    if in_enum {
                        for cap in variant_pattern.captures_iter(line) {
                            let type_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
                            // Check if it's a concrete provider/service type
                            if type_name.ends_with("Provider")
                                || type_name.ends_with("Service")
                                || type_name.ends_with("Repository")
                            {
                                concrete_types.push(type_name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    /// Check for forbidden cross-crate concrete imports from mcb-providers in infrastructure/server
    pub fn validate_cross_crate_imports(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Regex matches imports like: use mcb_providers::module::Type
        let import_pattern = Regex::new(r"use\s+mcb_providers::([a-z_]+)::([A-Z][a-zA-Z0-9_,\s]*)").expect("Invalid regex");
        let type_pattern = Regex::new(r"([A-Z][a-zA-Z0-9_]+)").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            let src_str = src_dir.to_string_lossy();
            // Only check mcb-infrastructure and mcb-server (not mcb-providers or mcb-domain)
            if !src_str.contains("mcb-infrastructure") && !src_str.contains("mcb-server") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files and DI modules
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                // Allow concrete imports in DI modules - they register concrete types as Shaku components
                if path_str.contains("/di/modules/") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = import_pattern.captures(line) {
                        let _module = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let types_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                        // Extract individual type names
                        for type_cap in type_pattern.captures_iter(types_str) {
                            let type_name = type_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                            // Check if it's a concrete type (not a trait)
                            if type_name.ends_with("Provider")
                                || type_name.ends_with("Service")
                                || type_name.ends_with("Repository")
                                || type_name.ends_with("Chunker")
                            {
                                violations.push(ShakuViolation::CrossCrateConcreteImport {
                                    file: path.to_path_buf(),
                                    line: line_num + 1,
                                    imported_type: type_name.to_string(),
                                    source_crate: "mcb_providers".to_string(),
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

    /// Check for services creating their own dependencies via ::new()
    pub fn validate_service_dependencies(&self) -> Result<Vec<ShakuViolation>> {
        let mut violations = Vec::new();
        // Pattern: field: Type::new() inside impl Service::new()
        let impl_new_pattern = Regex::new(r"impl\s+([A-Z][a-zA-Z0-9_]*(?:Impl|Service))\s*\{").expect("Invalid regex");
        let fn_new_pattern = Regex::new(r"pub\s+fn\s+new\s*\(").expect("Invalid regex");
        let dependency_new_pattern = Regex::new(r"([a-z_]+):\s*([A-Z][a-zA-Z0-9_]+)::new\s*\(").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate
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

                // Skip test files and DI modules
                if path_str.contains("/tests/") || path_str.contains("_test.rs") {
                    continue;
                }

                // Allow concrete imports in DI modules - they register concrete types as Shaku components
                if path_str.contains("/di/modules/") {
                    continue;
                }

                // Skip factory files (they're supposed to create instances)
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name == "factory.rs" || file_name == "bootstrap.rs" {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                let lines: Vec<&str> = content.lines().collect();

                let mut in_impl = false;
                let mut in_fn_new = false;
                let mut current_service = String::new();
                let mut brace_depth: i32 = 0;
                let mut impl_brace_depth: i32 = 0;
                let mut fn_brace_depth: i32 = 0;

                for (line_num, line) in lines.iter().enumerate() {
                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Detect impl ServiceImpl {
                    if let Some(cap) = impl_new_pattern.captures(line) {
                        in_impl = true;
                        current_service = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                        impl_brace_depth = brace_depth;
                    }

                    // Detect end of impl
                    if in_impl && brace_depth < impl_brace_depth {
                        in_impl = false;
                        in_fn_new = false;
                    }

                    // Detect fn new() inside impl
                    if in_impl && fn_new_pattern.is_match(line) {
                        in_fn_new = true;
                        fn_brace_depth = brace_depth;
                    }

                    // Detect end of fn new
                    if in_fn_new && brace_depth < fn_brace_depth {
                        in_fn_new = false;
                    }

                    // Look for dependency::new() inside fn new()
                    if in_fn_new {
                        if let Some(cap) = dependency_new_pattern.captures(line) {
                            let _field = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                            let dep_type = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                            // Check if it's creating a service/provider/chunker
                            if dep_type.ends_with("Provider")
                                || dep_type.ends_with("Service")
                                || dep_type.ends_with("Repository")
                                || dep_type.ends_with("Chunker")
                            {
                                violations.push(ShakuViolation::ServiceCreatingDependency {
                                    file: path.to_path_buf(),
                                    line: line_num + 1,
                                    service_name: current_service.clone(),
                                    created_type: dep_type.to_string(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
        create_test_crate_with_file(temp, name, "lib.rs", content);
    }

    fn create_test_crate_with_file(temp: &TempDir, name: &str, file_name: &str, content: &str) {
        // Create workspace Cargo.toml if it doesn't exist
        let workspace_cargo = temp.path().join("Cargo.toml");
        if !workspace_cargo.exists() {
            fs::write(
                &workspace_cargo,
                r#"
[workspace]
members = ["crates/*"]
"#,
            )
            .unwrap();
        }

        // Create crate structure
        let crate_dir = temp.path().join("crates").join(name).join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(crate_dir.join(file_name), content).unwrap();

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
    fn test_direct_instantiation() {
        let temp = TempDir::new().unwrap();
        // Use service.rs instead of lib.rs since lib.rs is skipped by validator
        create_test_crate_with_file(
            &temp,
            "mcb-test",
            "service.rs",
            r#"
pub fn setup() {
    let service = MyService::new();
    let provider = EmbeddingProvider::new();
}
"#,
        );

        let validator = ShakuValidator::new(temp.path());
        let violations = validator.validate_direct_instantiation().unwrap();

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_fake_implementation() {
        let temp = TempDir::new().unwrap();
        // Use service.rs instead of lib.rs since lib.rs is skipped by validator
        create_test_crate_with_file(
            &temp,
            "mcb-test",
            "service.rs",
            r#"
pub fn setup() {
    let provider: Arc<dyn Provider> = Arc::new(NullProvider::new());
    let mock = MockService::new();
}
"#,
        );

        let validator = ShakuValidator::new(temp.path());
        let violations = validator.validate_fake_implementations().unwrap();

        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_no_violations_in_tests() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        let mock = MockService::new();
        let null = NullProvider::new();
    }
}
"#,
        );

        let validator = ShakuValidator::new(temp.path());
        let violations = validator.validate_all().unwrap();

        assert!(violations.is_empty(), "Test code should not trigger violations");
    }
}
