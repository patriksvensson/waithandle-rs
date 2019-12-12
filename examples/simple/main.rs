use std::sync::Arc;
use std::thread;
use std::time::Duration;
use waithandle::{EventWaitHandle, WaitHandle};

fn main() {

    let handle = Arc::new(EventWaitHandle::new());
    let thread = thread::spawn(
    {
        let should_i_stop = handle.clone();

        move || {
            while !should_i_stop.check().unwrap() {
                println!("[WORK] Doing some work...");

                println!("[WORK] Waiting...");
                if should_i_stop.wait(std::time::Duration::from_secs(1)).unwrap() {
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
    handle.signal().unwrap();
    println!("[MAIN] Joining thread...");
    thread.join().unwrap();

    // We're all done.
    println!("[MAIN] Done!");
}
