//! Timing instrumentation helpers
//!
//! Provides reusable timing patterns for operation tracking.

use std::time::{Duration, Instant};

/// Timing instrumentation helper - tracks operation elapsed time
///
/// # Example
///
/// ```ignore
/// use mcb_infrastructure::utils::TimedOperation;
///
/// let timer = TimedOperation::start();
/// // Perform operation
/// let elapsed = timer.elapsed_ms();
/// ```
pub struct TimedOperation {
    start: Instant,
}

impl TimedOperation {
    /// Start a new timed operation
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Get elapsed time as Duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get remaining time before deadline (returns None if already exceeded)
    pub fn remaining(&self, deadline: Duration) -> Option<Duration> {
        deadline.checked_sub(self.start.elapsed())
    }
}
