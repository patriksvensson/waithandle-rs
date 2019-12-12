# Waithandle

A Rust library that makes signaling between threads more ergonomic than using [Condvar][2] directly.

Inspired by the .NET [System.Threading.EventWaitHandle][1] API. 
Uses [Condvar][2] and [Mutex][3] under the hood to block threads 
without consuming CPU time.

## Usage

```rust
use std::sync::Arc;
use std::time::Duration;
use waithandle::{EventWaitHandle, WaitHandle};

// Create a handle and wrap it
// in an Arc so we can share ownership
// of it with another thread.
let handle = Arc::new(EventWaitHandle::new());

// Signal a thread.
handle.signal()?;

// Did someone signal us?
if handle.check()? {
    println!("signal received");
}

// Wait for 5 seconds or until someone signals us.
if handle.wait(Duration::from_secs(5))? {
    println!("signal received");
}
```

## Running the example

```
> cargo run --example simple
```

```
[WORK] Doing some work...
[WORK] Doing some work...
[WORK] Doing some work...
[WORK] Doing some work...
[WORK] Doing some work...
[WORK] Waiting...
[MAIN] Signaling thread...
[MAIN] Joining thread...
[WORK] Someone told us to shut down!
[MAIN] Done!
```

[1]: https://docs.microsoft.com/en-us/dotnet/api/system.threading.eventwaithandle?view=netframework-4.8
[2]: https://doc.rust-lang.org/std/sync/struct.Condvar.html
[3]: https://doc.rust-lang.org/std/sync/struct.Mutex.html