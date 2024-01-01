use std::time::Duration;

use criterion::{criterion_main, BenchmarkId, Criterion};
use huffman::window::BitWindow;

const SOURCE_BYTES: usize = 40;

fn criterion_benchmark(c: &mut Criterion) {

    let mut show = c.benchmark_group("show");
    show.bench_function("show_u8", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            reader.show_u8()
        })
    });

    show.bench_function("show_exact(8)", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            reader.show_exact(8)
        })
    });
    show.finish();

    let mut show_exact = c.benchmark_group("show_exact(diff)");
    for input in 1..8 {
        show_exact.bench_with_input(BenchmarkId::new("", input), &input, |bencher, input| {
            bencher.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let reader: BitWindow<_> = slice_of_u8.into();
                reader.show_exact(*input)
            })
        });
    }
    show_exact.finish();

    let mut consume = c.benchmark_group("consume_small");
    for consumes in 0..5 {
        consume.bench_with_input(
            BenchmarkId::new("", consumes),
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
    consume.finish();
    let mut consume = c.benchmark_group("consume_big");
    for consumes in 1..5 {
        let consumes = consumes * 10;
        consume.bench_with_input(
            BenchmarkId::new("", consumes),
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
