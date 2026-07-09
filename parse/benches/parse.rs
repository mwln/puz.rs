//! Benchmarks for the parse and write paths.
//!
//! Run with `cargo bench`. Compare runs with `critcmp` by saving baselines:
//!     cargo bench -- --save-baseline before
//!     # ...make a change...
//!     cargo bench -- --save-baseline after
//!     critcmp before after

use criterion::{criterion_group, criterion_main, Criterion};
use puz_parse::{parse_bytes, to_bytes};
use std::hint::black_box;

// Fixtures embedded at compile time (they are excluded from the published
// package, so we can't read them from disk in all contexts).
const STANDARD: &[u8] = include_bytes!("../examples/data/standard1.puz");
const REBUS: &[u8] = include_bytes!("../examples/data/rebus.puz");
const CIRCLED: &[u8] = include_bytes!("../examples/data/circled.puz");

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    for (name, data) in [
        ("standard", STANDARD),
        ("rebus", REBUS),
        ("circled", CIRCLED),
    ] {
        group.bench_function(name, |b| b.iter(|| parse_bytes(black_box(data)).unwrap()));
    }
    group.finish();
}

fn bench_write(c: &mut Criterion) {
    // Parse once up front; benchmark only the write path.
    let puzzle = parse_bytes(STANDARD).unwrap();
    c.bench_function("write/standard", |b| {
        b.iter(|| to_bytes(black_box(&puzzle)).unwrap())
    });
}

fn bench_round_trip(c: &mut Criterion) {
    let puzzle = parse_bytes(STANDARD).unwrap();
    c.bench_function("round_trip/standard", |b| {
        b.iter(|| {
            let bytes = to_bytes(black_box(&puzzle)).unwrap();
            parse_bytes(black_box(&bytes)).unwrap()
        })
    });
}

criterion_group!(benches, bench_parse, bench_write, bench_round_trip);
criterion_main!(benches);
