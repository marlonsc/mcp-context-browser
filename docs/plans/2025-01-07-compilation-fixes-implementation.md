# Compilation Fixes Implementation Plan

> **IMPORTANT:** Start with fresh context. Run `/clear` before `/implement`.

Created: 2025-01-07
Status: PENDING

> **Status Lifecycle:** PENDING → COMPLETE → VERIFIED
> - PENDING: Initial state, awaiting implementation
> - COMPLETE: All tasks implemented (set by /implement)
> - VERIFIED: Rules supervisor passed (set automatically)

## Summary
**Goal:** Fix all 36+ compilation errors to enable project verification and testing

**Architecture:** Fix structural issues in MCP server, handlers, middleware, and type definitions

**Tech Stack:** Rust 2024, rmcp 0.12.0, axum, tokio

## Scope

### In Scope
- Fix all compilation errors preventing build
- Restore proper imports and exports
- Correct type mismatches and missing fields
- Implement missing middleware and handlers
- Ensure clean compilation with zero errors

### Out of Scope
- New features or functionality
- Performance optimizations
- Code refactoring beyond compilation fixes
- Test implementation (beyond fixing broken tests)

## Prerequisites
- Rust 2024 toolchain installed
- Dependencies updated (tokio, clap, etc.)
- Basic project structure intact

## Context for Implementer
- **rmcp crate**: MCP protocol implementation, provides `#[tool]` attributes and server functionality
- **axum**: Web framework for HTTP endpoints and middleware
- **Handler pattern**: Each MCP tool has a dedicated handler struct with business logic
- **Dependency injection**: Services are injected via Arc pointers for thread safety

## Progress Tracking

**MANDATORY: Update this checklist as tasks complete. Change `[ ]` to `[x]`.**

- [ ] Task 1: Fix handler imports and exports
- [ ] Task 2: Resolve tool attribute availability
- [ ] Task 3: Implement request validation middleware
- [ ] Task 4: Fix service construction and variable scoping
- [ ] Task 5: Correct type mismatches and schema issues
- [ ] Task 6: Add missing struct fields and implementations
- [ ] Task 7: Clean up unused imports and warnings

**Total Tasks:** 7 | **Completed:** 0 | **Remaining:** 7

## Implementation Tasks

### Task 1: Fix Handler Imports and Exports

**Objective:** Resolve missing handler imports in server.rs

**Files:**
- Modify: `src/server/server.rs`
- Verify: `src/server/handlers/mod.rs`

**Implementation Steps:**
1. Verify all handler structs are properly exported from handlers/mod.rs
2. Check that handler names match the imports in server.rs
3. Ensure handler modules are properly declared in server/mod.rs
4. Test that imports resolve correctly

**Definition of Done:**
- [ ] Handler imports in server.rs resolve without errors
- [ ] All handler structs are accessible
- [ ] No "unresolved import" errors for handlers

### Task 2: Resolve Tool Attribute Availability

**Objective:** Fix missing `#[tool]` attribute and related functionality

**Files:**
- Modify: `src/server/server.rs`
- Reference: rmcp crate documentation

**Implementation Steps:**
1. Check rmcp crate features and imports for tool attribute
2. Either import the correct attribute or implement alternative approach
3. Ensure tool registration works with rmcp server
4. Remove or replace invalid tool attributes

**Definition of Done:**
- [ ] No "cannot find attribute `tool`" errors
- [ ] Tool attributes compile successfully
- [ ] MCP server can register tools properly

### Task 3: Implement Request Validation Middleware

**Objective:** Create missing `request_validation_middleware` function

**Files:**
- Create/Modify: `src/metrics/http_server.rs`
- Reference: axum middleware patterns

**Implementation Steps:**
1. Define request validation middleware function
2. Implement basic request validation logic (headers, size limits, etc.)
3. Ensure middleware signature matches axum requirements
4. Add proper error handling and responses

**Definition of Done:**
- [ ] `request_validation_middleware` function exists and compiles
- [ ] Middleware integrates correctly with axum router
- [ ] No "cannot find value" errors for middleware

### Task 4: Fix Service Construction and Variable Scoping

**Objective:** Resolve service creation and variable scoping issues

**Files:**
- Modify: `src/server/server.rs`
- Modify: `src/server/init.rs`

**Implementation Steps:**
1. Fix `McpServer::new()` constructor calls with correct parameters
2. Add missing `resource_limits` variable initialization
3. Correct service dependency injection
4. Ensure proper scoping for all variables

**Definition of Done:**
- [ ] `McpServer::new()` compiles without errors
- [ ] All service dependencies properly injected
- [ ] No "cannot find value" errors for variables

### Task 5: Correct Type Mismatches and Schema Issues

**Objective:** Fix type mismatches in tool schemas and other type issues

**Files:**
- Modify: `src/server/server.rs`
- Reference: schemars and rmcp type requirements

**Implementation Steps:**
1. Correct `input_schema` types to match rmcp expectations
2. Fix `CallToolResult` construction with proper types
3. Resolve generic parameter issues (axum Request<T>)
4. Ensure all type conversions are valid

**Definition of Done:**
- [ ] No type mismatch errors in compilation
- [ ] Schema types compatible with rmcp requirements
- [ ] Generic parameters properly specified

### Task 6: Add Missing Struct Fields and Implementations

**Objective:** Add missing fields to structs and implement required traits

**Files:**
- Modify: `src/repository/chunk_repository.rs`
- Modify: `src/admin/auth.rs`
- Modify: Multiple files with missing fields

**Implementation Steps:**
1. Identify all structs with missing required fields
2. Add missing fields with appropriate default values
3. Implement missing trait methods
4. Ensure struct initialization is complete

**Definition of Done:**
- [ ] No "missing fields" compilation errors
- [ ] All required trait methods implemented
- [ ] Struct initialization compiles successfully

### Task 7: Clean Up Unused Imports and Warnings

**Objective:** Remove unused imports and reduce compilation warnings

**Files:**
- Modify: Multiple files with unused imports
- Target: Reduce warning count significantly

**Implementation Steps:**
1. Run compilation to identify unused imports
2. Remove genuinely unused imports
3. Keep imports that may be used indirectly (macros, traits)
4. Organize imports following Rust conventions

**Definition of Done:**
- [ ] Major unused import warnings resolved
- [ ] Import organization follows Rust conventions
- [ ] No breaking changes to functionality

## Testing Strategy
- **Compilation Tests**: Ensure clean compilation with `cargo check`
- **Basic Functionality**: Verify server can start without panicking
- **Handler Tests**: Test that MCP handlers are accessible
- **Type Safety**: Ensure all type conversions work correctly

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking API compatibility | Medium | High | Test all changes incrementally, maintain interface contracts |
| Missing edge cases | Low | Medium | Focus on compilation errors first, then verify functionality |
| Performance regression | Low | Low | These are compilation fixes, not algorithmic changes |
| Incomplete fixes | Medium | High | Test compilation after each task completion |

## Open Questions
- How should tool attributes be properly implemented with rmcp 0.12.0?
- Are there any deprecated APIs being used that need updating?
- Should middleware be configurable or have fixed behavior?

---
**USER: Please review this plan. Edit any section directly, then confirm to proceed.**