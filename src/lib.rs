#![allow(clippy::mutex_atomic)]

//! A library that makes signaling between threads a bit more ergonomic
//! than using a `CondVar` + `Mutex` directly.
//!
//! # Examples
//!
//! ```rust
//! use std::thread;
//! use std::time::Duration;
//!
//! let (signaler, listener) = waithandle::new();
//!
//! let thread = thread::spawn({
//!     move || {
//!         while !listener.check() {
//!             println!("Doing some work...");
//!
//!             // Wait for 1 second or until we receive a signal
//!             if listener.wait(Duration::from_secs(1)) {
//!                 println!("Someone told us to quit!");
//!                 break;
//!             }
//!         }
//!     }
//! });
//!
//! // Sleep for 5 seconds.
//! thread::sleep(Duration::from_secs(5));
//!
//! println!("Signaling thread...");
//! signaler.signal();
//!
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

#[derive(Debug, Default, Clone)]
struct WaitHandle {
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl WaitHandle {
    pub fn new() -> Self {
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        return WaitHandle { pair };
    }

    pub fn check(&self) -> WaitHandleResult<bool> {
        self.wait(Duration::from_micros(0))
    }

    pub fn wait(&self, timeout: Duration) -> WaitHandleResult<bool> {
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock()?;
        let result = cvar.wait_timeout_while(guard, timeout, |&mut pending| !pending)?;
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
        let (lock, cvar) = &*self.pair;
        let mut guard = lock.lock()?;
        if *guard != value {
            *guard = value;
            cvar.notify_one();
        }
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Signaler

/// The signaling half of a wait handle.
#[derive(Debug, Clone)]
pub struct WaitHandleSignaler {
    handle: Arc<WaitHandle>,
}

impl WaitHandleSignaler {
    fn new(handle: Arc<WaitHandle>) -> Self {
        Self { handle }
    }

    /// Resets the wait handle
    pub fn reset(&self) {
        self.try_reset().expect("error occured while resetting wait handle")
    }

    /// Tries to reset the wait handle
    pub fn try_reset(&self) -> WaitHandleResult<()> {
        self.handle.reset()
    }

    /// Signals the wait handle
    pub fn signal(&self) {
        self.try_signal().expect("error occured while signaling wait handle")
    }

    /// Tries to signal the wait handle
    pub fn try_signal(&self) -> WaitHandleResult<()> {
        self.handle.signal()
    }
}

///////////////////////////////////////////////////////////
// Listener

/// The listening half of a wait handle.
#[derive(Debug, Clone)]
pub struct WaitHandleListener {
    handle: Arc<WaitHandle>,
}

impl WaitHandleListener {
    fn new(handle: Arc<WaitHandle>) -> Self {
        Self { handle }
    }

    /// Checks whether or not the wait handle have been signaled.
    pub fn check(&self) -> bool {
        self.try_check().expect("an error occured while checking wait handle")
    }

    /// Tries checking whether or not the wait handle have been signaled.
    pub fn try_check(&self) -> WaitHandleResult<bool> {
        self.handle.check()
    }

    /// Waits until the wait handle have been signaled or the timeout occur,
    /// whichever comes first.
    pub fn wait(&self, timeout: Duration) -> bool {
        self.try_wait(timeout).expect("an error occured while waiting for wait handle")
    }

    /// Tries waiting until the wait handle have been signaled or the timeout occur,
    /// whichever comes first.
    pub fn try_wait(&self, timeout: Duration) -> WaitHandleResult<bool> {
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
