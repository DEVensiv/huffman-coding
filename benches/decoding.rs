use std::{fs::OpenOptions, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use huffman::hdecode;
use tempfile::tempfile;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decode", |bencher| {
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
