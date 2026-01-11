# Core Architecture Fixes Plan

**Created**: 2026-01-09
**Status**: IN_PROGRESS
**Phase**: Implementation
**Approved**: 2026-01-09

## User Decisions

-   **Q1 (Security Module Strategy)**: A - Full Rewrite of `auth.rs` and `crypto.rs`
-   **Q2 (Global State Elimination)**: A - Immediate DI Refactor
-   **Q3 (Priority Order)**: B - Architecture First

## Context

Architectural analysis of the `src/core/` module to identify flaws, weaknesses, and issues while the code is still small.

## Exploration Summary

Analyzed files: 16 core modules (auth.rs, backup.rs, cache.rs, crypto.rs, database.rs, error.rs, events.rs, http_client.rs, hybrid_search.rs, limits.rs, logging.rs, merkle.rs, mod.rs, rate_limit.rs, types.rs, validation.rs)

---

## Critical Issues Found

### 1. CRITICAL: Security Flaws in auth.rs

**Severity**: Critical
**Impact**: Authentication bypass, credential exposure

**Issues**:

-   JWT signature uses `seahash::hash()` (NOT cryptographically secure)
-   Hardcoded admin credentials: `email == "admin@context.browser" && password == "admin"`
-   User passwords compared in plaintext (never hashed)
-   No token refresh/revocation mechanism
-   Comments acknowledge "in production use proper database" but shipped as-is

**Fix Approach**:

-   Replace seahash with proper HMAC-SHA256 (jsonwebtoken crate)
-   Implement password hashing (argon2 or bcrypt)
-   Add token expiration and refresh flow
-   Move credentials to environment configuration

---

### 2. CRITICAL: Flawed Crypto Implementation in crypto.rs

**Severity**: Critical
**Impact**: Data encryption not production-ready

**Issues**:

-   Master key stored in plaintext file (`~/.context/master.key`)
-   No file permission validation or enforcement (should be 0600)
-   `key_id` always "master"; `generate_data_key()` creates keys but never used
-   `rotation_days` config parameter never checked/enforced
-   Nonce generation lacks entropy validation

**Fix Approach**:

-   Add file permission checks on key file creation (mode 0600)
-   Implement proper envelope encryption with data keys
-   Add key rotation enforcement
-   Validate entropy source

---

### 3. HIGH: Global Static State Pattern

**Severity**: High
**Impact**: Testing difficulty, race conditions, inflexibility

**Affected Files**:

-   `database.rs`: `DB_POOL` OnceLock singleton
-   `http_client.rs`: Global HTTP client singleton
-   `limits.rs`: Static resource limits

**Issues**:

-   `get_or_create_*` functions have potential race conditions
-   Cannot inject mock implementations in tests
-   Single instance per process limits flexibility

**Fix Approach**:

-   Convert to proper dependency injection via trait objects
-   Pass instances through constructors
-   Add factory functions for testing

---

### 4. HIGH: Mixed Sync/Async Patterns

**Severity**: High
**Impact**: Potential deadlocks, confusing API

**Affected Files**:

-   `database.rs`: `health_check()` is async but uses sync `get_connection()`
-   `crypto.rs`: Sync file operations in async context

**Fix Approach**:

-   Use `tokio::task::spawn_blocking()` for sync operations
-   Make API consistently async
-   Document sync vs async expectations

---

### 5. HIGH: Memory Management in rate_limit.rs

**Severity**: High
**Impact**: Potential memory leaks, unfair rate limiting

**Issues**:

-   Cleanup removes first half of entries regardless of age
-   Old entries may survive while newer ones are removed
-   `unreachable!()` macro can cause panic

**Fix Approach**:

-   Implement proper LRU eviction or time-based cleanup
-   Replace `unreachable!()` with proper error handling
-   Add cleanup interval configuration

---

### 6. MEDIUM: Error Handling Architecture in error.rs

**Severity**: Medium
**Impact**: Inconsistent error creation, confusing API

**Issues**:

-   `io()` helper creates `Generic` error, not `Io` variant
-   Both `String(String)` and `Generic` handle string errors
-   `From<&str>` and `From<String>` may conflict with `#[from]` on Generic

**Fix Approach**:

-   Fix `io()` to create proper `Io` variant
-   Consolidate string error paths
-   Add clear documentation on which variant to use

---

### 7. MEDIUM: Code Duplication in backup.rs

**Severity**: Medium
**Impact**: Maintenance burden, bug duplication

**Issues**:

-   `create_backup_sync()` and `do_create_backup()` are nearly identical
-   Both implement same tar.gz creation logic

**Fix Approach**:

-   Extract shared backup creation logic to private function
-   Call from both `create_backup_sync()` and `do_create_backup()`

---

### 8. MEDIUM: Hardcoded Strings and Magic Numbers

**Severity**: Medium
**Impact**: Type safety, refactoring difficulty

**Affected Files**:

-   `limits.rs`: String-based operation types ("indexing", "search", "embedding")
-   `auth.rs`: Hardcoded path "/api/context/metrics" for auth bypass
-   `hybrid_search.rs`: Magic numbers for weights

**Fix Approach**:

-   Create enum for operation types
-   Add configurable auth bypass paths
-   Extract magic numbers to named constants

---

### 9. LOW: Missing Validation and Defaults

**Severity**: Low
**Impact**: Edge case failures

**Issues**:

-   `merkle.rs`: `unwrap_or_default()` for file names
-   `database.rs`: Hardcoded PostgreSQL URL in default config
-   `hybrid_search.rs`: Config validation doesn't check weight sum = 1.0

**Fix Approach**:

-   Add explicit error handling for missing file names
-   Move database URL to environment variable
-   Add validation that BM25 + semantic weights sum to 1.0

---

### 10. LOW: Testing Infrastructure Gaps

**Severity**: Low
**Impact**: Test coverage, CI reliability

**Issues**:

-   Global statics prevent isolated unit tests
-   No mock implementations for core traits
-   Hardcoded temp paths in some tests

**Fix Approach**:

-   Add mockable trait implementations
-   Use tempdir consistently
-   Document test requirements

---

## Design Questions (Require User Input)

### Q1: Security Module Rewrite

The auth and crypto modules have fundamental security issues. Options:

**A) Full Rewrite**: Replace auth.rs and crypto.rs with production-grade implementations

-   Pros: Proper security, clean slate
-   Cons: Significant effort

**B) Partial Fix**: Fix critical issues only, document limitations

-   Pros: Faster, less risk
-   Cons: Technical debt remains

**C) Feature Flag**: Add new secure implementations alongside old ones

-   Pros: Backwards compatible, gradual migration
-   Cons: Code duplication

### Q2: Global State Elimination

**A) Immediate Refactor**: Remove all global statics, use DI

-   Pros: Clean architecture, testable
-   Cons: Affects many files

**B) Wrapper Pattern**: Keep statics but add testable wrappers

-   Pros: Less invasive
-   Cons: Still has global state

### Q3: Priority Order

Which category should we address first?

-   **A) Security First**: auth.rs, crypto.rs
-   **B) Architecture First**: Global state, mixed patterns
-   **C) Quality First**: Error handling, code duplication

---

## Proposed Implementation Plan

### Phase 1: Security Foundations

-   [ ] Replace seahash with HMAC-SHA256 in auth.rs
-   [ ] Add password hashing with argon2
-   [ ] Add file permission checks in crypto.rs
-   [ ] Remove hardcoded credentials

### Phase 2: Global State Elimination

-   [ ] Create traits for database pool, HTTP client, resource limits
-   [ ] Replace static singletons with dependency injection
-   [ ] Update all call sites

### Phase 3: Error Handling Cleanup

-   [ ] Fix io() helper
-   [ ] Consolidate string error variants
-   [ ] Add error documentation

### Phase 4: Code Quality

-   [ ] Extract backup logic duplication
-   [ ] Create enum for operation types
-   [ ] Add validation for config values

### Phase 5: Testing Infrastructure

-   [ ] Add mock implementations
-   [ ] Fix test isolation issues
-   [ ] Add integration tests for core modules

---

## Success Criteria

-   [ ] No critical security vulnerabilities
-   [ ] Zero global static state in core modules
-   [ ] Consistent async patterns
-   [ ] All tests pass with proper isolation
-   [ ] No `unwrap()`/`expect()` in production paths
-   [ ] Code coverage > 80% for core modules

---

## Files to Modify

| File | Changes |
|------|---------|
| src/core/auth.rs | Security rewrite |
| src/core/crypto.rs | Security hardening |
| src/core/database.rs | DI refactor |
| src/core/http_client.rs | DI refactor |
| src/core/limits.rs | DI refactor, enum types |
| src/core/rate_limit.rs | Memory management fix |
| src/core/error.rs | Consistency fixes |
| src/core/backup.rs | Code deduplication |
| src/core/hybrid_search.rs | Validation |

---

## Estimated Impact

-   **Breaking Changes**: Yes (API changes for DI)
-   **Test Updates Required**: Yes (new mocking patterns)
-   **Documentation Updates**: Yes (security guidelines)

---

## Next Steps

1.  User answers design questions above
2.  Prioritize based on user input
3.  Create detailed task breakdown for Phase 1
4.  Begin implementation with TDD approach
