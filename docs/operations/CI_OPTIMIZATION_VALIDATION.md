# CI Optimization - Validation & Testing Guide

## Status

This document tracks validation of CI optimizations before considering them production-ready.

**Last Updated**: 2026-01-28
**Status**: ⏳ IN PROGRESS (Awaiting validation tests)

## Critical Validations Required

### ✅ Completed Validations

1.  **Test Matrix Logic** (Commit: d211ffe)

-   [x] Verified `release-build` dependency updated from `test` to `test-main`
-   [x] Confirmed job dependencies are syntactically correct
-   [x] Validated workflow file passes YAML validation

### ⏳ Pending Validations

#### 1. Coverage Execution (Tarpaulin Installation & Test)

**Objective**: Verify that coverage job completes without timeout when integration tests are excluded

**Steps**:

```bash
# 1. Install tarpaulin
cargo install cargo-tarpaulin --locked

# 2. Run coverage with new configuration
make coverage LCOV=1 TEST_THREADS=4

# 3. Check results
ls -lh coverage/lcov.info
head -50 coverage/lcov.info
```

**Success Criteria**:

-   [ ] Tarpaulin installation completes successfully
-   [ ] Coverage job completes within 30 minutes
-   [ ] No timeout errors from Milvus/Ollama dependencies
-   [ ] `lcov.info` file is generated (> 1MB)
-   [ ] Integration tests are excluded from report (verify in output)
-   [ ] Coverage percentage is reasonable (> 50%)

**Expected Output**:

```
Generating LCOV coverage (excluding integration tests)...
...
Cover: X% (Y out of Z lines)
```

**What to Watch For**:

-   ❌ Timeout after 300 seconds
-   ❌ "Thread panicked" in integration tests
-   ❌ "Connection refused" (Milvus/Ollama)
-   ❌ Missing lcov.info file

---

#### 2. Path Filters - Docs-Only PR

**Objective**: Verify that CI workflow is SKIPPED for docs-only changes

**Setup**:

1.  Create branch from current commit
2.  Modify only documentation files
3.  Create PR to main

**Test Scenario**:

```bash
git checkout -b test/docs-only
echo "# Test docs" >> README.md
git add README.md
git commit -m "docs: test documentation-only PR"
```

**Files to Modify** (verify CI is SKIPPED):

-   [x] `README.md`
-   [x] `docs/**/*.md`
-   [x] Any file NOT in paths filter

**Success Criteria**:

-   [ ] CI workflow is NOT triggered
-   [ ] CodeQL is NOT triggered
-   [ ] Docs workflow IS triggered
-   [ ] PR shows "No checks running"

**What to Look For**:

-   ❌ Green checkmark for "CI" job
-   ❌ CodeQL analysis running
-   ✅ Only Docs pipeline should be visible

---

#### 3. Path Filters - Code PR

**Objective**: Verify that CI workflow RUNS for code changes

**Setup**:

1.  Create branch from current commit
2.  Modify a source file
3.  Create PR to main

**Test Scenario**:

```bash
git checkout -b test/code-change
echo "// test" >> crates/mcb/src/lib.rs
git add crates/mcb/src/lib.rs
git commit -m "test: verify CI runs on code changes"
```

**Files to Modify** (verify CI RUNS):

-   [x] `src/**`
-   [x] `crates/**`
-   [x] `tests/**`

**Success Criteria**:

-   [ ] CI workflow IS triggered
-   [ ] Lint job runs
-   [ ] Test-PR job runs (stable only)
-   [ ] Coverage job does NOT run (PR, not main)
-   [ ] Release-Build does NOT run (PR, not main)

**What to Look For**:

-   ✅ CI workflow triggered
-   ✅ Test-PR (stable) only
-   ❌ No Test-Main in PR
-   ❌ No Coverage in PR
-   ❌ No Release builds in PR

---

#### 4. Path Filters - Config Change

**Objective**: Verify that CI workflow RUNS for workflow config changes

**Setup**:

1.  Create branch from current commit
2.  Modify `.github/workflows/ci.yml`
3.  Create PR to main

**Test Scenario**:

```bash
git checkout -b test/workflow-change
echo "# test" >> .github/workflows/ci.yml
git add .github/workflows/ci.yml
git commit -m "test: verify CI runs on workflow changes"
```

**Success Criteria**:

-   [ ] CI workflow IS triggered (workflow file is in paths filter)
-   [ ] Lint job runs
-   [ ] Test-PR job runs

---

#### 5. Test Matrix - PR Branch

**Objective**: Verify test-pr runs in PR, test-main does NOT run

**Evidence**:

-   PR to main should show:
    -   ✅ Lint job
    -   ✅ Test-PR job with name "Test (PR - Stable)"
    -   ❌ NO Test-Main job
    -   ❌ NO Golden-Tests job
    -   ❌ NO Coverage job

**What NOT to See**:

-   ❌ "Test (Main - stable)"
-   ❌ "Test (Main - beta)"
-   ❌ "Golden Acceptance Tests"

---

#### 6. Test Matrix - Main Branch

**Objective**: Verify test-main runs on main, both stable and beta

**Setup**:

1.  Create branch from release/v0.1.4
2.  Push to main

**Evidence**:

-   Push to main should show:
    -   ✅ Lint job
    -   ✅ Test-Main jobs (stable AND beta - 2 matrix runs)
    -   ✅ Golden-Tests job
    -   ✅ Coverage job
    -   ✅ Release-Build jobs (3 platforms)

**What to Verify**:

-   ✅ Two test-main jobs (stable and beta)
-   ✅ Both complete before golden-tests starts
-   ✅ Coverage depends on test-main

---

#### 7. Coverage Execution - Main Branch

**Objective**: Verify coverage job completes on main without timeout

**Evidence**:

-   Coverage job in main push should:
    -   ✅ Install tarpaulin via setup-ci.sh --install-coverage
    -   ✅ Run `make coverage LCOV=1 TEST_THREADS=4`
    -   ✅ Upload to codecov
    -   ✅ Complete within 30-minute timeout

**Metrics to Check**:

-   Coverage % (should be > 50%)
-   Execution time (should be < 25 min)
-   Integration tests excluded from measurement

---

## Validation Execution Plan

### Phase A: Local Testing (Today)

-   [ ] Install tarpaulin
-   [ ] Run `make coverage` locally
-   [ ] Verify no timeouts
-   [ ] Check exclusion of integration tests

### Phase B: Dry-Run on release/v0.1.4

-   [ ] Create test PR from release/v0.1.4 to main with docs change
-   [ ] Verify CI is skipped
-   [ ] Create test PR with code change
-   [ ] Verify CI runs (test-pr only)

### Phase C: First Real PR/Push

-   [ ] Create real PR with code changes
-   [ ] Monitor CI execution
-   [ ] Watch for any failures
-   [ ] Validate timing meets expectations

### Phase D: Monitor Production

-   [ ] Track weekly CI job counts
-   [ ] Monitor coverage times
-   [ ] Check for false negatives
-   [ ] Gather metrics on 34% savings claim

## Risk Mitigation

**Risk**: Coverage timeouts again

-   **Mitigation**: Rollback to previous coverage target
-   **Detection**: Coverage job still running after 25 min
-   **Action**: Disable coverage job immediately, investigate

**Risk**: Path filters too restrictive (false negatives)

-   **Mitigation**: Monitor for bugs that slip through
-   **Detection**: Bug reported that wasn't caught by CI
-   **Action**: Expand path filters, add failing test

**Risk**: Path filters too permissive (not saving resources)

-   **Mitigation**: Still acceptable, filters can be tuned
-   **Detection**: CI still running for docs changes
-   **Action**: Add more specific path patterns

## Rollback Plan

If validations fail, rollback strategy:

```bash
# Revert the three optimization commits
git revert d211ffe    # Fix release-build dependency
git revert a29e1d6    # Phase 2 test matrix split
git revert 4c0ba12    # Phase 1 path filters

# Or reset to before optimizations
git reset --hard 4f7b792  # Before path filters
```

## Sign-Off Checklist

Before considering optimizations "production-ready":

-   [ ] Coverage runs locally without timeout
-   [ ] Test-PR runs in PRs, test-main does NOT
-   [ ] Path filters correctly skip docs-only changes
-   [ ] Path filters correctly run for code changes
-   [ ] Main branch runs comprehensive testing (stable+beta)
-   [ ] First real PR validates expectations
-   [ ] No false negatives in first week
-   [ ] Monthly job count shows 34%+ reduction

---

**Next Steps**: Start Phase A (Local Coverage Testing)
