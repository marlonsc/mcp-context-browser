//! Lock utilities for proper error handling of poisoned locks

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::core::error::{Error, Result};

/// Lock a Mutex and handle poisoning
pub fn lock_mutex<'a, T>(lock: &'a Mutex<T>, context: &str) -> Result<MutexGuard<'a, T>> {
    lock.lock().map_err(|_| Error::internal(format!("Mutex lock poisoned: {}", context)))
}

/// Lock a RwLock for reading and handle poisoning
pub fn lock_rwlock_read<'a, T>(lock: &'a RwLock<T>, context: &str) -> Result<RwLockReadGuard<'a, T>> {
    lock.read().map_err(|_| Error::internal(format!("RwLock read lock poisoned: {}", context)))
}

/// Lock a RwLock for writing and handle poisoning
pub fn lock_rwlock_write<'a, T>(lock: &'a RwLock<T>, context: &str) -> Result<RwLockWriteGuard<'a, T>> {
    lock.write().map_err(|_| Error::internal(format!("RwLock write lock poisoned: {}", context)))
}
