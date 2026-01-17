//! Async Pattern Validation
//!
//! Detects async-specific anti-patterns based on Tokio documentation:
//! - Blocking in async (std::thread::sleep, std::sync::Mutex in async)
//! - block_on() in async context
//! - Spawn patterns (missing JoinHandle handling)
//! - Wrong mutex types in async code

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Async violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncViolation {
    /// Blocking call in async function
    BlockingInAsync {
        file: PathBuf,
        line: usize,
        blocking_call: String,
        suggestion: String,
        severity: Severity,
    },
    /// block_on() used in async context
    BlockOnInAsync {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
    /// std::sync::Mutex used in async code (should use tokio::sync::Mutex)
    WrongMutexType {
        file: PathBuf,
        line: usize,
        mutex_type: String,
        suggestion: String,
        severity: Severity,
    },
    /// Spawn without awaiting JoinHandle
    UnawaitedSpawn {
        file: PathBuf,
        line: usize,
        context: String,
        severity: Severity,
    },
}

impl AsyncViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::BlockingInAsync { severity, .. } => *severity,
            Self::BlockOnInAsync { severity, .. } => *severity,
            Self::WrongMutexType { severity, .. } => *severity,
            Self::UnawaitedSpawn { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for AsyncViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlockingInAsync {
                file,
                line,
                blocking_call,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Blocking in async: {}:{} - {} ({})",
                    file.display(),
                    line,
                    blocking_call,
                    suggestion
                )
            }
            Self::BlockOnInAsync {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "block_on in async: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::WrongMutexType {
                file,
                line,
                mutex_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Wrong mutex type: {}:{} - {} ({})",
                    file.display(),
                    line,
                    mutex_type,
                    suggestion
                )
            }
            Self::UnawaitedSpawn {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "Unawaited spawn: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
        }
    }
}

impl Violation for AsyncViolation {
    fn id(&self) -> &str {
        match self {
            Self::BlockingInAsync { .. } => "ASYNC001",
            Self::BlockOnInAsync { .. } => "ASYNC002",
            Self::WrongMutexType { .. } => "ASYNC003",
            Self::UnawaitedSpawn { .. } => "ASYNC004",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Async
    }

    fn severity(&self) -> Severity {
        self.severity()
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::BlockingInAsync { file, .. } => Some(file),
            Self::BlockOnInAsync { file, .. } => Some(file),
            Self::WrongMutexType { file, .. } => Some(file),
            Self::UnawaitedSpawn { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::BlockingInAsync { line, .. } => Some(*line),
            Self::BlockOnInAsync { line, .. } => Some(*line),
            Self::WrongMutexType { line, .. } => Some(*line),
            Self::UnawaitedSpawn { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::BlockingInAsync { suggestion, .. } => Some(suggestion.clone()),
            Self::BlockOnInAsync { .. } => {
                Some("Use .await instead of block_on() in async context".to_string())
            }
            Self::WrongMutexType { suggestion, .. } => Some(suggestion.clone()),
            Self::UnawaitedSpawn { .. } => Some(
                "Assign JoinHandle to a variable or use let _ = to explicitly ignore".to_string(),
            ),
        }
    }
}

/// Async pattern validator
pub struct AsyncPatternValidator {
    config: ValidationConfig,
}

impl AsyncPatternValidator {
    /// Create a new async pattern validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all async validations
    pub fn validate_all(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_blocking_in_async()?);
        violations.extend(self.validate_block_on_usage()?);
        violations.extend(self.validate_mutex_types()?);
        violations.extend(self.validate_spawn_patterns()?);
        Ok(violations)
    }

    /// Detect blocking calls in async functions
    pub fn validate_blocking_in_async(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_fn_pattern = Regex::new(r"async\s+fn\s+(\w+)").expect("Invalid regex");

        let blocking_patterns = [
            (
                r"std::thread::sleep",
                "std::thread::sleep",
                "Use tokio::time::sleep instead",
            ),
            (
                r"thread::sleep",
                "thread::sleep",
                "Use tokio::time::sleep instead",
            ),
            (
                r"std::fs::read",
                "std::fs::read",
                "Use tokio::fs::read instead",
            ),
            (
                r"std::fs::write",
                "std::fs::write",
                "Use tokio::fs::write instead",
            ),
            (
                r"std::fs::File::open",
                "std::fs::File::open",
                "Use tokio::fs::File::open instead",
            ),
            (
                r"std::fs::File::create",
                "std::fs::File::create",
                "Use tokio::fs::File::create instead",
            ),
            (
                r"\.blocking_lock\(\)",
                ".blocking_lock()",
                "Use .lock().await instead in async context",
            ),
            (
                r"\.blocking_read\(\)",
                ".blocking_read()",
                "Use .read().await instead in async context",
            ),
            (
                r"\.blocking_write\(\)",
                ".blocking_write()",
                "Use .write().await instead in async context",
            ),
        ];

        let compiled_blocking: Vec<_> = blocking_patterns
            .iter()
            .filter_map(|(p, desc, sugg)| Regex::new(p).ok().map(|r| (r, *desc, *sugg)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_async_fn = false;
                let mut async_fn_depth = 0;
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

                    // Track async function entry
                    if async_fn_pattern.is_match(trimmed) {
                        in_async_fn = true;
                        async_fn_depth = 0;
                    }

                    // Track brace depth
                    if in_async_fn {
                        async_fn_depth += line.chars().filter(|c| *c == '{').count() as i32;
                        async_fn_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                        // Check for blocking calls
                        for (pattern, desc, sugg) in &compiled_blocking {
                            if pattern.is_match(line) {
                                violations.push(AsyncViolation::BlockingInAsync {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    blocking_call: desc.to_string(),
                                    suggestion: sugg.to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                        }

                        if async_fn_depth <= 0 {
                            in_async_fn = false;
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect block_on() usage in async context
    pub fn validate_block_on_usage(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_fn_pattern = Regex::new(r"async\s+fn\s+").expect("Invalid regex");
        let block_on_patterns = [
            r"block_on\(",
            r"futures::executor::block_on",
            r"tokio::runtime::Runtime::new\(\).*\.block_on",
            r"Runtime::new\(\).*\.block_on",
        ];

        let compiled_block_on: Vec<_> = block_on_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_async_fn = false;
                let mut async_fn_depth = 0;
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

                    // Track async function entry
                    if async_fn_pattern.is_match(trimmed) {
                        in_async_fn = true;
                        async_fn_depth = 0;
                    }

                    // Track brace depth
                    if in_async_fn {
                        async_fn_depth += line.chars().filter(|c| *c == '{').count() as i32;
                        async_fn_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                        // Check for block_on calls
                        for pattern in &compiled_block_on {
                            if pattern.is_match(line) {
                                violations.push(AsyncViolation::BlockOnInAsync {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    context: trimmed.chars().take(80).collect(),
                                    severity: Severity::Error,
                                });
                            }
                        }

                        if async_fn_depth <= 0 {
                            in_async_fn = false;
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect std::sync::Mutex usage in files with async code
    pub fn validate_mutex_types(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_indicator = Regex::new(r"async\s+fn|\.await").expect("Invalid regex");
        let std_mutex_patterns = [
            (
                r"use\s+std::sync::Mutex",
                "std::sync::Mutex import",
                "Use tokio::sync::Mutex for async code",
            ),
            (
                r"std::sync::Mutex<",
                "std::sync::Mutex type",
                "Use tokio::sync::Mutex for async code",
            ),
            (
                r"use\s+std::sync::RwLock",
                "std::sync::RwLock import",
                "Use tokio::sync::RwLock for async code",
            ),
            (
                r"std::sync::RwLock<",
                "std::sync::RwLock type",
                "Use tokio::sync::RwLock for async code",
            ),
        ];

        let compiled_mutex: Vec<_> = std_mutex_patterns
            .iter()
            .filter_map(|(p, desc, sugg)| Regex::new(p).ok().map(|r| (r, *desc, *sugg)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;

                // Only check files that have async code
                if !async_indicator.is_match(&content) {
                    continue;
                }

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

                    // Check for std mutex patterns
                    for (pattern, desc, sugg) in &compiled_mutex {
                        if pattern.is_match(line) {
                            violations.push(AsyncViolation::WrongMutexType {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                mutex_type: desc.to_string(),
                                suggestion: sugg.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect spawn without await patterns
    pub fn validate_spawn_patterns(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        // Pattern: tokio::spawn without assigning to variable or awaiting
        let spawn_pattern = Regex::new(r"tokio::spawn\s*\(").expect("Invalid regex");
        let assigned_spawn_pattern =
            Regex::new(r"let\s+\w+\s*=\s*tokio::spawn").expect("Invalid regex");
        let fn_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").expect("Invalid regex");

        // Function name patterns that indicate intentional fire-and-forget spawns
        // Includes constructor patterns that often spawn background workers
        let background_fn_patterns = [
            "spawn",
            "background",
            "graceful",
            "shutdown",
            "start",
            "run",
            "worker",
            "daemon",
            "listener",
            "handler",
            "process",
            "new",
            "with_",
            "init",
            "create",
            "build", // Constructor patterns
        ];

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;
                let mut current_fn_name = String::new();

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

                    // Track current function name
                    if let Some(cap) = fn_pattern.captures(line) {
                        current_fn_name =
                            cap.get(1).map(|m| m.as_str()).unwrap_or("").to_lowercase();
                    }

                    // Check for unassigned spawn
                    if spawn_pattern.is_match(line) && !assigned_spawn_pattern.is_match(line) {
                        // Check if it's being used in a chain (e.g., .await)
                        if !line.contains(".await") && !line.contains("let _") {
                            // Skip if function name suggests fire-and-forget is intentional
                            let is_background_fn = background_fn_patterns
                                .iter()
                                .any(|p| current_fn_name.contains(p));
                            if is_background_fn {
                                continue;
                            }
                            violations.push(AsyncViolation::UnawaitedSpawn {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                context: trimmed.chars().take(80).collect(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for AsyncPatternValidator {
    fn name(&self) -> &'static str {
        "async_patterns"
    }

    fn description(&self) -> &'static str {
        "Validates async patterns (blocking calls, mutex types, spawn patterns)"
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
    }

    #[test]
    fn test_blocking_in_async_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
async fn bad_function() {
    std::thread::sleep(std::time::Duration::from_secs(1));
}
"#,
        );

        let validator = AsyncPatternValidator::new(temp.path());
        let violations = validator.validate_blocking_in_async().unwrap();

        assert!(!violations.is_empty(), "Should detect blocking in async");
    }

    #[test]
    fn test_wrong_mutex_type_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
use std::sync::Mutex;

async fn use_mutex() {
    let m = Mutex::new(0);
    let guard = m.lock().await;
}
"#,
        );

        let validator = AsyncPatternValidator::new(temp.path());
        let violations = validator.validate_mutex_types().unwrap();

        assert!(
            !violations.is_empty(),
            "Should detect std::sync::Mutex in async code"
        );
    }

    #[test]
    fn test_no_async_file_exempt() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
use std::sync::Mutex;

fn sync_function() {
    let m = Mutex::new(0);
    let guard = m.lock().unwrap();
}
"#,
        );

        let validator = AsyncPatternValidator::new(temp.path());
        let violations = validator.validate_mutex_types().unwrap();

        assert!(
            violations.is_empty(),
            "Files without async code should be exempt: {:?}",
            violations
        );
    }

    #[test]
    fn test_test_module_exemption() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub async fn good_code() {}

#[cfg(test)]
mod tests {
    async fn test_blocking() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
"#,
        );

        let validator = AsyncPatternValidator::new(temp.path());
        let violations = validator.validate_blocking_in_async().unwrap();

        assert!(
            violations.is_empty(),
            "Test modules should be exempt: {:?}",
            violations
        );
    }
}
