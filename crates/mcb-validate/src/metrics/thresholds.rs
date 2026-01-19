//! Metric thresholds configuration
//!
//! Defines threshold values for various code metrics and how they map to violations.

use crate::violation_trait::Severity;
use std::collections::HashMap;

/// Types of metrics we can measure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    /// Cognitive complexity - how hard code is to understand
    CognitiveComplexity,
    /// Number of lines/statements in a function
    FunctionLength,
    /// Maximum nesting depth (if/for/while/match)
    NestingDepth,
}

impl MetricType {
    /// Get the human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::CognitiveComplexity => "Function",
            Self::FunctionLength => "Function",
            Self::NestingDepth => "Function",
        }
    }

    /// Get metric description
    pub fn description(&self) -> &'static str {
        match self {
            Self::CognitiveComplexity => "cognitive complexity",
            Self::FunctionLength => "length",
            Self::NestingDepth => "nesting depth",
        }
    }

    /// Get suggestion for fixing
    pub fn suggestion(&self) -> String {
        match self {
            Self::CognitiveComplexity => {
                "Consider breaking this function into smaller, focused functions. \
                 Extract complex conditions into named functions or early returns."
                    .to_string()
            }
            Self::FunctionLength => {
                "Consider extracting helper functions or using the Extract Method refactoring. \
                 Functions should ideally do one thing well."
                    .to_string()
            }
            Self::NestingDepth => {
                "Consider using early returns, guard clauses, or extracting nested logic \
                 into separate functions to reduce nesting."
                    .to_string()
            }
        }
    }
}

/// A single metric threshold configuration
#[derive(Debug, Clone)]
pub struct MetricThreshold {
    /// Maximum allowed value
    pub max_value: u32,
    /// Severity when threshold is exceeded
    pub severity: Severity,
}

/// Configuration for all metric thresholds
#[derive(Debug, Clone)]
pub struct MetricThresholds {
    thresholds: HashMap<MetricType, MetricThreshold>,
}

impl Default for MetricThresholds {
    fn default() -> Self {
        Self::new()
            // Default thresholds based on common industry standards
            .with_threshold(MetricType::CognitiveComplexity, 15, Severity::Warning)
            .with_threshold(MetricType::FunctionLength, 50, Severity::Warning)
            .with_threshold(MetricType::NestingDepth, 4, Severity::Warning)
    }
}

impl MetricThresholds {
    /// Create empty thresholds
    pub fn new() -> Self {
        Self {
            thresholds: HashMap::new(),
        }
    }

    /// Add or update a threshold
    pub fn with_threshold(mut self, metric: MetricType, max_value: u32, severity: Severity) -> Self {
        self.thresholds.insert(
            metric,
            MetricThreshold {
                max_value,
                severity,
            },
        );
        self
    }

    /// Get threshold for a metric type
    pub fn get(&self, metric: MetricType) -> Option<&MetricThreshold> {
        self.thresholds.get(&metric)
    }

    /// Parse thresholds from YAML rule config
    pub fn from_yaml(config: &serde_json::Value) -> Self {
        let mut thresholds = Self::new();

        if let Some(obj) = config.as_object() {
            // Parse cognitive_complexity
            if let Some(cc) = obj.get("cognitive_complexity") {
                if let Some(max) = cc.get("max").and_then(|v| v.as_u64()) {
                    let severity = cc
                        .get("severity")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "error" => Severity::Error,
                            "info" => Severity::Info,
                            _ => Severity::Warning,
                        })
                        .unwrap_or(Severity::Warning);

                    thresholds = thresholds.with_threshold(
                        MetricType::CognitiveComplexity,
                        max as u32,
                        severity,
                    );
                }
            }

            // Parse function_length
            if let Some(fl) = obj.get("function_length") {
                if let Some(max) = fl.get("max").and_then(|v| v.as_u64()) {
                    let severity = fl
                        .get("severity")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "error" => Severity::Error,
                            "info" => Severity::Info,
                            _ => Severity::Warning,
                        })
                        .unwrap_or(Severity::Warning);

                    thresholds =
                        thresholds.with_threshold(MetricType::FunctionLength, max as u32, severity);
                }
            }

            // Parse nesting_depth
            if let Some(nd) = obj.get("nesting_depth") {
                if let Some(max) = nd.get("max").and_then(|v| v.as_u64()) {
                    let severity = nd
                        .get("severity")
                        .and_then(|v| v.as_str())
                        .map(|s| match s {
                            "error" => Severity::Error,
                            "info" => Severity::Info,
                            _ => Severity::Warning,
                        })
                        .unwrap_or(Severity::Warning);

                    thresholds =
                        thresholds.with_threshold(MetricType::NestingDepth, max as u32, severity);
                }
            }
        }

        // Fill in defaults for missing thresholds
        if thresholds.get(MetricType::CognitiveComplexity).is_none() {
            thresholds = thresholds.with_threshold(MetricType::CognitiveComplexity, 15, Severity::Warning);
        }
        if thresholds.get(MetricType::FunctionLength).is_none() {
            thresholds = thresholds.with_threshold(MetricType::FunctionLength, 50, Severity::Warning);
        }
        if thresholds.get(MetricType::NestingDepth).is_none() {
            thresholds = thresholds.with_threshold(MetricType::NestingDepth, 4, Severity::Warning);
        }

        thresholds
    }

    /// Create thresholds from a MetricsConfig struct (from ValidatedRule)
    pub fn from_metrics_config(config: &crate::rules::yaml_loader::MetricsConfig) -> Self {
        let mut thresholds = Self::new();

        // Convert cognitive_complexity
        if let Some(cc) = &config.cognitive_complexity {
            let severity = cc
                .severity
                .as_ref()
                .map(|s| match s.as_str() {
                    "error" => Severity::Error,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                })
                .unwrap_or(Severity::Warning);
            thresholds = thresholds.with_threshold(MetricType::CognitiveComplexity, cc.max, severity);
        }

        // Convert cyclomatic_complexity (maps to cognitive for now)
        if let Some(cyc) = &config.cyclomatic_complexity {
            let severity = cyc
                .severity
                .as_ref()
                .map(|s| match s.as_str() {
                    "error" => Severity::Error,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                })
                .unwrap_or(Severity::Warning);
            // If cognitive_complexity wasn't set, use cyclomatic as a fallback
            if config.cognitive_complexity.is_none() {
                thresholds =
                    thresholds.with_threshold(MetricType::CognitiveComplexity, cyc.max, severity);
            }
        }

        // Convert function_length
        if let Some(fl) = &config.function_length {
            let severity = fl
                .severity
                .as_ref()
                .map(|s| match s.as_str() {
                    "error" => Severity::Error,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                })
                .unwrap_or(Severity::Warning);
            thresholds = thresholds.with_threshold(MetricType::FunctionLength, fl.max, severity);
        }

        // Convert nesting_depth
        if let Some(nd) = &config.nesting_depth {
            let severity = nd
                .severity
                .as_ref()
                .map(|s| match s.as_str() {
                    "error" => Severity::Error,
                    "info" => Severity::Info,
                    _ => Severity::Warning,
                })
                .unwrap_or(Severity::Warning);
            thresholds = thresholds.with_threshold(MetricType::NestingDepth, nd.max, severity);
        }

        thresholds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_thresholds() {
        let thresholds = MetricThresholds::default();

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 15);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 50);

        let nd = thresholds.get(MetricType::NestingDepth).unwrap();
        assert_eq!(nd.max_value, 4);
    }

    #[test]
    fn test_custom_thresholds() {
        let thresholds = MetricThresholds::new()
            .with_threshold(MetricType::CognitiveComplexity, 10, Severity::Error)
            .with_threshold(MetricType::FunctionLength, 30, Severity::Warning);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 10);
        assert_eq!(cc.severity, Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 30);
        assert_eq!(fl.severity, Severity::Warning);
    }

    #[test]
    fn test_from_yaml() {
        let yaml = serde_json::json!({
            "cognitive_complexity": {
                "max": 20,
                "severity": "error"
            },
            "function_length": {
                "max": 100
            }
        });

        let thresholds = MetricThresholds::from_yaml(&yaml);

        let cc = thresholds.get(MetricType::CognitiveComplexity).unwrap();
        assert_eq!(cc.max_value, 20);
        assert_eq!(cc.severity, Severity::Error);

        let fl = thresholds.get(MetricType::FunctionLength).unwrap();
        assert_eq!(fl.max_value, 100);
        assert_eq!(fl.severity, Severity::Warning); // Default
    }
}
