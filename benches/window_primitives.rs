use std::time::Duration;

use criterion::{criterion_main, BenchmarkId, Criterion};
use huffman::window::BitWindow;

const SOURCE_BYTES: usize = 1000 * 3;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("show_u8", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            reader.show_u8()
        })
    });

    c.bench_function("show_exact(8)", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            reader.show_exact(8)
        })
    });

    let mut show_diff = c.benchmark_group("show_exact(diff)");
    for input in 1..8 {
        show_diff.bench_with_input(
            BenchmarkId::new("show_exact(diff)", input),
            &input,
            |bencher, input| {
                bencher.iter(|| {
                    let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                    let reader: BitWindow<_> = slice_of_u8.into();
                    reader.show_exact(*input)
                })
            },
        );
    }
    show_diff.finish();

    c.bench_function("consume", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            consume(reader, slice_of_u8.len())
        })
    });
}

fn consume(mut reader: BitWindow<&[u8]>, bytes: usize) {
    for _ in 0..(bytes / 3) {
        // You obviously should use try! or some other error handling mechanism here
        reader.consume(1).unwrap();
        reader.consume(2).unwrap();
        reader.consume(3).unwrap();
        reader.consume(6).unwrap();

        reader.consume(1).unwrap();
        reader.consume(2).unwrap();
        reader.consume(3).unwrap();
        reader.consume(6).unwrap();
    }
}

pub fn benches() {
    let mut criterion = Criterion::default()
        .configure_from_args()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(1));
    criterion_benchmark(&mut criterion);
}
criterion_main!(benches);
