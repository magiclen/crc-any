extern crate crc_any;
#[macro_use]
extern crate criterion;

use crc_any::CRC;
use criterion::{Benchmark, Criterion, Throughput};

fn crc16_construct(c: &mut Criterion) {
    c.bench_function("crc16_construct", |b| {
        // CRC16 IBM
        b.iter(|| CRC::create_crc(0x000000000000A001, 16, 0x0000000000000000, 0x0000000000000000, true))
    });
}

fn crc16_update_megabytes(c: &mut Criterion) {
    let mut crc = CRC::crc16ibm();
    let bytes = vec![0u8; 1_000_000];
    c.bench(
        "crc16_update_megabytes",
        Benchmark::new("crc16_update_megabytes", move |b| {
            b.iter(|| {
                crc.digest(&bytes[..]);
                crc.get_crc()
            })
        }).throughput(Throughput::Bytes(1_000_000)),
    );
}

fn crc32_construct(c: &mut Criterion) {
    c.bench_function("crc32_construct", |b| {
        b.iter(|| CRC::create_crc(0x00000000EDB88320, 32, 0x00000000FFFFFFFF, 0x00000000FFFFFFFF, true))
    });
}

fn crc32_update_megabytes(c: &mut Criterion) {
    let mut crc = CRC::crc32ieee();
    let bytes = vec![0u8; 1_000_000];
    c.bench(
        "crc32_update_megabytes",
        Benchmark::new("crc32_update_megabytes", move |b| {
            b.iter(|| {
                crc.digest(&bytes[..]);
                crc.get_crc()
            })
        }).throughput(Throughput::Bytes(1_000_000)),
    );
}

fn crc64_construct(c: &mut Criterion) {
    c.bench_function("crc64_construct", |b| {
        b.iter(|| CRC::create_crc(0xD800000000000000u64, 64, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, true))
    });
}

fn crc64_update_megabytes(c: &mut Criterion) {
    let mut crc = CRC::crc64iso();
    let bytes = vec![0u8; 1_000_000];
    c.bench(
        "crc64_update_megabytes",
        Benchmark::new("crc64_update_megabytes", move |b| {
            b.iter(|| {
                crc.digest(&bytes[..]);
                crc.get_crc()
            })
        }).throughput(Throughput::Bytes(1_000_000)),
    );
}

criterion_group!(crc16, crc16_construct, crc16_update_megabytes);
criterion_group!(crc32, crc32_construct, crc32_update_megabytes);
criterion_group!(crc64, crc64_construct, crc64_update_megabytes);
criterion_main!(crc16, crc32, crc64);
