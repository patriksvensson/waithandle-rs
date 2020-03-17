//use std::sync::Arc;
use std::thread;
use std::time::Duration;
use waithandle::{WaitHandle, WaitHandleSignaler};

fn main() {

    let (signaler, listener) = waithandle::make_pair();
    let thread = thread::spawn(
    {
        move || {
            while !listener.check().unwrap() {
                println!("[WORK] Doing some work...");

                println!("[WORK] Waiting...");
                if listener.wait(std::time::Duration::from_secs(1)).unwrap() {
                    println!("[WORK] Someone told us to shut down!");
                    break;
                }
            }
        }
    });

    // Sleep for 5 seconds.
    std::thread::sleep(Duration::from_secs(5));

    // Signal the thread to stop and 
    // then wait for the thread to join.
    println!("[MAIN] Signaling thread...");
    signaler.signal().unwrap();
    println!("[MAIN] Joining thread...");
    thread.join().unwrap();

    // We're all done.
    println!("[MAIN] Done!");
}
