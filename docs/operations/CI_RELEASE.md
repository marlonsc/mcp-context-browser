# CI/CD and Release Process

## Overview

This document describes the automated CI/CD pipeline and release process for MCP Context Browser. The system uses:

-   **Local Validation**: Git pre-commit hooks and `make` targets for fast feedback
-   **GitHub Actions**: Automated CI pipeline matching local validation
-   **Automated Releases**: Tag-based release workflow with binary artifacts

## Table of Contents

1.  [Local Validation (Pre-commit)](#local-validation-pre-commit)
2.  [CI Pipeline (GitHub Actions)](#ci-pipeline-github-actions)
3.  [Automated Release Deployment](#automated-release-deployment)
4.  [Test Timeout Management](#test-timeout-management)
5.  [Troubleshooting](#troubleshooting)

---

## Local Validation (Pre-commit)

### Installing Git Hooks

Install pre-commit hooks that validate code before each commit:

```bash
make install-hooks
```

This installs `.git/hooks/pre-commit` which runs validation checks automatically.

### What Pre-commit Validates

The pre-commit hook runs the same checks as the CI pipeline but **skips tests** for fast feedback (< 30 seconds typical):

```bash
# Step 1: Lint checks (Rust 2024 compliance)
make lint CI_MODE=1

# Step 2: Architecture validation (QUICK mode, no tests)
make validate QUICK=1
```

**Validation includes:**

-   Format check (rustfmt)
-   Clippy lints with Rust 2024 edition compatibility
-   Architecture validation (imports, dependencies, layer boundaries)
-   No test execution (tests run in CI after push)

### Running Pre-commit Manually

To run pre-commit validation without committing:

```bash
# Run exactly what pre-commit hook runs
make ci-local

# Or manually run lint + quick validate
make lint CI_MODE=1 && make validate QUICK=1
```

### Bypassing Pre-commit (Not Recommended)

If you need to bypass pre-commit checks temporarily:

```bash
git commit --no-verify
```

⚠️ **Warning**: The commit will still fail in GitHub CI if it doesn't pass validation.

---

## CI Pipeline (GitHub Actions)

### Pipeline Triggers

The CI pipeline runs automatically on:

1.  **Push to main or develop**: Runs on every push
2.  **Pull requests**: Validates changes before merge
3.  **Tag push**: Triggers release workflow (see below)

### CI Jobs and Dependencies

The `.github/workflows/ci.yml` defines the following jobs:

```
lint → test ──┬→ validate → release-build
              │
              ├→ golden-tests
              │
              ├→ audit
              │
              ├→ docs
              │
              └→ coverage
```

**Job Details:**

| Job | Purpose | Timeout | Prerequisites |
|-----|---------|---------|---------------|
| `lint` | Format & clippy checks | 15 min | None (runs first) |
| `test` | Unit & integration tests | 30 min | lint passes |
| `validate` | Architecture validation | 15 min | lint passes |
| `golden-tests` | Acceptance tests | 15 min | test passes |
| `audit` | Security audit (Cargo-audit) | 15 min | None (parallel) |
| `docs` | Documentation build | 15 min | None (parallel) |
| `coverage` | Code coverage | 30 min | test passes |
| `release-build` | Build artifacts (3 platforms) | 20 min | test + validate pass |

### How to Match CI Locally

To run the **exact same pipeline locally** before pushing:

```bash
# Full CI pipeline (matches GitHub exactly)
make ci-full

# This runs:
# 1. Lint (Rust 2024 compliance)
# 2. Unit and integration tests (4 threads to prevent timeouts)
# 3. Architecture validation (strict mode)
# 4. Golden acceptance tests (2 threads)
# 5. Security audit
# 6. Documentation build
```

### Viewing CI Results

Check GitHub Actions for detailed results:

```bash
# View latest workflow runs
gh run list --workflow=ci.yml -L 5

# View specific run details
gh run view <run-id> --log

# View job logs
gh run view <run-id> -j <job-name>
```

---

## Automated Release Deployment

### Release Triggers

Releases are triggered by pushing a version tag to main:

```bash
# Bump version (choose one)
make version BUMP=patch  # 0.1.2 → 0.1.3
make version BUMP=minor  # 0.1.2 → 0.2.0
make version BUMP=major  # 0.1.2 → 1.0.0

# Create git tag
git tag v$(make version | grep "Current version" | cut -d: -f2 | xargs)

# Or manually tag the current version
git tag v0.1.4

# Push tag (triggers release workflow)
git push origin v0.1.4
```

### Release Workflow (`.github/workflows/release.yml`)

The release workflow is triggered by tags matching `v*` pattern and performs:

1.  **Pre-Release Validation**: Runs full CI validation

-   Lint (Rust 2024 compliance)
-   Unit tests (4 threads)
-   Integration tests
-   Architecture validation (strict)
-   Security audit
-   Documentation build

1.  **Build Release Artifacts**: Compiles for all platforms

-   Linux: `mcb-x86_64-linux-gnu`
-   macOS: `mcb-x86_64-macos`
-   Windows: `mcb-x86_64-windows.exe`

1.  **Create GitHub Release**: Publishes release with:

-   Automatic changelog (git log since previous release)
-   All binary artifacts as downloads
-   Release notes from CHANGELOG.md

### Release Process Summary

```
Push tag v0.1.4 → GitHub detects tag
                 → Release workflow starts
                 → Validates: lint, test, validate, audit, docs ✓
                 → Builds: Linux, macOS, Windows binaries ✓
                 → Creates GitHub Release with artifacts ✓
                 → Release ready for download ✓
```

### Downloading Releases

Releases are available at:

```
https://github.com/marlonsc/mcb/releases
```

Each release includes:

-   Pre-compiled binaries for all platforms
-   Automatic changelog
-   Installation instructions

---

## Test Timeout Management

### Why Timeouts Happen

Tests can timeout in CI due to:

-   High parallelization (many tests running simultaneously)
-   GitHub runner resource constraints
-   Integration tests that take time
-   Network-dependent operations

### Timeout Configuration

The CI pipeline uses thread limiting to prevent timeouts:

```bash
# In GitHub Actions CI:
make test TEST_THREADS=4          # 4 parallel test threads (instead of auto)
make test SCOPE=golden TEST_THREADS=2  # Acceptance tests with 2 threads
```

### Available TEST_THREADS Values

| Value | Use Case | Default |
|-------|----------|---------|
| `0` | Auto (all available cores) | ✓ Local only |
| `1` | Sequential (slowest, fewest timeouts) | For slow CI |
| `2` | Limited parallelization | Acceptance tests |
| `4` | Balanced parallelization | Standard CI |
| `8+` | High parallelization | Local only |

### Adjusting Timeout Limits

**For local development:**

```bash
# Run tests with limited parallelization
make test TEST_THREADS=4

# Or use ci-full which already has good defaults
make ci-full
```

**For GitHub Actions CI:** Edit `.github/workflows/ci.yml` and adjust:

```yaml
- run: make test TEST_THREADS=4  # Increase/decrease as needed
```

### Monitoring Test Performance

Check CI logs for timeout issues:

```bash
# View test job logs
gh run view <run-id> -j test --log

# Look for timeout messages or slow tests
```

---

## Troubleshooting

### Pre-commit Hook Not Running

**Problem**: Commits don't run pre-commit validation

**Solution**:

```bash
# Reinstall hooks
make install-hooks

# Verify installation
cat .git/hooks/pre-commit
```

### CI Fails But Pre-commit Passed Locally

**Possible causes:**

1.  Different environment (macOS vs Linux)
2.  Different Rust versions
3.  Cache issues
4.  Race conditions in tests

**Solutions:**

```bash
# Run exact CI validation
make ci-full

# Clear cache and rebuild
make clean
cargo build

# Check Rust version matches
rustc --version  # Should be stable
```

### Tests Timeout in CI

**Problem**: `test` job timeout after 30 minutes

**Solutions:**

1.  **Increase timeout** in `.github/workflows/ci.yml`:

   ```yaml
   timeout-minutes: 45
   ```

1.  **Reduce parallelization**:

   ```yaml
   - run: make test TEST_THREADS=2
   ```

1.  **Run tests locally** to identify slow tests:

   ```bash
   make test TEST_THREADS=4
   ```

### Release Build Fails

**Problem**: `release-build` job fails with compilation error

**Check**:

1.  All CI checks passed before release job
2.  Built locally successfully

   ```bash
   make build RELEASE=1
   ```

1.  No uncommitted changes in version

   ```bash
   git status
   ```

### GitHub Release Not Created

**Problem**: Tag pushed but release workflow didn't complete

**Check**:

```bash
# View release workflow runs
gh run list --workflow=release.yml -L 5

# View specific run
gh run view <run-id> --log
```

**Common issues:**

-   Pre-release validation failed (check test/lint/audit logs)
-   Tag format incorrect (must be `v*` like `v0.1.4`)
-   Artifacts failed to upload

---

## CI Pipeline Commands Reference

### Local Commands

```bash
# Install pre-commit hooks
make install-hooks

# Pre-commit validation (lint + validate QUICK)
make ci-local

# Full CI pipeline (matches GitHub)
make ci-full

# Individual checks
make lint CI_MODE=1        # Format & clippy
make test                  # All tests
make validate STRICT=1     # Architecture validation
make audit                 # Security audit
make docs                  # Documentation
make coverage LCOV=1       # Code coverage
```

### GitHub Actions

Pipeline is automated. No manual intervention needed.

To view results:

```bash
gh run list --workflow=ci.yml
gh run view <run-id>
```

### Makefile Parameters

```bash
# Test parallelization
make test TEST_THREADS=4

# Specific test scopes
make test SCOPE=unit       # Unit tests only
make test SCOPE=doc        # Doctests only
make test SCOPE=golden     # Acceptance tests only
make test SCOPE=integration  # Integration tests only

# Lint modes
make lint CI_MODE=1        # Rust 2024 compatibility checks
make lint FIX=1            # Auto-fix issues

# Validation modes
make validate STRICT=1     # Strict validation
make validate QUICK=1      # Skip tests

# Release
make version BUMP=patch    # Show next patch version
make release               # Full release pipeline
```

---

## See Also

-   [Deployment Guide](./DEPLOYMENT.md) - Installation and configuration
-   [CHANGELOG](./CHANGELOG.md) - Release history
-   [Architecture](../architecture/ARCHITECTURE.md) - System design
