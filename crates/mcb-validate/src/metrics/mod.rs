//! Code Metrics Module
//!
//! Provides code complexity metrics analysis.
//!
//! ## Supported Metrics
//!
//! - **Cognitive Complexity**: Measures how difficult code is to understand
//! - **Cyclomatic Complexity**: Number of linearly independent paths
//! - **Function Length**: Lines of code in a function
//! - **Nesting Depth**: Maximum nesting level
//! - **Halstead metrics**: Volume, Difficulty, Effort (via rust-code-analysis)
//! - **Maintainability Index**: Overall maintainability score (via rust-code-analysis)
//! - **LOC metrics**: SLOC, PLOC, LLOC, CLOC, BLANK (via rust-code-analysis)
//!
//! ## Supported Languages
//!
//! - Rust (via syn-based MetricsAnalyzer)
//! - Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Kotlin (via RcaAnalyzer)

mod rca_analyzer;
mod thresholds;

pub use rca_analyzer::{RcaAnalyzer, RcaFunctionMetrics, RcaMetrics};
pub use thresholds::{MetricThreshold, MetricThresholds, MetricType};

use crate::violation_trait::{Severity, Violation, ViolationCategory};
use crate::{Result, ValidationError};
use std::path::{Path, PathBuf};
// Note: complexity crate uses syn 1.x, but we use syn 2.x.
// We implement a simplified cognitive complexity calculation.

/// A metric violation when a threshold is exceeded
#[derive(Debug, Clone)]
pub struct MetricViolation {
    /// File path
    pub file: PathBuf,
    /// Line number where the function/item starts
    pub line: usize,
    /// Name of the function/item
    pub item_name: String,
    /// Type of metric that was exceeded
    pub metric_type: MetricType,
    /// Actual value measured
    pub actual_value: u32,
    /// Configured threshold
    pub threshold: u32,
    /// Severity level
    pub severity: Severity,
}

impl std::fmt::Display for MetricViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} `{}` has {} of {} (threshold: {}) in {}",
            self.id(),
            self.metric_type.name(),
            self.item_name,
            self.metric_type.description(),
            self.actual_value,
            self.threshold,
            self.file.display()
        )
    }
}

impl Violation for MetricViolation {
    fn id(&self) -> &str {
        match self.metric_type {
            MetricType::CognitiveComplexity => "METRIC001",
            MetricType::CyclomaticComplexity => "METRIC004",
            MetricType::FunctionLength => "METRIC002",
            MetricType::NestingDepth => "METRIC003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&PathBuf> {
        Some(&self.file)
    }

    fn line(&self) -> Option<usize> {
        Some(self.line)
    }

    fn message(&self) -> String {
        format!(
            "{} `{}` has {} of {} (threshold: {})",
            self.metric_type.name(),
            self.item_name,
            self.metric_type.description(),
            self.actual_value,
            self.threshold
        )
    }

    fn suggestion(&self) -> Option<String> {
        Some(self.metric_type.suggestion())
    }
}

/// Metrics analyzer for code complexity analysis
pub struct MetricsAnalyzer {
    thresholds: MetricThresholds,
}

impl Default for MetricsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsAnalyzer {
    /// Create a new metrics analyzer with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: MetricThresholds::default(),
        }
    }

    /// Create a new metrics analyzer with custom thresholds
    pub fn with_thresholds(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze a Rust file for cognitive complexity
    pub fn analyze_rust_file(&self, path: &Path) -> Result<Vec<MetricViolation>> {
        let content = std::fs::read_to_string(path).map_err(|e| ValidationError::Io(e))?;

        self.analyze_rust_content(&content, path)
    }

    /// Analyze Rust content for cognitive complexity
    pub fn analyze_rust_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<MetricViolation>> {
        let mut violations = Vec::new();

        // Parse the Rust file
        let syntax = syn::parse_file(content).map_err(|e| ValidationError::Parse {
            file: file_path.to_path_buf(),
            message: e.to_string(),
        })?;

        // Analyze each function
        for item in &syntax.items {
            self.analyze_item(item, file_path, &mut violations);
        }

        Ok(violations)
    }

    /// Analyze a single item (function, impl block, etc.)
    fn analyze_item(
        &self,
        item: &syn::Item,
        file_path: &Path,
        violations: &mut Vec<MetricViolation>,
    ) {
        match item {
            syn::Item::Fn(func) => {
                let name = func.sig.ident.to_string();
                // Get line number from span (fallback to 1 if not available)
                let line = 1; // Line number not easily extractable without proc-macro2 span locations

                // Calculate cognitive complexity using simplified algorithm
                let complexity = self.calculate_cognitive_complexity(&func.block);

                // Check threshold
                if let Some(threshold) = self.thresholds.get(MetricType::CognitiveComplexity) {
                    if complexity as u32 > threshold.max_value {
                        violations.push(MetricViolation {
                            file: file_path.to_path_buf(),
                            line,
                            item_name: name.clone(),
                            metric_type: MetricType::CognitiveComplexity,
                            actual_value: complexity as u32,
                            threshold: threshold.max_value,
                            severity: threshold.severity,
                        });
                    }
                }

                // Calculate function length (lines)
                let func_length = self.estimate_function_length(&func.block);
                if let Some(threshold) = self.thresholds.get(MetricType::FunctionLength) {
                    if func_length > threshold.max_value {
                        violations.push(MetricViolation {
                            file: file_path.to_path_buf(),
                            line,
                            item_name: name.clone(),
                            metric_type: MetricType::FunctionLength,
                            actual_value: func_length,
                            threshold: threshold.max_value,
                            severity: threshold.severity,
                        });
                    }
                }

                // Calculate nesting depth
                let nesting = self.calculate_nesting_depth(&func.block);
                if let Some(threshold) = self.thresholds.get(MetricType::NestingDepth) {
                    if nesting > threshold.max_value {
                        violations.push(MetricViolation {
                            file: file_path.to_path_buf(),
                            line,
                            item_name: name,
                            metric_type: MetricType::NestingDepth,
                            actual_value: nesting,
                            threshold: threshold.max_value,
                            severity: threshold.severity,
                        });
                    }
                }
            }
            syn::Item::Impl(impl_block) => {
                // Analyze methods in impl blocks
                for item in &impl_block.items {
                    if let syn::ImplItem::Fn(method) = item {
                        let name = method.sig.ident.to_string();
                        let line = 1; // Line number not easily extractable without proc-macro2 span locations

                        let complexity = self.calculate_cognitive_complexity(&method.block);

                        if let Some(threshold) =
                            self.thresholds.get(MetricType::CognitiveComplexity)
                        {
                            if complexity as u32 > threshold.max_value {
                                violations.push(MetricViolation {
                                    file: file_path.to_path_buf(),
                                    line,
                                    item_name: name.clone(),
                                    metric_type: MetricType::CognitiveComplexity,
                                    actual_value: complexity as u32,
                                    threshold: threshold.max_value,
                                    severity: threshold.severity,
                                });
                            }
                        }

                        let func_length = self.estimate_function_length(&method.block);
                        if let Some(threshold) = self.thresholds.get(MetricType::FunctionLength) {
                            if func_length > threshold.max_value {
                                violations.push(MetricViolation {
                                    file: file_path.to_path_buf(),
                                    line,
                                    item_name: name.clone(),
                                    metric_type: MetricType::FunctionLength,
                                    actual_value: func_length,
                                    threshold: threshold.max_value,
                                    severity: threshold.severity,
                                });
                            }
                        }

                        let nesting = self.calculate_nesting_depth(&method.block);
                        if let Some(threshold) = self.thresholds.get(MetricType::NestingDepth) {
                            if nesting > threshold.max_value {
                                violations.push(MetricViolation {
                                    file: file_path.to_path_buf(),
                                    line,
                                    item_name: name,
                                    metric_type: MetricType::NestingDepth,
                                    actual_value: nesting,
                                    threshold: threshold.max_value,
                                    severity: threshold.severity,
                                });
                            }
                        }
                    }
                }
            }
            syn::Item::Mod(module) => {
                // Recursively analyze inline modules
                if let Some((_, items)) = &module.content {
                    for item in items {
                        self.analyze_item(item, file_path, violations);
                    }
                }
            }
            _ => {}
        }
    }

    /// Estimate function length by counting statements
    fn estimate_function_length(&self, block: &syn::Block) -> u32 {
        // Simple heuristic: count statements
        block.stmts.len() as u32
    }

    /// Calculate simplified cognitive complexity for a block
    ///
    /// This is a simplified implementation based on counting control flow structures.
    /// Each if/else/for/while/loop/match adds 1, plus nesting depth penalty.
    fn calculate_cognitive_complexity(&self, block: &syn::Block) -> u32 {
        self.block_complexity(block, 0)
    }

    /// Calculate complexity for a block at a given nesting level
    fn block_complexity(&self, block: &syn::Block, nesting: u32) -> u32 {
        let mut total = 0u32;
        for stmt in &block.stmts {
            total += self.stmt_complexity(stmt, nesting);
        }
        total
    }

    /// Calculate complexity contribution from a statement
    fn stmt_complexity(&self, stmt: &syn::Stmt, nesting: u32) -> u32 {
        match stmt {
            syn::Stmt::Expr(expr, _) => self.expr_complexity(expr, nesting),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    self.expr_complexity(&init.expr, nesting)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Calculate complexity contribution from an expression
    fn expr_complexity(&self, expr: &syn::Expr, nesting: u32) -> u32 {
        match expr {
            syn::Expr::If(if_expr) => {
                // +1 for if, +nesting for nested if
                let mut complexity = 1 + nesting;
                // Process then branch
                complexity += self.block_complexity(&if_expr.then_branch, nesting + 1);
                // Process else branch
                if let Some((_, else_expr)) = &if_expr.else_branch {
                    complexity += 1; // +1 for else
                    complexity += self.expr_complexity(else_expr, nesting + 1);
                }
                complexity
            }
            syn::Expr::While(while_expr) => {
                // +1 for while, +nesting for nested while
                1 + nesting + self.block_complexity(&while_expr.body, nesting + 1)
            }
            syn::Expr::ForLoop(for_expr) => {
                // +1 for for, +nesting for nested for
                1 + nesting + self.block_complexity(&for_expr.body, nesting + 1)
            }
            syn::Expr::Loop(loop_expr) => {
                // +1 for loop, +nesting for nested loop
                1 + nesting + self.block_complexity(&loop_expr.body, nesting + 1)
            }
            syn::Expr::Match(match_expr) => {
                // +1 for match (not per-arm)
                let mut complexity = 1;
                for arm in &match_expr.arms {
                    complexity += self.expr_complexity(&arm.body, nesting + 1);
                }
                complexity
            }
            syn::Expr::Block(block_expr) => self.block_complexity(&block_expr.block, nesting),
            syn::Expr::Closure(closure) => {
                // +1 for closure
                1 + self.expr_complexity(&closure.body, nesting + 1)
            }
            syn::Expr::Binary(binary) => {
                // +1 for && or || (logical operators add complexity)
                let op_complexity = match binary.op {
                    syn::BinOp::And(_) | syn::BinOp::Or(_) => 1,
                    _ => 0,
                };
                op_complexity
                    + self.expr_complexity(&binary.left, nesting)
                    + self.expr_complexity(&binary.right, nesting)
            }
            syn::Expr::Break(_) | syn::Expr::Continue(_) => {
                // +1 for break/continue (flow-breaking)
                1
            }
            _ => 0,
        }
    }

    /// Calculate maximum nesting depth in a block
    fn calculate_nesting_depth(&self, block: &syn::Block) -> u32 {
        let mut max_depth = 0u32;

        for stmt in &block.stmts {
            let depth = self.stmt_nesting_depth(stmt, 1);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Calculate nesting depth for a statement
    fn stmt_nesting_depth(&self, stmt: &syn::Stmt, current_depth: u32) -> u32 {
        match stmt {
            syn::Stmt::Expr(expr, _) => self.expr_nesting_depth(expr, current_depth),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    self.expr_nesting_depth(&init.expr, current_depth)
                } else {
                    current_depth
                }
            }
            _ => current_depth,
        }
    }

    /// Calculate nesting depth for an expression
    fn expr_nesting_depth(&self, expr: &syn::Expr, current_depth: u32) -> u32 {
        match expr {
            syn::Expr::If(if_expr) => {
                let then_depth = self.block_nesting_depth(&if_expr.then_branch, current_depth + 1);
                let else_depth = if_expr
                    .else_branch
                    .as_ref()
                    .map_or(current_depth, |(_, else_expr)| {
                        self.expr_nesting_depth(else_expr, current_depth + 1)
                    });
                then_depth.max(else_depth)
            }
            syn::Expr::While(while_expr) => {
                self.block_nesting_depth(&while_expr.body, current_depth + 1)
            }
            syn::Expr::ForLoop(for_expr) => {
                self.block_nesting_depth(&for_expr.body, current_depth + 1)
            }
            syn::Expr::Loop(loop_expr) => {
                self.block_nesting_depth(&loop_expr.body, current_depth + 1)
            }
            syn::Expr::Match(match_expr) => {
                let mut max_depth = current_depth;
                for arm in &match_expr.arms {
                    let arm_depth = self.expr_nesting_depth(&arm.body, current_depth + 1);
                    max_depth = max_depth.max(arm_depth);
                }
                max_depth
            }
            syn::Expr::Block(block_expr) => {
                self.block_nesting_depth(&block_expr.block, current_depth)
            }
            syn::Expr::Closure(closure) => {
                self.expr_nesting_depth(&closure.body, current_depth + 1)
            }
            _ => current_depth,
        }
    }

    /// Calculate max nesting depth in a block
    fn block_nesting_depth(&self, block: &syn::Block, current_depth: u32) -> u32 {
        let mut max_depth = current_depth;
        for stmt in &block.stmts {
            let depth = self.stmt_nesting_depth(stmt, current_depth);
            max_depth = max_depth.max(depth);
        }
        max_depth
    }

    /// Analyze multiple files
    pub fn analyze_files(&self, paths: &[PathBuf]) -> Result<Vec<MetricViolation>> {
        let mut all_violations = Vec::new();

        for path in paths {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    match self.analyze_rust_file(path) {
                        Ok(violations) => all_violations.extend(violations),
                        Err(e) => {
                            // Log parse errors but continue
                            eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
                        }
                    }
                }
                // TODO: Add support for other languages via Tree-sitter
            }
        }

        Ok(all_violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function_analysis() {
        let content = r#"
fn simple() {
    let x = 1;
    let y = 2;
}
"#;
        let analyzer = MetricsAnalyzer::new();
        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        // Simple function should have low complexity
        assert!(
            violations.is_empty(),
            "Simple function should have no violations"
        );
    }

    #[test]
    fn test_complex_function_detection() {
        let content = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                if x > 1000 {
                    for i in 0..x {
                        match i {
                            0 => return 0,
                            1 => return 1,
                            2 => return 2,
                            _ => continue,
                        }
                    }
                }
            }
        }
    }
    x
}
"#;
        // Use lower thresholds to trigger violations
        let thresholds = MetricThresholds::new()
            .with_threshold(MetricType::CognitiveComplexity, 5, Severity::Warning)
            .with_threshold(MetricType::NestingDepth, 3, Severity::Warning);

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        // Complex function should trigger violations
        assert!(
            !violations.is_empty(),
            "Complex function should have violations"
        );
    }

    #[test]
    fn test_nesting_depth_calculation() {
        let content = r#"
fn nested() {
    if true {
        if true {
            if true {
                println!("deep");
            }
        }
    }
}
"#;
        let thresholds =
            MetricThresholds::new().with_threshold(MetricType::NestingDepth, 2, Severity::Warning);

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        // Should detect deep nesting
        let nesting_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::NestingDepth)
            .collect();

        assert!(
            !nesting_violations.is_empty(),
            "Should detect nesting depth violation"
        );
    }

    #[test]
    fn test_impl_method_analysis() {
        let content = r#"
struct Foo;

impl Foo {
    fn simple(&self) {
        let x = 1;
    }

    fn complex(&self, x: i32) -> i32 {
        if x > 0 {
            if x > 10 {
                if x > 100 {
                    return x * 2;
                }
            }
        }
        x
    }
}
"#;
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = MetricsAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_rust_content(content, Path::new("test.rs"))
            .unwrap();

        // Should find the complex method but not the simple one
        assert!(!violations.is_empty(), "Should detect complex method");
        assert!(violations.iter().any(|v| v.item_name == "complex"));
    }
}
