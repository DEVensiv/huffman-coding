use std::time::Duration;

use bitreader::BitReader;
use bitstream_io::{BigEndian, BitRead};
use criterion::{criterion_main, BenchmarkId, Criterion};
use huffman::window::BitWindow;

const SOURCE_BYTES: usize = 1000 * 3;

fn criterion_benchmark(c: &mut Criterion) {
    let mut setup = c.benchmark_group("setup");

    setup.bench_function("bitstream", |b| {
        b.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader: bitstream_io::BitReader<&[u8], BigEndian> =
                bitstream_io::BitReader::new(slice_of_u8);
            reader
        })
    });

    setup.bench_function("bitreader", |b| {
        b.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader = BitReader::new(slice_of_u8);
            reader
        })
    });

    setup.bench_function("window", |b| {
        b.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            reader
        })
    });

    setup.finish();

    let mut single_dyn_read = c.benchmark_group("single_dyn_read");

    for bits in 1..9 {
        // single_dyn_read.bench_with_input(BenchmarkId::new("bitreader", bits), &bits, |b, input| {
        //     b.iter(|| {
        //         let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
        //         let mut reader = BitReader::new(slice_of_u8);
        //         reader.read_u8(*input as u8).unwrap()
        //     })
        // });

        single_dyn_read.bench_with_input(BenchmarkId::new("bitstream", bits), &bits, |b, input| {
            b.iter(|| {
                let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
                let mut reader: bitstream_io::BitReader<&[u8], BigEndian> =
                    bitstream_io::BitReader::new(slice_of_u8);
                reader.read::<u8>(*input as u32).unwrap()
            })
        });

        single_dyn_read.bench_with_input(BenchmarkId::new("window", bits), &bits, |b, input| {
            b.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let mut reader: BitWindow<_> = slice_of_u8.into();
                let bits = reader.show_exact(*input as usize);
                reader.consume(*input).unwrap();
                bits
            })
        });
    }
    single_dyn_read.finish();

    let mut multi_const_read = c.benchmark_group("multi_const_read");

    for bytes in 0..5 {
        let bytes = bytes * 10;
        // multi_const_read.bench_with_input(
        //     BenchmarkId::new("bitreader", bytes),
        //     &bytes,
        //     |b, input| {
        //         b.iter(|| {
        //             let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
        //             let mut reader = BitReader::new(slice_of_u8);
        //             let mut acc = [0; 40];
        //             for i in acc.iter_mut().take(*input) {
        //                 *i = reader.read_u8(8).unwrap();
        //             }
        //             acc
        //         })
        //     },
        // );

        multi_const_read.bench_with_input(
            BenchmarkId::new("bitstream", bytes),
            &bytes,
            |b, input| {
                b.iter(|| {
                    let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
                    let mut reader: bitstream_io::BitReader<&[u8], BigEndian> =
                        bitstream_io::BitReader::new(slice_of_u8);
                    let mut acc = [0; 40];
                    for i in acc.iter_mut().take(*input) {
                        *i = reader.read::<u8>(8).unwrap();
                    }
                    acc
                })
            },
        );

        multi_const_read.bench_with_input(BenchmarkId::new("window, const fn", bytes), &bytes, |b, input| {
            b.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let mut reader: BitWindow<_> = slice_of_u8.into();
                let mut acc = [0; 40];
                for i in acc.iter_mut().take(*input) {
                    *i = reader.show_u8();
                    reader.consume(8).unwrap();
                }
                acc
            })
        });

        multi_const_read.bench_with_input(BenchmarkId::new("window", bytes), &bytes, |b, input| {
            b.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let mut reader: BitWindow<_> = slice_of_u8.into();
                let mut acc = [0; 40];
                for i in acc.iter_mut().take(*input) {
                    *i = reader.show_exact(8);
                    reader.consume(8).unwrap();
                }
                acc
            })
        });
    }
    multi_const_read.finish();

    let mut multi_const_read_unaligned = c.benchmark_group("multi_const_read_unaligned");

    for bytes in 0..5 {
        let bytes = bytes * 10;
        // multi_const_read_unaligned.bench_with_input(
        //     BenchmarkId::new("bitreader", bytes),
        //     &bytes,
        //     |b, input| {
        //         b.iter(|| {
        //             let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
        //             let mut reader = BitReader::new(slice_of_u8);
        //             let mut acc = [0; 40];
        //             for i in acc.iter_mut().take(*input) {
        //                 *i = reader.read_u8(3).unwrap();
        //             }
        //             acc
        //         })
        //     },
        // );

        multi_const_read_unaligned.bench_with_input(
            BenchmarkId::new("bitstream", bytes),
            &bytes,
            |b, input| {
                b.iter(|| {
                    let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
                    let mut reader: bitstream_io::BitReader<&[u8], BigEndian> =
                        bitstream_io::BitReader::new(slice_of_u8);
                    let mut acc = [0; 40];
                    for i in acc.iter_mut().take(*input) {
                        *i = reader.read::<u8>(3).unwrap();
                    }
                    acc
                })
            },
        );

        multi_const_read_unaligned.bench_with_input(BenchmarkId::new("window", bytes), &bytes, |b, input| {
            b.iter(|| {
                let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
                let mut reader: BitWindow<_> = slice_of_u8.into();
                let mut acc = [0; 40];
                for i in acc.iter_mut().take(*input) {
                    *i = reader.show_exact(3);
                    reader.consume(3).unwrap();
                }
                acc
            })
        });
    }
    multi_const_read_unaligned.finish();

}

pub fn benches() {
    let mut criterion = Criterion::default()
        .configure_from_args()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(1));
    criterion_benchmark(&mut criterion);
}
criterion_main!(benches);
