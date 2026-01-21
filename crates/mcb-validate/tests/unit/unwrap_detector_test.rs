//! Tests for AST-based unwrap detector

#[cfg(test)]
mod unwrap_detector_tests {
    use mcb_validate::ast::UnwrapDetector;

    #[test]
    fn test_detector_creation() {
        let detector = UnwrapDetector::new();
        assert!(
            detector.is_ok(),
            "Should create unwrap detector successfully"
        );
    }

    #[test]
    fn test_detect_unwrap_simple() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code = "fn main() { let x = Some(1).unwrap(); }";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should detect unwrap");

        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].method, "unwrap");
        assert!(!detections[0].in_test);
    }

    #[test]
    fn test_detect_expect() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code = "fn main() { let x = Some(1).expect(\"error\"); }";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should detect expect");

        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].method, "expect");
    }

    #[test]
    fn test_detect_multiple() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code =
            "fn main() {\n    let x = Some(1).unwrap();\n    let y = Some(2).expect(\"error\");\n}";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should detect multiple");

        assert_eq!(detections.len(), 2);
        assert_eq!(detections[0].method, "unwrap");
        assert_eq!(detections[1].method, "expect");
    }

    #[test]
    fn test_ignore_safe_alternatives() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code = "fn main() {\n    let x = Some(1).unwrap_or(0);\n    let y = Some(2).unwrap_or_default();\n}";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should not detect safe alternatives");

        assert_eq!(detections.len(), 0, "Should not detect unwrap_or variants");
    }

    #[test]
    fn test_detect_in_test_module() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code = "#[cfg(test)]\nmod tests {\n    fn test() {\n        let x = Some(1).unwrap();\n    }\n}";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should detect in test module");

        assert_eq!(detections.len(), 1);
        assert!(detections[0].in_test, "Should be marked as in test");
    }

    #[test]
    fn test_line_numbers_are_correct() {
        let mut detector = UnwrapDetector::new().expect("Should create detector");
        let code = "fn main() {\n    let x = Some(1).unwrap();\n}\n";

        let detections = detector
            .detect_in_content(code, "test.rs")
            .expect("Should detect");

        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].line, 2, "Should be on line 2");
    }
}
