//! AST Query Engine
//!
//! Provides querying capabilities over unified AST format
//! for rule validation.

use regex::Regex;

use super::{AstNode, AstViolation};

/// AST query for pattern matching
#[derive(Debug, Clone)]
pub struct AstQuery {
    pub language: String,
    pub node_type: String,
    pub conditions: Vec<QueryCondition>,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// Node has specific field value
    HasField { field: String, value: String },
    /// Node does not have specific field
    NotHasField { field: String },
    /// Node name matches regex pattern
    NameMatches { pattern: String },
    /// Node has children of specific type
    HasChild { child_type: String },
    /// Node has no children of specific type
    NoChild { child_type: String },
    /// Node has specific metadata value
    MetadataEquals { key: String, value: serde_json::Value },
    /// Custom condition function
    Custom { name: String },
}

impl AstQuery {
    pub fn new(language: &str, node_type: &str, message: &str, severity: &str) -> Self {
        Self {
            language: language.to_string(),
            node_type: node_type.to_string(),
            conditions: Vec::new(),
            message: message.to_string(),
            severity: severity.to_string(),
        }
    }

    pub fn with_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Execute query on AST node
    pub fn execute(&self, node: &AstNode) -> Vec<AstViolation> {
        let mut violations = Vec::new();

        if self.matches_node(node) {
            violations.push(AstViolation {
                rule_id: format!("AST_{}_{}", self.language, self.node_type),
                file: "unknown".to_string(), // Would be set by caller
                node: node.clone(),
                message: self.message.clone(),
                severity: self.severity.clone(),
            });
        }

        // Recursively check children
        for child in &node.children {
            violations.extend(self.execute(child));
        }

        violations
    }

    /// Check if node matches query conditions
    fn matches_node(&self, node: &AstNode) -> bool {
        // Check node type
        if node.kind != self.node_type {
            return false;
        }

        // Check all conditions
        for condition in &self.conditions {
            if !self.check_condition(condition, node) {
                return false;
            }
        }

        true
    }

    /// Check individual condition
    fn check_condition(&self, condition: &QueryCondition, node: &AstNode) -> bool {
        match condition {
            QueryCondition::HasField { field, value } => {
                node.metadata.get(field)
                    .and_then(|v| v.as_str())
                    .map(|s| s == value)
                    .unwrap_or(false)
            }
            QueryCondition::NotHasField { field } => {
                !node.metadata.contains_key(field)
            }
            QueryCondition::NameMatches { pattern } => {
                if let Some(name) = &node.name {
                    Regex::new(pattern).map(|re| re.is_match(name)).unwrap_or(false)
                } else {
                    false
                }
            }
            QueryCondition::HasChild { child_type } => {
                node.children.iter().any(|child| child.kind == *child_type)
            }
            QueryCondition::NoChild { child_type } => {
                !node.children.iter().any(|child| child.kind == *child_type)
            }
            QueryCondition::MetadataEquals { key, value } => {
                node.metadata.get(key)
                    .map(|v| v == value)
                    .unwrap_or(false)
            }
            QueryCondition::Custom { name } => {
                self.check_custom_condition(name, node)
            }
        }
    }

    /// Check custom conditions
    fn check_custom_condition(&self, name: &str, node: &AstNode) -> bool {
        match name {
            "has_no_docstring" => self.has_no_docstring(node),
            "is_async" => node.metadata.get("is_async")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "has_return_type" => node.metadata.get("has_return_type")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            "is_test_function" => node.name.as_ref()
                .map(|n| n.starts_with("test_") || n.contains("test"))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Check if function has docstring/documentation
    fn has_no_docstring(&self, node: &AstNode) -> bool {
        // Look for documentation comments before the function
        // This is a simplified check - real implementation would
        // need to check source code around the node
        !node.metadata.get("has_docstring")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

/// Query builder for fluent API
pub struct AstQueryBuilder {
    language: String,
    node_type: String,
    conditions: Vec<QueryCondition>,
    message: String,
    severity: String,
}

impl AstQueryBuilder {
    pub fn new(language: &str, node_type: &str) -> Self {
        Self {
            language: language.to_string(),
            node_type: node_type.to_string(),
            conditions: Vec::new(),
            message: String::new(),
            severity: "warning".to_string(),
        }
    }

    pub fn with_condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn severity(mut self, severity: &str) -> Self {
        self.severity = severity.to_string();
        self
    }

    pub fn build(self) -> AstQuery {
        AstQuery {
            language: self.language,
            node_type: self.node_type,
            conditions: self.conditions,
            message: self.message,
            severity: self.severity,
        }
    }
}

/// Common query patterns
pub struct AstQueryPatterns;

impl AstQueryPatterns {
    /// Query for functions without documentation
    pub fn undocumented_functions(language: &str) -> AstQuery {
        let node_type = match language {
            "rust" => "function_item",
            "python" => "function_definition",
            "javascript" | "typescript" => "function_declaration",
            "go" => "function_declaration",
            _ => "function",
        };

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::Custom {
                name: "has_no_docstring".to_string()
            })
            .message("Functions must be documented")
            .severity("warning")
            .build()
    }

    /// Query for unwrap() usage in non-test code
    pub fn unwrap_usage(language: &str) -> AstQuery {
        let node_type = match language {
            "rust" => "call_expression",
            "python" => "call",
            "javascript" | "typescript" => "call_expression",
            "go" => "call_expression",
            _ => "call",
        };

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::HasField {
                field: "function_name".to_string(),
                value: "unwrap".to_string(),
            })
            .with_condition(QueryCondition::Custom {
                name: "is_test_function".to_string(),
            })
            .message("Avoid unwrap() in production code")
            .severity("error")
            .build()
    }

    /// Query for async functions
    pub fn async_functions(language: &str) -> AstQuery {
        let node_type = match language {
            "rust" => "function_item",
            "python" => "function_definition",
            "javascript" | "typescript" => "function_declaration",
            "go" => "function_declaration",
            _ => "function",
        };

        AstQueryBuilder::new(language, node_type)
            .with_condition(QueryCondition::Custom {
                name: "is_async".to_string(),
            })
            .message("Async function detected")
            .severity("info")
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = AstQueryBuilder::new("rust", "function_item")
            .with_condition(QueryCondition::Custom {
                name: "has_no_docstring".to_string(),
            })
            .message("Function needs documentation")
            .severity("warning")
            .build();

        assert_eq!(query.language, "rust");
        assert_eq!(query.node_type, "function_item");
        assert_eq!(query.message, "Function needs documentation");
        assert_eq!(query.severity, "warning");
        assert_eq!(query.conditions.len(), 1);
    }

    #[test]
    fn test_query_patterns() {
        let query = AstQueryPatterns::undocumented_functions("rust");
        assert_eq!(query.language, "rust");
        assert_eq!(query.node_type, "function_item");
        assert_eq!(query.message, "Functions must be documented");
    }

    #[test]
    fn test_unwrap_pattern() {
        let query = AstQueryPatterns::unwrap_usage("rust");
        assert_eq!(query.language, "rust");
        assert_eq!(query.node_type, "call_expression");
        assert_eq!(query.message, "Avoid unwrap() in production code");
        assert_eq!(query.severity, "error");
    }
}