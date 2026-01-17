//! Timing Utility Tests

use mcb_infrastructure::utils::TimedOperation;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_timed_operation() {
    let timer = TimedOperation::start();
    sleep(Duration::from_millis(10));
    assert!(timer.elapsed_ms() >= 10);
}

#[test]
fn test_elapsed_secs() {
    let timer = TimedOperation::start();
    sleep(Duration::from_millis(100));
    assert!(timer.elapsed_secs() >= 0.1);
}
