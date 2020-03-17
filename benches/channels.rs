use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

struct Message {}

fn with_channels() {
    let (tx, rx) = channel::<Message>();
    let thread = thread::spawn({
        move || loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) => break,
                Err(_) => {}
            }
        }
    });

    tx.send(Message {}).unwrap();
    thread.join().unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("channels", |b| b.iter(|| with_channels()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
