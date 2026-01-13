# DRY & SOLID Refactoring - Baseline Metrics

**Date**: 2026-01-13  
**Commit**: ccb25b6 (Pre-work cleanup complete)

## Phase 1A: FileUtils Migration Baseline

### Current Helper Adoption
- **FileUtils current usage**: 11 call sites
- **TimeUtils current usage**: 26 call sites
- **tokio::fs direct usage**: 20+ instances
- **Manual serde_json operations**: 12+ instances

### Target Files for FileUtils Migration

#### High Priority (>30 lines saved each)
1. **src/infrastructure/snapshot/manager.rs** - 4 calls, ~30 lines
   - Line 86-90: `fs::read_to_string` + `serde_json::from_str`
   - Line 311-315: `serde_json::to_string_pretty` + `fs::write`
   - Line 33-34: `fs::create_dir_all`
   - Line 314: Direct `fs::write`

2. **src/adapters/providers/vector_store/filesystem.rs** - 5 calls, ~40 lines
   - Multiple shard metadata operations
   - Index persistence operations

#### Medium Priority (15-25 lines saved each)
3. **src/server/admin/service/helpers/configuration.rs** - 2 calls, ~15 lines
4. **src/infrastructure/crypto.rs** - 2 calls, ~15 lines
5. **src/server/handlers/get_indexing_status.rs** - 1 call, ~8 lines

### Total Potential Savings: 150-170 lines (Phase 1A)

## Phase 1B: TimeUtils Migration Baseline

### Current TimeUtils Usage
- Already used in 26 files
- Pattern: `TimeUtils::now_unix_secs()`, `TimeUtils::now_unix_millis()`, etc.

### Target Patterns

#### High Priority
1. **src/infrastructure/daemon/service.rs** - Already using TimeUtils (verify consistency)
2. **src/infrastructure/snapshot/manager.rs** - 2 calls to `SystemTime::now().duration_since(UNIX_EPOCH)`
   - Lines 63-66, 267-270

#### Note
- **src/infrastructure/sync/manager.rs** - File metadata timestamps (KEEP as-is, edge case)
  - These are from `fs::metadata().modified()`, not current time

### Total Potential Savings: 20-25 lines (Phase 1B)

## Phase 1 Total: 250-300 lines saved

## Test Baseline
- **Total tests**: 790+ (run with `make test`)
- **Current status**: All passing ✅
- **Quality gates**: `make quality` passing ✅

## Reference Data
- FileUtils location: `src/infrastructure/utils.rs:422-492`
- TimeUtils location: `src/infrastructure/utils.rs:288-320`
- Constants reference: `src/infrastructure/constants.rs` (785 lines)
