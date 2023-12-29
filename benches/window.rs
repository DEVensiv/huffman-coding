use std::{hint::black_box, io::BufReader, thread, time::Duration};

use bitreader::BitReader;
use bitstream_io::{BitRead, LittleEndian};
use criterion::{criterion_group, criterion_main, Criterion};
use huffman::window::BitWindow;

const SOURCE_BYTES: usize = 1000 * 3;

fn criterion_benchmark(c: &mut Criterion) {
    let mut setup = c.benchmark_group("setup");

    setup.bench_function("bitstream setup", |b| {
        b.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader: bitstream_io::BitReader<&[u8], LittleEndian> =
                bitstream_io::BitReader::new(slice_of_u8);
            black_box(reader)
        })
    });

    println!("wait 3 s to allow cpu cool");
    thread::sleep(Duration::from_secs(3));

    setup.bench_function("bitreader setup", |b| {
        b.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader = BitReader::new(slice_of_u8);
            black_box(reader)
        })
    });

    println!("wait 3 s to allow cpu cool");
    thread::sleep(Duration::from_secs(3));

    setup.bench_function("window setup", |b| {
        b.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            black_box(reader)
        })
    });

    setup.finish();
    let mut runtime = c.benchmark_group("runtime");

    println!("wait 5 s to allow cpu cool");
    thread::sleep(Duration::from_secs(5));

    runtime.bench_function("bitstream", |bencher| {
        bencher.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader: bitstream_io::BitReader<BufReader<&[u8]>, LittleEndian> =
                bitstream_io::BitReader::new(BufReader::new(slice_of_u8));
            bench_bitstream(reader, slice_of_u8.len());
        })
    });

    println!("wait 5 s to allow cpu cool");
    thread::sleep(Duration::from_secs(5));

    runtime.bench_function("bitreader", |bencher| {
        bencher.iter(|| {
            let slice_of_u8 = &[0b1000_1111; SOURCE_BYTES];
            let reader = BitReader::new(slice_of_u8);
            bench_reader_optimal(reader, slice_of_u8.len());
        })
    });

    println!("wait 5 s to allow cpu cool");
    thread::sleep(Duration::from_secs(5));

    runtime.bench_function("window", |bencher| {
        bencher.iter(|| {
            let slice_of_u8: &[u8] = &[0b1000_1111; SOURCE_BYTES];
            let reader: BitWindow<_> = slice_of_u8.into();
            bench_window_reader(reader, slice_of_u8.len());
        })
    });
}

fn bench_bitstream(mut reader: impl BitRead, bytes: usize) {
    for _ in 0..(bytes / 3) {
        // You obviously should use try! or some other error handling mechanism here
        let _: u8 = reader.read(1).unwrap();
        let _: u8 = reader.read(2).unwrap();
        let _: u8 = reader.read(3).unwrap();
        let _: u8 = reader.read(6).unwrap();

        let _: u8 = reader.read(1).unwrap();
        let _: u8 = reader.read(2).unwrap();
        let _: u8 = reader.read(3).unwrap();
        let _: u8 = reader.read(6).unwrap();
    }
}

fn bench_reader_optimal(mut reader: BitReader, bytes: usize) {
    for _ in 0..(bytes / 3) {
        let _ = black_box(reader.read_u8(1).unwrap()); // 1
        let _ = black_box(reader.read_u8(2).unwrap()); // 1
        let _ = black_box(reader.read_u8(3).unwrap()); // 0
        let _ = black_box(reader.read_u8(6).unwrap()); // 0b1111

        let _ = black_box(reader.read_u8(1).unwrap()); // 1
        let _ = black_box(reader.read_u8(2).unwrap()); // 1
        let _ = black_box(reader.read_u8(3).unwrap()); // 0
        let _ = black_box(reader.read_u8(6).unwrap()); // 0b1111
    }
}

fn bench_window_reader(mut reader: BitWindow<&[u8]>, bytes: usize) {
    for _ in 0..(bytes / 3) {
        // You obviously should use try! or some other error handling mechanism here
        let _ = black_box(reader.show_exact(1));
        reader.consume(1).unwrap();
        let _ = black_box(reader.show_exact(2));
        reader.consume(2).unwrap();
        let _ = black_box(reader.show_exact(3));
        reader.consume(3).unwrap();
        let _ = black_box(reader.show_exact(6));
        reader.consume(6).unwrap();

        let _ = black_box(reader.show_exact(1));
        reader.consume(1).unwrap();
        let _ = black_box(reader.show_exact(2));
        reader.consume(2).unwrap();
        let _ = black_box(reader.show_exact(3));
        reader.consume(3).unwrap();
        let _ = black_box(reader.show_exact(6));
        reader.consume(6).unwrap();
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
