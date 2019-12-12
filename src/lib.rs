//! A library that makes signaling 
//! between threads more ergonomic.

#![allow(clippy::needless_return)]
#![allow(clippy::mutex_atomic)]

use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

/// Represents a wait handle.
pub trait WaitHandle {
    /// Checks if the current wait handle has
    /// received a signal.
    fn check(&self) -> WaitHandleResult<bool>;

    /// Blocks the current thread until the current
    /// wait handle receives a signal or waiting times out.
    fn wait(&self, timeout: Duration) -> WaitHandleResult<bool>;
}

/// Represents a thread synchronization event.
#[derive(Debug, Default)]
pub struct EventWaitHandle {
    mutex: Mutex<bool>,
    cond: Condvar,
}

/// The result of wait handle operations.
pub type WaitHandleResult<T> = std::result::Result<T, WaitHandleError>;

/// Represents a wait handle error.
#[derive(Debug, Clone)]
pub enum WaitHandleError {
    LockPoisoned,
}

impl fmt::Display for WaitHandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WaitHandleError::LockPoisoned => write!(f, "wait handle lock poisoned"),
        }
    }
}
impl error::Error for WaitHandleError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        return None;
    }
}
impl<T> From<PoisonError<T>> for WaitHandleError {
    fn from(_: PoisonError<T>) -> Self {
        return WaitHandleError::LockPoisoned;
    }
}

impl EventWaitHandle {
    /// Creates a new wait handle.
    pub fn new() -> Self {
        return EventWaitHandle {
            mutex: Mutex::new(false),
            cond: Condvar::new(),
        };
    }

    /// Sets the state of the event to nonsignaled,
    /// causing threads to block.
    pub fn reset(&self) -> WaitHandleResult<()> {
        return self.set(false);
    }

    /// Sets the state of the event to signaled,
    /// allowing one or more waiting threads to proceed.
    pub fn signal(&self) -> WaitHandleResult<()> {
        return self.set(true);
    }

    fn set(&self, value: bool) -> WaitHandleResult<()> {
        let mut lock = self.mutex.lock()?;
        if *lock != value {
            *lock = value;
            self.cond.notify_all();
        }
        return Ok(());
    }
}

impl WaitHandle for EventWaitHandle {
    fn check(&self) -> WaitHandleResult<bool> {
        return self.wait(Duration::from_micros(0));
    }

    fn wait(&self, timeout: Duration) -> WaitHandleResult<bool> {
        let mut guard = self.mutex.lock()?;
        let result = self.cond.wait_timeout(guard, timeout)?;
        guard = result.0;
        if *guard {
            return Ok(true);
        }
        return Ok(false);
    }
}