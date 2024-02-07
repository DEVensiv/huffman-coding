use std::{fs::OpenOptions, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use huffman::{hdecode, hencode};
use tempfile::tempfile;

fn criterion_benchmark(c: &mut Criterion) {
    let mut integration = c.benchmark_group("integration");

    integration.bench_function("encode", |bencher| {
        bencher.iter(|| {
            let mut out = tempfile().expect("temfile err");
            let mut raw = OpenOptions::new()
                .read(true)
                .open("flake.lock")
                .expect("file err");
            hencode(&mut raw, &mut out).expect("io err");
        })
    });

    // make sure flake.lock.rxc exists
    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .open("flake.lock.rxc")
        .expect("file err");
    let mut raw = OpenOptions::new()
        .read(true)
        .open("flake.lock")
        .expect("file err");
    hencode(&mut raw, &mut out).expect("io err");

    integration.bench_function("decode", |bencher| {
        bencher.iter(|| {
            let mut out = tempfile().expect("temfile err");
            let raw = OpenOptions::new()
                .read(true)
                .open("flake.lock.rxc")
                .expect("file err");
            let mut reader = BufReader::new(raw);
            hdecode(&mut reader, &mut out).expect("io err");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
