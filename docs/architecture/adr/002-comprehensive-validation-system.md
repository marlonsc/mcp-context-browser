# ADR 002: Comprehensive Validation System

Date: 2026-01-07

## Status

Accepted

## Context

The MCP Context Browser handles complex business data structures (CodeChunk, Embedding, configuration) that require validation at multiple levels:

1. **Data integrity**: Ensuring required fields are present and valid
2. **Business rules**: Enforcing domain-specific constraints
3. **Security**: Preventing malicious input and injection attacks
4. **Performance**: Early validation to avoid expensive operations with invalid data

The current validation is scattered across the codebase with inconsistent patterns and incomplete coverage.

## Decision

Implement a comprehensive validation system using the `validator` crate with custom business logic validators. The system will provide:

1. **Declarative validation** using derive macros
2. **Custom validators** for business-specific rules
3. **Multi-layer validation** (input, business logic, security)
4. **Consistent error handling** with actionable messages
5. **Performance optimization** through early validation

## Consequences

### Positive
- **Consistency**: Unified validation approach across all data structures
- **Maintainability**: Centralized validation logic
- **Security**: Comprehensive input sanitization and validation
- **Performance**: Early rejection of invalid data
- **Developer Experience**: Clear validation errors with actionable messages

### Negative
- **Dependency**: Additional crate dependency
- **Learning curve**: New validation DSL to learn
- **Runtime overhead**: Validation execution time

### Risks
- **Performance impact**: Validation on hot paths
- **Error message quality**: Ensuring messages are actionable
- **Coverage completeness**: Missing validation rules

## Implementation

### Data Structure Validation
```rust
#[derive(Debug, Validate)]
pub struct CodeChunk {
    #[validate(length(min = 1))]
    pub id: String,

    #[validate(length(min = 1, max = 10000))]
    pub content: String,

    #[validate(length(min = 1))]
    pub file_path: String,

    #[validate(range(min = 1))]
    pub start_line: u32,

    #[validate(range(min = 1))]
    #[validate(custom(function = "validate_line_range", arg = "&self.start_line"))]
    pub end_line: u32,
}
```

### Custom Validators
```rust
fn validate_file_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::new("Path cannot be empty"));
    }

    if path.contains("..") {
        return Err(ValidationError::new("Path cannot contain directory traversal"));
    }

    Ok(())
}

fn validate_line_range(end_line: u32, start_line: &u32) -> Result<(), ValidationError> {
    if *start_line > end_line {
        return Err(ValidationError::new("Start line cannot be greater than end line"));
    }
    Ok(())
}
```

### Business Logic Integration
```rust
impl CodeChunk {
    pub fn validate_business_rules(&self) -> Result<(), Error> {
        // Additional business logic validation beyond basic field validation
        if self.language == Language::Unknown && self.content.contains("fn ") {
            return Err(Error::validation("Unknown language but appears to be Rust code"));
        }

        Ok(())
    }
}
```

## Alternatives Considered

### Option 1: Manual Validation
```rust
if chunk.content.is_empty() {
    return Err("Content cannot be empty");
}
```
- **Pros**: No dependencies, full control
- **Cons**: Verbose, error-prone, inconsistent

### Option 2: Custom Derive Macros
- **Pros**: Clean syntax, compile-time validation
- **Cons**: Complex macro implementation, maintenance burden

### Option 3: JSON Schema Validation
- **Pros**: Standard schemas, tooling support
- **Cons**: Runtime-only, less type-safe

## Validation Layers

### 1. Input Validation
- Required fields presence
- Type constraints (string length, number ranges)
- Format validation (paths, URLs)

### 2. Business Logic Validation
- Domain-specific rules
- Cross-field validation
- Consistency checks

### 3. Security Validation
- Path traversal prevention
- XSS prevention
- Injection attack prevention

### 4. Performance Validation
- Size limits to prevent DoS
- Complexity limits
- Resource usage validation

## Error Handling

Validation errors provide:
- **Field identification**: Which field failed validation
- **Error type**: What validation rule was violated
- **Actionable message**: How to fix the issue
- **Context**: Additional debugging information

## References

- [Validator Crate](https://docs.rs/validator/latest/validator/)
- [Input Validation](https://owasp.org/www-community/Input_Validation_Cheat_Sheet)
- [Domain Validation](https://martinfowler.com/bliki/EvansClassification.html)