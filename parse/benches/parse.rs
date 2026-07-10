//! Benchmarks for the core parse and write paths.
//!
//! Inputs are generated in-code via the library's own writer (see
//! `fixtures.rs`) so the benchmarks measure library performance only — no file
//! I/O and no bundled `.puz` files. A size sweep (5x5 / 15x15 / 21x21) plus
//! feature variants (rebus, circles, given) reveal both scaling and per-feature
//! cost.
//!
//! Run with `cargo bench`. Compare runs with `critcmp` by saving baselines:
//!     cargo bench -- --save-baseline before
//!     # ...make a change...
//!     cargo bench -- --save-baseline after
//!     critcmp before after

mod fixtures;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use puz_parse::{parse_bytes, to_bytes};
use std::hint::black_box;

fn bench_parse(c: &mut Criterion) {
    let fixtures = fixtures::all();
    let mut group = c.benchmark_group("parse");
    for f in &fixtures {
        group.throughput(Throughput::Bytes(f.bytes.len() as u64));
        group.bench_function(&f.name, |b| {
            b.iter(|| parse_bytes(black_box(&f.bytes)).unwrap())
        });
    }
    group.finish();
}

fn bench_write(c: &mut Criterion) {
    let fixtures = fixtures::all();
    let mut group = c.benchmark_group("write");
    for f in &fixtures {
        group.throughput(Throughput::Bytes(f.bytes.len() as u64));
        group.bench_function(&f.name, |b| {
            b.iter(|| to_bytes(black_box(&f.puzzle)).unwrap())
        });
    }
    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    let fixtures = fixtures::all();
    let mut group = c.benchmark_group("round_trip");
    for f in &fixtures {
        group.bench_function(&f.name, |b| {
            b.iter(|| {
                let bytes = to_bytes(black_box(&f.puzzle)).unwrap();
                parse_bytes(black_box(&bytes)).unwrap()
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_write, bench_round_trip);
criterion_main!(benches);
