//! Layer Event Flow Validation
//!
//! Validates that dependencies flow in correct Clean Architecture direction:
//! mcb-domain -> mcb-application -> mcb-providers -> mcb-infrastructure -> mcb-server

use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationConfig};
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Layer Flow Violations
#[derive(Debug, Clone, Serialize)]
pub enum LayerFlowViolation {
    ForbiddenDependency {
        source_crate: String,
        target_crate: String,
        import_path: String,
        file: PathBuf,
        line: usize,
    },
    CircularDependency {
        crate_a: String,
        crate_b: String,
        file: PathBuf,
        line: usize,
    },
    DomainExternalDependency {
        crate_name: String,
        external_crate: String,
        file: PathBuf,
        line: usize,
    },
}

impl std::fmt::Display for LayerFlowViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForbiddenDependency {
                source_crate,
                target_crate,
                import_path,
                file,
                line,
            } => write!(
                f,
                "CA: Forbidden import in {}: {} (imports {}) at {}:{}",
                source_crate,
                import_path,
                target_crate,
                file.display(),
                line
            ),
            Self::CircularDependency {
                crate_a,
                crate_b,
                file,
                line,
            } => write!(
                f,
                "CA: Circular dependency: {} <-> {} at {}:{}",
                crate_a,
                crate_b,
                file.display(),
                line
            ),
            Self::DomainExternalDependency {
                crate_name,
                external_crate,
                file,
                line,
            } => write!(
                f,
                "CA: Domain {} imports external: {} at {}:{}",
                crate_name,
                external_crate,
                file.display(),
                line
            ),
        }
    }
}

impl Violation for LayerFlowViolation {
    fn id(&self) -> &str {
        match self {
            Self::ForbiddenDependency { .. } => "LAYER001",
            Self::CircularDependency { .. } => "LAYER002",
            Self::DomainExternalDependency { .. } => "LAYER003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ForbiddenDependency { .. } | Self::CircularDependency { .. } => Severity::Error,
            Self::DomainExternalDependency { .. } => Severity::Warning,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::ForbiddenDependency { file, .. }
            | Self::CircularDependency { file, .. }
            | Self::DomainExternalDependency { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ForbiddenDependency { line, .. }
            | Self::CircularDependency { line, .. }
            | Self::DomainExternalDependency { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::ForbiddenDependency {
                source_crate,
                target_crate,
                ..
            } => Some(format!(
                "Remove {} from {} - violates CA",
                target_crate, source_crate
            )),
            Self::CircularDependency { .. } => {
                Some("Extract shared types to mcb-domain".to_string())
            }
            Self::DomainExternalDependency { .. } => {
                Some("Domain should only use std/serde/thiserror".to_string())
            }
        }
    }
}

struct LayerRules {
    forbidden: HashMap<&'static str, HashSet<&'static str>>,
}

impl Default for LayerRules {
    fn default() -> Self {
        let mut forbidden = HashMap::new();
        forbidden.insert(
            "mcb-domain",
            [
                "mcb-application",
                "mcb-providers",
                "mcb-infrastructure",
                "mcb-server",
            ]
            .into_iter()
            .collect(),
        );
        forbidden.insert(
            "mcb-application",
            ["mcb-providers", "mcb-infrastructure", "mcb-server"]
                .into_iter()
                .collect(),
        );
        forbidden.insert(
            "mcb-providers",
            ["mcb-infrastructure", "mcb-server"].into_iter().collect(),
        );
        forbidden.insert("mcb-infrastructure", ["mcb-server"].into_iter().collect());
        forbidden.insert("mcb-server", ["mcb-providers"].into_iter().collect());
        Self { forbidden }
    }
}

/// Layer Flow Validator
pub struct LayerFlowValidator {
    rules: LayerRules,
}

impl Default for LayerFlowValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerFlowValidator {
    pub fn new() -> Self {
        Self {
            rules: LayerRules::default(),
        }
    }

    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.check_forbidden_imports(config)?);
        violations.extend(self.check_circular_dependencies(config)?);
        Ok(violations)
    }

    fn check_forbidden_imports(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        let crates_dir = config.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(violations);
        }

        let import_pattern = Regex::new(r"use\s+(mcb_\w+)").expect("Invalid regex");

        for crate_name in self.rules.forbidden.keys() {
            let crate_dir = crates_dir.join(crate_name).join("src");
            if !crate_dir.exists() {
                continue;
            }

            let forbidden_deps = &self.rules.forbidden[crate_name];
            let crate_name_underscored = crate_name.replace('-', "_");

            for entry in WalkDir::new(&crate_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }

                let content = std::fs::read_to_string(path)?;
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                        continue;
                    }

                    for captures in import_pattern.captures_iter(line) {
                        let imported_crate = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                        let imported_crate_dashed = imported_crate.replace('_', "-");
                        if imported_crate == crate_name_underscored {
                            continue;
                        }
                        if forbidden_deps.contains(imported_crate_dashed.as_str()) {
                            violations.push(LayerFlowViolation::ForbiddenDependency {
                                source_crate: crate_name.to_string(),
                                target_crate: imported_crate_dashed,
                                import_path: line.trim().to_string(),
                                file: path.to_path_buf(),
                                line: line_num + 1,
                            });
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    fn check_circular_dependencies(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<LayerFlowViolation>> {
        let mut violations = Vec::new();
        let crates_dir = config.workspace_root.join("crates");
        if !crates_dir.exists() {
            return Ok(violations);
        }

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
        let crate_names = [
            "mcb-domain",
            "mcb-application",
            "mcb-providers",
            "mcb-infrastructure",
            "mcb-server",
        ];

        for crate_name in &crate_names {
            let cargo_toml = crates_dir.join(crate_name).join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }
            let content = std::fs::read_to_string(&cargo_toml)?;
            let mut crate_deps = HashSet::new();
            for line in content.lines() {
                for dep_crate in &crate_names {
                    if *dep_crate != *crate_name && line.contains(*dep_crate) {
                        crate_deps.insert((*dep_crate).to_string());
                    }
                }
            }
            deps.insert((*crate_name).to_string(), crate_deps);
        }

        let crate_list: Vec<_> = deps.keys().cloned().collect();
        for (i, crate_a) in crate_list.iter().enumerate() {
            for crate_b in crate_list.iter().skip(i + 1) {
                let a_deps_b = deps.get(crate_a).is_some_and(|d| d.contains(crate_b));
                let b_deps_a = deps.get(crate_b).is_some_and(|d| d.contains(crate_a));
                if a_deps_b && b_deps_a {
                    violations.push(LayerFlowViolation::CircularDependency {
                        crate_a: crate_a.clone(),
                        crate_b: crate_b.clone(),
                        file: crates_dir.join(crate_a).join("Cargo.toml"),
                        line: 1,
                    });
                }
            }
        }
        Ok(violations)
    }
}

impl crate::validator_trait::Validator for LayerFlowValidator {
    fn name(&self) -> &'static str {
        "layer_flow"
    }

    fn description(&self) -> &'static str {
        "Validates Clean Architecture layer dependency rules"
    }

    fn validate(&self, config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_rules() {
        let rules = LayerRules::default();
        assert!(rules.forbidden["mcb-domain"].contains("mcb-providers"));
        assert!(rules.forbidden["mcb-server"].contains("mcb-providers"));
    }
}
