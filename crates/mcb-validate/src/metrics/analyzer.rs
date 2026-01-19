//! Tree-sitter based metrics analyzer for multiple languages
//!
//! Provides code metrics analysis using Tree-sitter parsers for:
//! - Rust, Python, JavaScript, TypeScript, Go, Java, C, C++
//!
//! Metrics supported:
//! - Cognitive Complexity
//! - Cyclomatic Complexity
//! - Function Length
//! - Nesting Depth

use crate::{Result, ValidationError};
use std::path::{Path, PathBuf};

use super::MetricViolation;
use super::thresholds::{MetricThresholds, MetricType};

/// Supported languages for metrics analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    C,
    Cpp,
}

impl MetricsLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Self::Cpp),
            _ => None,
        }
    }

    /// Get tree-sitter language for this language
    pub fn tree_sitter_language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        }
    }

    /// Get function node types for this language
    fn function_node_types(&self) -> &[&str] {
        match self {
            Self::Rust => &["function_item", "impl_item"],
            Self::Python => &["function_definition", "async_function_definition"],
            Self::JavaScript | Self::TypeScript => &[
                "function_declaration",
                "function_expression",
                "arrow_function",
                "method_definition",
            ],
            Self::Go => &["function_declaration", "method_declaration"],
            Self::Java => &["method_declaration", "constructor_declaration"],
            Self::C | Self::Cpp => &["function_definition"],
        }
    }

    /// Get control flow node types for complexity calculation
    fn control_flow_types(&self) -> &[&str] {
        match self {
            Self::Rust => &[
                "if_expression",
                "while_expression",
                "for_expression",
                "loop_expression",
                "match_expression",
            ],
            Self::Python => &[
                "if_statement",
                "while_statement",
                "for_statement",
                "try_statement",
                "with_statement",
                "match_statement",
            ],
            Self::JavaScript | Self::TypeScript => &[
                "if_statement",
                "while_statement",
                "for_statement",
                "for_in_statement",
                "do_statement",
                "switch_statement",
                "try_statement",
                "catch_clause",
                "ternary_expression",
            ],
            Self::Go => &[
                "if_statement",
                "for_statement",
                "switch_statement",
                "select_statement",
                "type_switch_statement",
            ],
            Self::Java => &[
                "if_statement",
                "while_statement",
                "for_statement",
                "enhanced_for_statement",
                "do_statement",
                "switch_expression",
                "try_statement",
                "catch_clause",
                "ternary_expression",
            ],
            Self::C | Self::Cpp => &[
                "if_statement",
                "while_statement",
                "for_statement",
                "do_statement",
                "switch_statement",
                "case_statement",
            ],
        }
    }

    /// Get logical operator types for cyclomatic complexity
    fn logical_operator_types(&self) -> &[&str] {
        match self {
            Self::Rust => &["binary_expression"], // Check for && and ||
            Self::Python => &["boolean_operator"],
            Self::JavaScript | Self::TypeScript => &["binary_expression"],
            Self::Go => &["binary_expression"],
            Self::Java => &["binary_expression"],
            Self::C | Self::Cpp => &["binary_expression"],
        }
    }
}

/// Function metrics extracted from AST
#[derive(Debug, Clone)]
pub struct FunctionMetrics {
    /// Function name
    pub name: String,
    /// Start line (1-indexed)
    pub line: usize,
    /// Cognitive complexity score
    pub cognitive_complexity: u32,
    /// Cyclomatic complexity score
    pub cyclomatic_complexity: u32,
    /// Number of lines/statements
    pub length: u32,
    /// Maximum nesting depth
    pub nesting_depth: u32,
}

/// Tree-sitter based metrics analyzer
pub struct TreeSitterAnalyzer {
    thresholds: MetricThresholds,
}

impl Default for TreeSitterAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeSitterAnalyzer {
    /// Create analyzer with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: MetricThresholds::default(),
        }
    }

    /// Create analyzer with custom thresholds
    pub fn with_thresholds(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    /// Analyze a file and return violations
    pub fn analyze_file(&self, path: &Path) -> Result<Vec<MetricViolation>> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| ValidationError::Config("File has no extension".to_string()))?;

        let language = MetricsLanguage::from_extension(ext).ok_or_else(|| {
            ValidationError::Config(format!("Unsupported language for extension: {}", ext))
        })?;

        let content = std::fs::read_to_string(path).map_err(ValidationError::Io)?;

        self.analyze_content(&content, path, language)
    }

    /// Analyze content with specified language
    pub fn analyze_content(
        &self,
        content: &str,
        file_path: &Path,
        language: MetricsLanguage,
    ) -> Result<Vec<MetricViolation>> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language.tree_sitter_language())
            .map_err(|e| ValidationError::Config(format!("Failed to set language: {}", e)))?;

        let tree = parser
            .parse(content, None)
            .ok_or_else(|| ValidationError::Parse {
                file: file_path.to_path_buf(),
                message: "Failed to parse file".to_string(),
            })?;

        let root = tree.root_node();
        let mut violations = Vec::new();

        // Find all functions and analyze them
        let functions = self.find_functions(&root, content, language);

        for func in functions {
            // Check cognitive complexity
            if let Some(threshold) = self.thresholds.get(MetricType::CognitiveComplexity) {
                if func.cognitive_complexity > threshold.max_value {
                    violations.push(MetricViolation {
                        file: file_path.to_path_buf(),
                        line: func.line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::CognitiveComplexity,
                        actual_value: func.cognitive_complexity,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }

            // Check cyclomatic complexity
            if let Some(threshold) = self.thresholds.get(MetricType::CyclomaticComplexity) {
                if func.cyclomatic_complexity > threshold.max_value {
                    violations.push(MetricViolation {
                        file: file_path.to_path_buf(),
                        line: func.line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::CyclomaticComplexity,
                        actual_value: func.cyclomatic_complexity,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }

            // Check function length
            if let Some(threshold) = self.thresholds.get(MetricType::FunctionLength) {
                if func.length > threshold.max_value {
                    violations.push(MetricViolation {
                        file: file_path.to_path_buf(),
                        line: func.line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::FunctionLength,
                        actual_value: func.length,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }

            // Check nesting depth
            if let Some(threshold) = self.thresholds.get(MetricType::NestingDepth) {
                if func.nesting_depth > threshold.max_value {
                    violations.push(MetricViolation {
                        file: file_path.to_path_buf(),
                        line: func.line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::NestingDepth,
                        actual_value: func.nesting_depth,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Find all functions in the AST
    fn find_functions(
        &self,
        root: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
    ) -> Vec<FunctionMetrics> {
        let mut functions = Vec::new();
        let function_types = language.function_node_types();

        self.visit_node(root, content, language, function_types, &mut functions);

        functions
    }

    /// Recursively visit nodes to find functions
    fn visit_node(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
        function_types: &[&str],
        functions: &mut Vec<FunctionMetrics>,
    ) {
        let node_type = node.kind();

        if function_types.contains(&node_type) {
            if let Some(metrics) = self.analyze_function(node, content, language) {
                functions.push(metrics);
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, content, language, function_types, functions);
        }
    }

    /// Analyze a single function node
    fn analyze_function(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
    ) -> Option<FunctionMetrics> {
        let name = self.extract_function_name(node, content, language)?;
        let line = node.start_position().row + 1;

        let cognitive_complexity = self.calculate_cognitive_complexity(node, language, 0);
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(node, content, language);
        let length = self.calculate_function_length(node);
        let nesting_depth = self.calculate_nesting_depth(node, language, 0);

        Some(FunctionMetrics {
            name,
            line,
            cognitive_complexity,
            cyclomatic_complexity,
            length,
            nesting_depth,
        })
    }

    /// Extract function name from AST node
    fn extract_function_name(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
    ) -> Option<String> {
        let name_field = match language {
            MetricsLanguage::Rust => "name",
            MetricsLanguage::Python => "name",
            MetricsLanguage::JavaScript | MetricsLanguage::TypeScript => "name",
            MetricsLanguage::Go => "name",
            MetricsLanguage::Java => "name",
            MetricsLanguage::C | MetricsLanguage::Cpp => "declarator",
        };

        // Try to find name child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == name_field {
                let start = child.start_byte();
                let end = child.end_byte();
                if end <= content.len() {
                    return Some(content[start..end].to_string());
                }
            }
            // For Rust impl blocks, look deeper
            if child.kind() == "function_item" {
                return self.extract_function_name(&child, content, language);
            }
        }

        // Fallback: look for first identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let start = child.start_byte();
                let end = child.end_byte();
                if end <= content.len() {
                    return Some(content[start..end].to_string());
                }
            }
        }

        Some("<anonymous>".to_string())
    }

    /// Calculate cognitive complexity (nesting-aware)
    fn calculate_cognitive_complexity(
        &self,
        node: &tree_sitter::Node,
        language: MetricsLanguage,
        nesting: u32,
    ) -> u32 {
        let mut complexity = 0u32;
        let control_types = language.control_flow_types();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_type = child.kind();

            if control_types.contains(&child_type) {
                // +1 for control structure, +nesting for nested structures
                complexity += 1 + nesting;
                // Recurse with increased nesting
                complexity += self.calculate_cognitive_complexity(&child, language, nesting + 1);
            } else {
                // Recurse without increasing nesting
                complexity += self.calculate_cognitive_complexity(&child, language, nesting);
            }
        }

        complexity
    }

    /// Calculate cyclomatic complexity (decision points + 1)
    fn calculate_cyclomatic_complexity(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
    ) -> u32 {
        let mut complexity = 1u32; // Base complexity
        let control_types = language.control_flow_types();
        let logical_types = language.logical_operator_types();

        self.count_decision_points(
            node,
            content,
            language,
            control_types,
            logical_types,
            &mut complexity,
        );

        complexity
    }

    /// Count decision points for cyclomatic complexity
    fn count_decision_points(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        language: MetricsLanguage,
        control_types: &[&str],
        logical_types: &[&str],
        complexity: &mut u32,
    ) {
        let node_type = node.kind();

        // Control flow structures add 1
        if control_types.contains(&node_type) {
            *complexity += 1;
        }

        // Logical operators (&&, ||) add 1 each
        if logical_types.contains(&node_type) {
            // Check if it's actually && or ||
            let start = node.start_byte();
            let end = node.end_byte();
            if end <= content.len() {
                let text = &content[start..end];
                if text.contains("&&")
                    || text.contains("||")
                    || text.contains(" and ")
                    || text.contains(" or ")
                {
                    *complexity += 1;
                }
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.count_decision_points(
                &child,
                content,
                language,
                control_types,
                logical_types,
                complexity,
            );
        }
    }

    /// Calculate function length (lines)
    fn calculate_function_length(&self, node: &tree_sitter::Node) -> u32 {
        let start_line = node.start_position().row;
        let end_line = node.end_position().row;
        (end_line - start_line + 1) as u32
    }

    /// Calculate maximum nesting depth
    fn calculate_nesting_depth(
        &self,
        node: &tree_sitter::Node,
        language: MetricsLanguage,
        current_depth: u32,
    ) -> u32 {
        let mut max_depth = current_depth;
        let control_types = language.control_flow_types();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let child_type = child.kind();

            let child_depth = if control_types.contains(&child_type) {
                self.calculate_nesting_depth(&child, language, current_depth + 1)
            } else {
                self.calculate_nesting_depth(&child, language, current_depth)
            };

            max_depth = max_depth.max(child_depth);
        }

        max_depth
    }

    /// Analyze multiple files
    pub fn analyze_files(&self, paths: &[PathBuf]) -> Result<Vec<MetricViolation>> {
        let mut all_violations = Vec::new();

        for path in paths {
            match self.analyze_file(path) {
                Ok(violations) => all_violations.extend(violations),
                Err(e) => {
                    eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
                }
            }
        }

        Ok(all_violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::violation_trait::Severity;

    #[test]
    fn test_language_detection() {
        assert_eq!(
            MetricsLanguage::from_extension("rs"),
            Some(MetricsLanguage::Rust)
        );
        assert_eq!(
            MetricsLanguage::from_extension("py"),
            Some(MetricsLanguage::Python)
        );
        assert_eq!(
            MetricsLanguage::from_extension("js"),
            Some(MetricsLanguage::JavaScript)
        );
        assert_eq!(
            MetricsLanguage::from_extension("ts"),
            Some(MetricsLanguage::TypeScript)
        );
        assert_eq!(
            MetricsLanguage::from_extension("go"),
            Some(MetricsLanguage::Go)
        );
        assert_eq!(
            MetricsLanguage::from_extension("java"),
            Some(MetricsLanguage::Java)
        );
        assert_eq!(
            MetricsLanguage::from_extension("c"),
            Some(MetricsLanguage::C)
        );
        assert_eq!(
            MetricsLanguage::from_extension("cpp"),
            Some(MetricsLanguage::Cpp)
        );
        assert_eq!(MetricsLanguage::from_extension("txt"), None);
    }

    #[test]
    fn test_analyze_rust_content() {
        let content = r#"
fn simple() {
    let x = 1;
}

fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
        }
    }
    x
}
"#;

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = TreeSitterAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_content(content, Path::new("test.rs"), MetricsLanguage::Rust)
            .unwrap();

        // Should detect complexity in 'complex' function
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.item_name == "complex"));
    }

    #[test]
    fn test_analyze_python_content() {
        let content = r#"
def simple():
    x = 1

def complex(x):
    if x > 0:
        if x > 10:
            if x > 100:
                return x * 2
    return x
"#;

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = TreeSitterAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_content(content, Path::new("test.py"), MetricsLanguage::Python)
            .unwrap();

        // Should detect complexity
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_analyze_javascript_content() {
        let content = r#"
function simple() {
    let x = 1;
}

function complex(x) {
    if (x > 0) {
        if (x > 10) {
            if (x > 100) {
                return x * 2;
            }
        }
    }
    return x;
}
"#;

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CognitiveComplexity,
            3,
            Severity::Warning,
        );

        let analyzer = TreeSitterAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_content(content, Path::new("test.js"), MetricsLanguage::JavaScript)
            .unwrap();

        // Should detect complexity
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_cyclomatic_complexity() {
        let content = r#"
fn many_branches(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        2
    } else {
        3
    }
}
"#;

        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CyclomaticComplexity,
            2,
            Severity::Warning,
        );

        let analyzer = TreeSitterAnalyzer::with_thresholds(thresholds);
        let violations = analyzer
            .analyze_content(content, Path::new("test.rs"), MetricsLanguage::Rust)
            .unwrap();

        // Should detect cyclomatic complexity > 2
        let cc_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.metric_type == MetricType::CyclomaticComplexity)
            .collect();

        assert!(!cc_violations.is_empty());
    }
}
