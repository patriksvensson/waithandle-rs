//use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let (signaler, listener) = waithandle::new();
    let thread = thread::spawn({
        move || {
            while !listener.check().unwrap() {
                println!("Doing some work...");

                println!("Waiting...");
                if listener.wait(Duration::from_secs(1)).unwrap() {
                    println!("Someone told us to shut down!");
                    break;
                }
            }
        }
    });

    // Sleep for 5 seconds.
    std::thread::sleep(Duration::from_secs(5));

    // Signal the thread to stop and then wait for the thread to join.
    println!("Signaling thread...");
    signaler.signal().unwrap();
    println!("Joining thread...");
    thread.join().unwrap();

    // We're all done.
    println!("Done!");
}
