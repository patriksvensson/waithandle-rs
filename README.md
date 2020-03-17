# Waithandle

A Rust library that makes signaling between threads a bit more ergonomic.

Uses [Condvar][1] and [Mutex][2] under the hood to block threads 
without consuming CPU time.

## Usage

```rust
use std::sync::Arc;
use std::time::Duration;

// Create the signaler and the listener
let (signaler, listener) = waithandle::new();

// Signal a thread
signaler.signal()?;

// Did someone signal us?
if listener.check()? {
    println!("signal received");
}

// Wait for 5 seconds or until someone signals us
if listener.wait(Duration::from_secs(5))? {
    println!("signal received");
}
```

## Running the example

```
> cargo run --example simple
```

```
Doing some work...
Doing some work...
Doing some work...
Doing some work...
Doing some work...
Signaling thread...
Joining thread...
Someone told us to exit!
Done!
```

[1]: https://doc.rust-lang.org/std/sync/struct.Condvar.html
[2]: https://doc.rust-lang.org/std/sync/struct.Mutex.html