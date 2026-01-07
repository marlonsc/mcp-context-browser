# daemon Module

**Source**: `src/daemon/`
**Files**: 1
**Lines of Code**: 308
**Traits**: 0
**Structs**: 3
**Enums**: 0
**Functions**: 0

## Overview

Background daemon for automatic lock cleanup and monitoring
//!
Provides continuous monitoring and maintenance services:
- Automatic cleanup of stale lockfiles
- Sync activity monitoring and reporting
- Background health checks

## Key Exports

`crate::sync::lockfile::{CodebaseLockManager, LockMetadata},crate::sync::manager::{SyncConfig, SyncManager, SyncStats},`

## File Structure

```text
mod.rs
```

---

*Auto-generated from source code on qua 07 jan 2026 18:27:27 -03*
