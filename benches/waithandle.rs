use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn waithandle() {
    let (signaler, listener) = waithandle::new();
    let thread = thread::spawn({
        move || {
            while !listener.check().unwrap() {
                if listener.wait(Duration::from_secs(1)).unwrap() {
                    break;
                }
            }
        }
    });
    signaler.signal().unwrap();
    thread.join().unwrap();
}

fn channels() {
    struct Message {}
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
    let mut group = c.benchmark_group("Benchmarks");
    group.bench_function("Waithandle", |b| b.iter(|| waithandle()));
    group.bench_function("Channels", |b| b.iter(|| channels()));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
