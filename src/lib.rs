#![allow(clippy::mutex_atomic)]

//! A library that makes signaling between threads a bit more ergonomic
//! than using a `CondVar` + `Mutex` directly.
//!
//! # Examples
//!
//! ```rust
//! use std::thread;
//!
//! let (signaler, listener) = waithandle::new();
//!
//! let thread = thread::spawn({
//!     move || {
//!         while !listener.check().unwrap() {
//!             println!("Doing some work...");
//!             if listener.wait(Duration::from_secs(1)).unwrap() {
//!                 println!("Someone told us to exit!");
//!                 break;
//!             }
//!         }
//!     }
//! });
//!
//! thread::sleep(Duration::from_secs(5));
//!
//! println!("Signaling thread...");
//! signaler.signal().unwrap();
//! println!("Joining thread...");
//! thread.join().unwrap();
//! ```

use std::error;
use std::fmt;
use std::fmt::Formatter;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// The result of a wait handle operation.
pub type WaitHandleResult<T> = std::result::Result<T, WaitHandleError>;

///////////////////////////////////////////////////////////
// Constructor

/// Creates a wait handle pair for signaling and listening.
pub fn new() -> (WaitHandleSignaler, WaitHandleListener) {
    let wait_handle = Arc::new(WaitHandle::new());
    let signaler = WaitHandleSignaler::new(wait_handle.clone());
    let listener = WaitHandleListener::new(wait_handle);
    (signaler, listener)
}

///////////////////////////////////////////////////////////
// Wait handle

#[derive(Debug, Default)]
struct WaitHandle {
    mutex: Mutex<bool>,
    cond: Condvar,
}

impl WaitHandle {
    /// Creates a new wait handle.
    pub fn new() -> Self {
        return WaitHandle {
            mutex: Mutex::new(false),
            cond: Condvar::new(),
        };
    }

    pub fn check(&self) -> WaitHandleResult<bool> {
        self.wait(Duration::from_micros(0))
    }

    pub fn wait(&self, timeout: Duration) -> WaitHandleResult<bool> {
        let mut guard = self.mutex.lock()?;
        let result = self.cond.wait_timeout(guard, timeout)?;
        guard = result.0;
        if *guard {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn reset(&self) -> WaitHandleResult<()> {
        self.set(false)
    }

    pub fn signal(&self) -> WaitHandleResult<()> {
        self.set(true)
    }

    fn set(&self, value: bool) -> WaitHandleResult<()> {
        let mut lock = self.mutex.lock()?;
        if *lock != value {
            *lock = value;
            self.cond.notify_all();
        }
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Signaler

/// The signaling half of a wait handle.
#[derive(Clone)]
pub struct WaitHandleSignaler {
    handle: Arc<WaitHandle>,
}

impl WaitHandleSignaler {
    fn new(handle: Arc<WaitHandle>) -> Self {
        Self { handle }
    }

    pub fn reset(&self) -> WaitHandleResult<()> {
        self.handle.reset()
    }

    pub fn signal(&self) -> WaitHandleResult<()> {
        self.handle.signal()
    }
}

///////////////////////////////////////////////////////////
// Listener

/// The listening half of a wait handle.
#[derive(Clone)]
pub struct WaitHandleListener {
    handle: Arc<WaitHandle>,
}

impl WaitHandleListener {
    fn new(handle: Arc<WaitHandle>) -> Self {
        Self { handle }
    }

    pub fn check(&self) -> WaitHandleResult<bool> {
        self.handle.check()
    }

    pub fn wait(&self, timeout: Duration) -> WaitHandleResult<bool> {
        self.handle.wait(timeout)
    }
}

///////////////////////////////////////////////////////////
// Errors

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
        None
    }
}

impl<T> From<PoisonError<T>> for WaitHandleError {
    fn from(_: PoisonError<T>) -> Self {
        WaitHandleError::LockPoisoned
    }
}
