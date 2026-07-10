//! In-code fixture generation for benchmarks.
//!
//! Rather than bundling `.puz` files, we build `Puzzle` values programmatically
//! and use the library's own writer (`to_bytes`) to produce the byte buffers we
//! parse. This keeps benchmarks focused on core-library performance (no file
//! I/O, no bundled fixtures) and lets us vary size and features freely.
//!
//! Note: benchmarks only *time* code, they don't assert correctness, so using
//! the writer to generate parser inputs is fine here — a shared bug would show
//! up in the correctness test suite, not silently pass a benchmark.

use puz_parse::{Clues, Extensions, Grid, Puzzle, PuzzleInfo, Rebus};
use std::collections::HashMap;

/// A generated puzzle plus its serialized bytes, ready for benchmarking either
/// direction.
#[derive(Debug)]
pub(crate) struct Fixture {
    pub(crate) name: String,
    pub(crate) puzzle: Puzzle,
    pub(crate) bytes: Vec<u8>,
}

/// Build the standard set of benchmark fixtures: a size sweep plus
/// feature-specific variants (rebus, circles, given).
pub(crate) fn all() -> Vec<Fixture> {
    let mut out = Vec::new();

    // Size sweep on plain (all-open) grids.
    for (name, n) in [
        ("small_5x5", 5),
        ("standard_15x15", 15),
        ("large_21x21", 21),
    ] {
        out.push(build(name, plain(n)));
    }

    // Feature variants on a standard-sized grid.
    out.push(build("rebus_15x15", with_rebus(plain(15))));
    out.push(build("circles_15x15", with_circles(plain(15))));
    out.push(build("given_15x15", with_given(plain(15))));

    out
}

/// Serialize a puzzle into a `Fixture`, panicking if the writer rejects it
/// (which would indicate a bug in the generator).
fn build(name: &str, puzzle: Puzzle) -> Fixture {
    let bytes = puz_parse::to_bytes(&puzzle)
        .unwrap_or_else(|e| panic!("fixture {name} failed to serialize: {e}"));
    Fixture {
        name: name.to_string(),
        puzzle,
        bytes,
    }
}

/// Build a plain `n`x`n` puzzle with no black squares and every implied clue
/// filled in. An open grid's numbering is simple: each row starts one across
/// word and each column starts one down word, numbered in reading order.
fn plain(n: usize) -> Puzzle {
    let solution: Vec<String> = (0..n)
        .map(|r| (0..n).map(|c| letter(r, c)).collect())
        .collect();
    let blank: Vec<String> = (0..n).map(|_| "-".repeat(n)).collect();

    // Numbering: walk reading order; a cell starts across if col == 0, starts
    // down if row == 0 (true for an all-open grid).
    let mut across = HashMap::new();
    let mut down = HashMap::new();
    let mut number = 1u16;
    for r in 0..n {
        for c in 0..n {
            let starts_across = c == 0;
            let starts_down = r == 0;
            if starts_across || starts_down {
                if starts_across {
                    across.insert(number, format!("across {number}"));
                }
                if starts_down {
                    down.insert(number, format!("down {number}"));
                }
                number += 1;
            }
        }
    }

    Puzzle {
        info: PuzzleInfo {
            title: "Benchmark".into(),
            author: "Generated".into(),
            copyright: "(c) puz.rs".into(),
            notes: String::new(),
            width: n as u8,
            height: n as u8,
            version: "1.3".into(),
            is_scrambled: false,
        },
        grid: Grid { solution, blank },
        clues: Clues { across, down },
        extensions: Extensions {
            rebus: None,
            circles: None,
            given: None,
        },
    }
}

/// Deterministic letter for cell `(r, c)`.
fn letter(r: usize, c: usize) -> char {
    (b'A' + ((r * 7 + c * 3) % 26) as u8) as char
}

fn with_rebus(mut p: Puzzle) -> Puzzle {
    let (w, h) = (p.info.width as usize, p.info.height as usize);
    let mut grid = vec![vec![0u8; w]; h];
    grid[0][0] = 1; // one rebus cell
    let mut table = HashMap::new();
    table.insert(1u8, "HEART".to_string());
    p.extensions.rebus = Some(Rebus { grid, table });
    p
}

fn with_circles(mut p: Puzzle) -> Puzzle {
    let (w, h) = (p.info.width as usize, p.info.height as usize);
    // Circle the main diagonal.
    let grid = (0..h).map(|r| (0..w).map(|c| r == c).collect()).collect();
    p.extensions.circles = Some(grid);
    p
}

fn with_given(mut p: Puzzle) -> Puzzle {
    let (w, h) = (p.info.width as usize, p.info.height as usize);
    // Mark the first row as given.
    let grid = (0..h).map(|r| (0..w).map(|_| r == 0).collect()).collect();
    p.extensions.given = Some(grid);
    p
}
