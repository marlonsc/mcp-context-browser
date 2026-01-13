# Magic Numbers & Constants Centralization Plan

**Status**: PLANNING
**Created**: 2026-01-13
**Owner**: Code Quality Initiative
**Target**: Complete elimination of hardcoded magic numbers

## Goal

Centralize all magic numbers and hardcoded constants across the codebase and tests so that:
- No magic numbers exist outside of `src/infrastructure/constants.rs` or appropriate config files
- All configuration values are discoverable in one place
- Tests use the same constants as production code
- Values can be easily modified for different environments

## Current State

**Constants Module**: `src/infrastructure/constants.rs` (699 lines)
- ✅ Created with 200+ constants across 30+ categories
- ✅ Comprehensive documentation for each constant
- ✅ Organized by functional area

**Remaining Work**: Refactor code to use constants instead of hardcoded values

## Categories & Refactoring Phases

### Phase 1: HTTP & Network Configuration (Files: 3)
**Status**: PENDING
**Files**:
- `src/adapters/http_client.rs` - Replace 5 hardcoded values
- `src/infrastructure/config/server.rs` - Replace port hardcoding
- `src/infrastructure/config/metrics.rs` - Replace metrics port

**Changes**:
```rust
// Before
idle_timeout: Duration::from_secs(90),
keepalive: Duration::from_secs(60),
timeout: Duration::from_secs(30),
max_idle_per_host: 10,

// After (use constants)
idle_timeout: HTTP_CLIENT_IDLE_TIMEOUT,
keepalive: HTTP_KEEPALIVE_DURATION,
timeout: HTTP_REQUEST_TIMEOUT,
max_idle_per_host: HTTP_MAX_IDLE_PER_HOST,
```

### Phase 2: Database Connection Pool (Files: 1)
**Status**: PENDING
**Files**:
- `src/adapters/database.rs` - Replace 5 connection pool values

**Changes**:
```rust
// Before
max_connections: 20,
min_idle: 5,
max_lifetime: Duration::from_secs(1800),
idle_timeout: Duration::from_secs(600),
connection_timeout: Duration::from_secs(30),

// After
max_connections: DB_MAX_CONNECTIONS,
min_idle: DB_MIN_IDLE,
max_lifetime: DB_CONNECTION_MAX_LIFETIME,
idle_timeout: DB_CONNECTION_IDLE_TIMEOUT,
connection_timeout: DB_CONNECTION_TIMEOUT,
```

### Phase 3: Resource Limits (Files: 1)
**Status**: PENDING
**Files**:
- `src/infrastructure/limits/config.rs` - Replace 12 threshold values

**Replacements**: ~12 values for memory, CPU, disk thresholds

### Phase 4: Admin Service Defaults (Files: 1)
**Status**: PENDING
**Files**:
- `src/admin/service/helpers/admin_defaults.rs` - Replace 30+ default values

**Replacements**: All DEFAULT_* constants should use the new centralized constants

### Phase 5: Health Check Thresholds (Files: 1)
**Status**: PENDING
**Files**:
- `src/admin/service/helpers/defaults.rs` - Replace 12 health threshold values

### Phase 6: Hybrid Search & Routing (Files: 3)
**Status**: PENDING
**Files**:
- `src/adapters/hybrid_search/config.rs` - Replace 3 weight values
- `src/adapters/hybrid_search/bm25.rs` - Replace k1, b parameters
- `src/adapters/providers/routing/router.rs` - Replace 15 weight/score values

### Phase 7: Circuit Breaker (Files: 1)
**Status**: PENDING
**Files**:
- `src/adapters/providers/routing/circuit_breaker.rs` - Replace 5 values

### Phase 8: Code Chunking (Files: 12)
**Status**: PENDING
**Files**:
- `src/chunking/engine.rs` - Replace chunk_size
- `src/chunking/config.rs` - Replace default chunk_size
- `src/chunking/languages/*.rs` (12 files) - Replace language-specific chunk sizes

**Replacements**:
- Use `CHUNK_SIZE_RUST`, `CHUNK_SIZE_PYTHON`, etc.
- Replace min_length, min_lines, max_depth per language

### Phase 9: Vector Store Configuration (Files: 4)
**Status**: PENDING
**Files**:
- `src/adapters/providers/vector_store/milvus.rs` - Replace 4 values
- `src/adapters/providers/vector_store/filesystem.rs` - Replace 3 values
- `src/adapters/providers/vector_store/edgevec.rs` - Replace 3 quantization levels

### Phase 10: Embedding Provider Limits (Files: 5)
**Status**: PENDING
**Files**:
- `src/adapters/providers/embedding/voyageai.rs` - Replace 1 max token value
- `src/adapters/providers/embedding/ollama.rs` - Replace 7 model-specific values
- `src/adapters/providers/embedding/openai.rs` - Replace 1 value
- `src/adapters/providers/embedding/fastembed.rs` - Replace 1 value
- `src/adapters/providers/embedding/gemini.rs` - Replace 1 value

### Phase 11: Indexing Configuration (Files: 2)
**Status**: PENDING
**Files**:
- `src/application/indexing.rs` - Replace batch_size
- `src/admin/service/implementation.rs` - Replace chunk_size, max_file_size

### Phase 12: Event Bus Configuration (Files: 2)
**Status**: PENDING
**Files**:
- `src/infrastructure/events/tokio_impl.rs` - Replace channel capacity
- `src/infrastructure/events/nats.rs` - Replace 4 NATS values

### Phase 13: Rate Limiting (Files: 2)
**Status**: PENDING
**Files**:
- `src/infrastructure/rate_limit.rs` - Replace 3 values
- `src/infrastructure/auth/rate_limit.rs` - Replace rate limit configs

### Phase 14: Authentication (Files: 3)
**Status**: PENDING
**Files**:
- `src/infrastructure/auth/config.rs` - Replace JWT_SECRET_MIN_LENGTH
- `src/infrastructure/auth/user_store.rs` - Replace JWT secret length
- `src/infrastructure/auth/password.rs` - Replace password min length

### Phase 15: Miscellaneous Infrastructure (Files: 10)
**Status**: PENDING
**Files**:
- `src/infrastructure/connection_tracker.rs` - Replace max_connections
- `src/infrastructure/provider_connection_tracker.rs` - Replace polling interval
- `src/infrastructure/cache.rs` - Replace cache TTLs
- `src/infrastructure/cache/queue.rs` - Replace TTL
- `src/infrastructure/cache/providers/redis.rs` - Replace pool timeout
- `src/infrastructure/logging.rs` - Replace buffer capacity
- `src/infrastructure/metrics/performance.rs` - Replace history size
- `src/server/security.rs` - Replace HSTS, request size limits
- `src/infrastructure/crypto.rs` - Replace key rotation, key size
- `src/infrastructure/binary_watcher.rs` - Replace debounce, sleep intervals
- `src/server/transport/config.rs` - Replace session configuration
- `src/infrastructure/recovery.rs` - Replace retry delay values
- `src/infrastructure/backup.rs` - Replace channel capacity
- `src/infrastructure/utils.rs` - Replace validation limits
- `src/daemon/types.rs` - Replace lock age
- `src/sync/manager.rs` - Replace retention

## Tests Refactoring

### Test Categories to Update

**1. Integration Tests** (Tests: 20+)
- Replace all hardcoded duration values
- Use constants for buffer sizes, timeouts
- Update test fixtures to use constants

**2. Unit Tests** (Tests: 50+)
- Test fixtures with hardcoded values
- Performance test parameters
- Mock configuration values

**3. Property-Based Tests**
- Boundary value tests should use constants
- Range tests should reference constants

**Test Files to Update**:
- `tests/admin/**/*.rs` - Admin service tests
- `tests/chunking/**/*.rs` - Chunking configuration tests
- `tests/core/**/*.rs` - Core functionality tests
- `tests/providers/**/*.rs` - Provider tests
- `tests/server/**/*.rs` - Server configuration tests
- `src/**/tests.rs` - Inline tests

### Example Test Refactoring

```rust
// Before
#[test]
fn test_timeout() {
    let timeout = Duration::from_secs(30);
    assert_eq!(timeout.as_secs(), 30);
}

// After
use crate::infrastructure::constants::HTTP_REQUEST_TIMEOUT;

#[test]
fn test_timeout() {
    assert_eq!(HTTP_REQUEST_TIMEOUT.as_secs(), 30);
}
```

## Implementation Order

**Priority 1: High Impact** (Affects multiple files, commonly used)
1. Phase 1: HTTP & Network Configuration
2. Phase 2: Database Connection Pool
3. Phase 8: Code Chunking
4. Phase 10: Embedding Provider Limits

**Priority 2: Medium Impact** (Affects multiple subsystems)
5. Phase 3: Resource Limits
6. Phase 6: Hybrid Search & Routing
7. Phase 7: Circuit Breaker
8. Phase 12: Event Bus Configuration

**Priority 3: Lower Impact** (More localized)
9. Phase 4: Admin Service Defaults
10. Phase 5: Health Check Thresholds
11. Phase 9: Vector Store Configuration
12. Phase 11: Indexing Configuration
13. Phase 13: Rate Limiting
14. Phase 14: Authentication
15. Phase 15: Miscellaneous Infrastructure

## Testing Strategy

### Compile Check
```bash
make build  # Verify all constants are properly imported
```

### Test Execution
```bash
make test   # Ensure all tests pass with new constants
```

### Validation
```bash
cargo grep "Duration::from_secs([0-9])" --type rust
cargo grep "::from_secs([0-9])" --type rust
# Should return minimal/no matches after refactoring
```

## Success Criteria

- [ ] All 200+ magic numbers replaced with named constants
- [ ] Constants properly imported in all modules
- [ ] All 450+ tests updated to use constants
- [ ] `make quality` passes with 0 errors
- [ ] No hardcoded Duration, usize, u32 values outside constants.rs
- [ ] All language-specific chunk sizes use constants
- [ ] All provider configurations use centralized values
- [ ] Test fixtures and mocks use constants

## Files to Modify

**Configuration Module**: 1 file
- `src/infrastructure/constants.rs` ✅ COMPLETED

**Source Code**: ~40 files
- HTTP/Network: 3 files
- Database: 1 file
- Resources: 1 file
- Admin: 5 files
- Chunking: 13 files
- Embedding: 6 files
- Hybrid Search: 3 files
- Routing: 3 files
- Event Bus: 2 files
- Rate Limiting: 2 files
- Auth: 3 files
- Infrastructure: 15 files

**Test Code**: ~60 files
- Integration tests: 20+ files
- Unit tests: 40+ files
- Inline tests: Various files

**Metrics**:
- Total lines to change: ~500-1000
- Total constants to apply: 200+
- Estimated time per phase: 30-90 minutes
- Total estimated time: 20-30 hours

## Notes

### Config vs Constants

**Constants** (use for production defaults):
- Server ports (3000, 3001)
- Timeouts that are fundamental to operation (30s, 5min, 1hr)
- Fixed limits (max tokens, dimensions)
- Weights and thresholds

**Config** (use for runtime customization):
- Port numbers (allow env override)
- Timeout multipliers
- Resource limits (allow tuning)
- Rate limits (allow tuning)

Current approach: Everything starts in constants, can be moved to config if needed later.

### Import Pattern

All files should import constants like:
```rust
use crate::infrastructure::constants::*;
// Or specific imports
use crate::infrastructure::constants::{
    HTTP_REQUEST_TIMEOUT,
    DB_MAX_CONNECTIONS,
};
```

### Documentation

Each phase should include:
- List of files changed
- Before/after code samples
- Test verification steps
- Validation checklist

## Risk Assessment

**Low Risk**:
- Constants are correctly defined
- Type safety is maintained
- Imports are straightforward

**Medium Risk**:
- Some values might need adjustment after testing
- Tests might reveal assumptions about specific values
- Config file updates if values are externalized

## Rollback Plan

If issues arise:
1. Each phase is independent and can be rolled back
2. Constants file is single source of truth
3. Git history allows reverting individual phases
4. No breaking API changes

## Related ADRs

- ADR 006: Code Audit and Improvements (type safety, unwrap elimination)
- Project CLAUDE.md: Code standards requiring DRY constants

---

**Status**: READY FOR IMPLEMENTATION
**Next Steps**: Begin Phase 1 refactoring
