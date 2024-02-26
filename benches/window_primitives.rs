use std::time::Duration;

use criterion::{criterion_main, BenchmarkId, Criterion};
use huffman::window::BitWindow;

const SOURCE_BYTES: usize = 50;

fn criterion_benchmark(c: &mut Criterion) {
    let mut show_exact = c.benchmark_group("show(diff)");
    for input in 1..8 {
        show_exact.bench_with_input(BenchmarkId::new("", input), &input, |bencher, input| {
            bencher.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let reader: BitWindow<_> = slice_of_u8.into();
                reader.show(*input)
            })
        });
    }
    show_exact.finish();

    let mut consume = c.benchmark_group("consume_bits");
    for consumes in 0..5 {
        consume.bench_with_input(
            BenchmarkId::new("", consumes),
            &consumes,
            |bencher, input| {
                bencher.iter(|| {
                    let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                    let mut reader: BitWindow<_> = slice_of_u8.into();
                    reader.consume(*input).unwrap();
                    reader
                })
            },
        );
    }
    consume.finish();
    let mut consume = c.benchmark_group("consume_full");
    for consumes in 0..5 {
        let consumes = consumes * 10;
        consume.bench_with_input(
            BenchmarkId::new("aligned", consumes),
            &consumes,
            |bencher, input| {
                bencher.iter(|| {
                    let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                    let mut reader: BitWindow<_> = slice_of_u8.into();
                    for _ in 0..*input {
                        reader.consume(8).unwrap();
                    }
                    reader
                })
            },
        );
    }
    for consumes in 0..5 {
        let consumes = consumes * 10;
        consume.bench_with_input(
            BenchmarkId::new("unaligned", consumes),
            &consumes,
            |bencher, input| {
                bencher.iter(|| {
                    let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                    let mut reader: BitWindow<_> = slice_of_u8.into();
                    // consume 3 bits to disalign the reader
                    reader.consume(3).unwrap();
                    for _ in 0..*input {
                        reader.consume(8).unwrap();
                    }
                    reader
                })
            },
        );
    }
    consume.finish();
}

pub fn benches() {
    let mut criterion = Criterion::default()
        .configure_from_args()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(1));
    criterion_benchmark(&mut criterion);
}
criterion_main!(benches);
