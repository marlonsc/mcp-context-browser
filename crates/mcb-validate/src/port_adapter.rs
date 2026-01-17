//! Port/Adapter Compliance Validation
//!
//! Validates Clean Architecture port/adapter patterns.

use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationConfig};
use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Port/Adapter Violations
#[derive(Debug, Clone, Serialize)]
pub enum PortAdapterViolation {
    AdapterMissingPortImpl {
        adapter_name: String,
        file: PathBuf,
        line: usize,
    },
    AdapterUsesAdapter {
        adapter_name: String,
        other_adapter: String,
        file: PathBuf,
        line: usize,
    },
    PortTooLarge {
        trait_name: String,
        method_count: usize,
        file: PathBuf,
        line: usize,
    },
    PortTooSmall {
        trait_name: String,
        method_count: usize,
        file: PathBuf,
        line: usize,
    },
}

impl std::fmt::Display for PortAdapterViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdapterMissingPortImpl {
                adapter_name,
                file,
                line,
            } => write!(
                f,
                "Adapter {} missing port impl at {}:{}",
                adapter_name,
                file.display(),
                line
            ),
            Self::AdapterUsesAdapter {
                adapter_name,
                other_adapter,
                file,
                line,
            } => write!(
                f,
                "Adapter {} uses {} directly at {}:{}",
                adapter_name,
                other_adapter,
                file.display(),
                line
            ),
            Self::PortTooLarge {
                trait_name,
                method_count,
                file,
                line,
            } => write!(
                f,
                "Port {} has {} methods (>10) at {}:{}",
                trait_name,
                method_count,
                file.display(),
                line
            ),
            Self::PortTooSmall {
                trait_name,
                method_count,
                file,
                line,
            } => write!(
                f,
                "Port {} has {} method(s) at {}:{}",
                trait_name,
                method_count,
                file.display(),
                line
            ),
        }
    }
}

impl Violation for PortAdapterViolation {
    fn id(&self) -> &str {
        match self {
            Self::AdapterMissingPortImpl { .. } => "PORT001",
            Self::AdapterUsesAdapter { .. } => "PORT002",
            Self::PortTooLarge { .. } => "PORT003",
            Self::PortTooSmall { .. } => "PORT004",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::AdapterMissingPortImpl { .. } | Self::AdapterUsesAdapter { .. } => {
                Severity::Warning
            }
            Self::PortTooLarge { .. } | Self::PortTooSmall { .. } => Severity::Info,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::AdapterMissingPortImpl { file, .. }
            | Self::AdapterUsesAdapter { file, .. }
            | Self::PortTooLarge { file, .. }
            | Self::PortTooSmall { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::AdapterMissingPortImpl { line, .. }
            | Self::AdapterUsesAdapter { line, .. }
            | Self::PortTooLarge { line, .. }
            | Self::PortTooSmall { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::AdapterMissingPortImpl { .. } => {
                Some("Implement a port trait from mcb-application/ports/".to_string())
            }
            Self::AdapterUsesAdapter { .. } => {
                Some("Depend on port traits, not concrete adapters".to_string())
            }
            Self::PortTooLarge { .. } => {
                Some("Consider splitting into smaller interfaces (ISP)".to_string())
            }
            Self::PortTooSmall { .. } => Some("May indicate over-fragmentation".to_string()),
        }
    }
}

/// Port/Adapter Compliance Validator
pub struct PortAdapterValidator;

impl Default for PortAdapterValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl PortAdapterValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, config: &ValidationConfig) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.check_port_trait_sizes(config)?);
        violations.extend(self.check_adapter_direct_usage(config)?);
        Ok(violations)
    }

    fn check_port_trait_sizes(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        let ports_dir = config
            .workspace_root
            .join("crates/mcb-application/src/ports");
        if !ports_dir.exists() {
            return Ok(violations);
        }

        let trait_start_re = Regex::new(r"pub\s+trait\s+(\w+)").expect("Invalid regex");
        let fn_re = Regex::new(r"^\s*(?:async\s+)?fn\s+\w+").expect("Invalid regex");

        for entry in WalkDir::new(&ports_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut current_trait: Option<(String, usize, usize)> = None;
            let mut brace_depth = 0;
            let mut in_trait = false;

            for (line_num, line) in lines.iter().enumerate() {
                if let Some(captures) = trait_start_re.captures(line) {
                    let trait_name = captures.get(1).map(|m| m.as_str().to_string()).unwrap();
                    current_trait = Some((trait_name, line_num + 1, 0));
                    in_trait = true;
                }

                if in_trait {
                    brace_depth += line.matches('{').count();
                    brace_depth -= line.matches('}').count();

                    if fn_re.is_match(line) {
                        if let Some((_, _, ref mut count)) = current_trait {
                            *count += 1;
                        }
                    }

                    if brace_depth == 0 && current_trait.is_some() {
                        let (trait_name, start_line, method_count) = current_trait.take().unwrap();
                        in_trait = false;

                        if method_count > 10 {
                            violations.push(PortAdapterViolation::PortTooLarge {
                                trait_name,
                                method_count,
                                file: path.to_path_buf(),
                                line: start_line,
                            });
                        } else if method_count < 2 && method_count > 0 {
                            violations.push(PortAdapterViolation::PortTooSmall {
                                trait_name,
                                method_count,
                                file: path.to_path_buf(),
                                line: start_line,
                            });
                        }
                    }
                }
            }
        }
        Ok(violations)
    }

    fn check_adapter_direct_usage(
        &self,
        config: &ValidationConfig,
    ) -> Result<Vec<PortAdapterViolation>> {
        let mut violations = Vec::new();
        let providers_dir = config.workspace_root.join("crates/mcb-providers/src");
        if !providers_dir.exists() {
            return Ok(violations);
        }

        let adapter_suffixes = ["Provider", "Repository", "Adapter", "Client"];
        let adapter_import_re = Regex::new(
            r"use\s+(?:crate|super)::(?:\w+::)*(\w+(?:Provider|Repository|Adapter|Client))",
        )
        .expect("Invalid regex");

        for entry in WalkDir::new(&providers_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "mod.rs" || file_name == "lib.rs" {
                continue;
            }

            let content = std::fs::read_to_string(path)?;
            let current_adapter = file_name.trim_end_matches(".rs");

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                if let Some(captures) = adapter_import_re.captures(line) {
                    let imported = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                    if imported.to_lowercase().contains(current_adapter) {
                        continue;
                    }

                    for suffix in &adapter_suffixes {
                        if imported.ends_with(suffix) && !imported.starts_with("dyn") {
                            violations.push(PortAdapterViolation::AdapterUsesAdapter {
                                adapter_name: current_adapter.to_string(),
                                other_adapter: imported.to_string(),
                                file: path.to_path_buf(),
                                line: line_num + 1,
                            });
                            break;
                        }
                    }
                }
            }
        }
        Ok(violations)
    }
}

impl crate::validator_trait::Validator for PortAdapterValidator {
    fn name(&self) -> &'static str {
        "port_adapter"
    }

    fn description(&self) -> &'static str {
        "Validates port/adapter patterns for Clean Architecture compliance"
    }

    fn validate(
        &self,
        config: &ValidationConfig,
    ) -> anyhow::Result<Vec<Box<dyn crate::violation_trait::Violation>>> {
        let violations = self.validate(config)?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn crate::violation_trait::Violation>)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_pattern() {
        let re = Regex::new(r"pub\s+trait\s+(\w+)").unwrap();
        assert!(re.is_match("pub trait EmbeddingProvider {"));
    }
}
