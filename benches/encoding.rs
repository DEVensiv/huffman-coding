use std::{fs, io::BufReader};

use criterion::{criterion_group, criterion_main, Criterion};
use huffman::{hdecode, hencode};

fn criterion_benchmark(c: &mut Criterion) {
    let raw = fs::read("flake.lock").expect("io err");
    let encoded = fs::read("flake.lock.rxc").expect("io err");
    c.bench_function("encode", |bencher| {
        bencher.iter(|| {
            let mut out: Vec<u8> = Vec::new();
            hencode(&mut BufReader::<&[u8]>::new(&raw), &mut out).expect("io err");
        })
    });

    c.bench_function("decode", |bencher| {
        bencher.iter(|| {
            let mut out: Vec<u8> = Vec::new();
            hdecode(&mut BufReader::<&[u8]>::new(&encoded), &mut out).expect("io err");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
